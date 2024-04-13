mod structs;
use std::collections::HashMap;
use std::env::var;
use rsheet_lib::connect::{ConnectionError, Manager, Reader, ReaderWriter, Writer};
use rsheet_lib::replies::Reply;

use std::error::Error;
use std::fmt::format;

use log::info;
use rsheet_lib::cell_value::CellValue;
use rsheet_lib::command_runner::{CellArgument, CommandRunner};
use fancy_regex::Regex;
use rsheet_lib::cells::{column_name_to_number, column_number_to_name};
use crate::structs::Command;

pub fn start_server<M>(mut manager: M) -> Result<(), Box<dyn Error>>
where
    M: Manager,
{
    let mut cells: HashMap<String, CellValue>= HashMap::new();
    let (mut recv, mut send) = manager.accept_new_connection().unwrap();
    loop {
        info!("Just got message");
        let msg = match recv.read_message() {
            Ok(msg) => msg,
            Err(_) => return Ok(())
        };
        let command = match parse_command(msg, &mut cells) {
            Ok(command) => command,
            Err(err) => {
                send.write_message(Reply::Error(err));
                Command::None
            },
        };
        match execute_command(command, &mut cells) {
            Err(err) => {
                send.write_message(Reply::Error(err));
            },
            Ok(Some(reply)) => {
                send.write_message(reply);
            }
            Ok(None) => {}
        };
    }
}

fn parse_command(msg: String, cells: &mut HashMap<String, CellValue>) -> Result<Command, String> {
    let mut words = msg.split_whitespace();
    let cell_address_regex: Regex = Regex::new(r"^[A-Z]+\d+$").unwrap();
    if let Some(first_word) = words.next() {
        match first_word {
            "get" => {
                let remainder = words.collect::<Vec<&str>>().join(" ");
                if cell_address_regex.is_match(&remainder).unwrap() {
                    Ok(Command::Get(remainder))
                } else {
                    Err("Invalid cell reference in get command".to_string())
                }
            },
            "set" => {
                let addr = match words.next() {
                    None => return Err("No cell reference given in set command".to_string()),
                    Some(addr) => addr
                };
                if cell_address_regex.is_match(addr).unwrap() {
                    let expression = words.collect::<Vec<&str>>().join(" ");
                    if expression.len() == 0 {
                        return Err("Must provide expression for set".to_string());
                    }
                    Ok(Command::Set(addr.to_string(), expression))
                } else {
                    Err("Invalid cell reference in set command".to_string())
                }
            },
            _ => return Err("Invalid operation: ".to_string() + first_word),
        }
    } else {
        Err("No operation specified.".to_string())
    }
}

fn execute_command(command: Command, cells: &mut HashMap<String, CellValue>) -> Result<Option<Reply>, String> {
    match command {
        Command::Get(addr) => {
            // Handle case where cell contains dependency error
            if cells.contains_key(&addr) {
                if let CellValue::Error(err) = cells[&addr].clone() {
                    if err.eq("Dependency error") {
                        return Err(err);
                    }
                }
            }
            // Handles case where cells doesn't contain the address
            Ok(Some(Reply::Value(addr.clone(), cells.entry(addr).or_insert(CellValue::None).clone())))
        }
        Command::Set(addr, expression) => {
            let runner = CommandRunner::new(&expression);
            let variables = runner.find_variables();
            let result_map = match convert_variables(variables, cells) {
                Ok(result_map) => result_map,
                Err(err) => {
                    // Check for dependency error
                    if err.eq("Dependency error") {
                        cells.insert(addr, CellValue::Error(err));
                    }
                    return Ok(None);
                }
            };
            cells.insert(addr, CommandRunner::new(&expression).run(&result_map));
            return Ok(None);
        }
        Command::None => return Ok(None)
    }
}

// converts from Vec<String> to HashMap<String, CellArgument>
fn convert_variables(variables: Vec<String>, cells: &mut HashMap<String, CellValue>) -> Result<HashMap<String, CellArgument>, String> {

    let scalar_variable_regex: Regex = Regex::new(r"^[A-Z]+\d+$").unwrap();
    let vector_variable_regex = Regex::new(r"^[A-Z]+(\d+)_[A-Z]+\1$").unwrap();
    let matrix_variable_regex = Regex::new(r"^[A-Z]+\d+_[A-Z]+\d+$").unwrap();
    let column_regex = Regex::new(r"[A-Z]+").unwrap();
    let row_regex = Regex::new(r"\d+").unwrap();

    let mut result_map: HashMap<String, CellArgument> = HashMap::new();
    for variable in variables {

        // Simple case of scalar variable
        if scalar_variable_regex.is_match(&variable).unwrap() {
            if cells.contains_key(&variable) {
                if let CellValue::Error(_) = cells[&variable].clone() {
                    return Err("Dependency error".to_string());
                }
            }
            let cell_value = cells.entry(variable.clone()).or_insert(CellValue::None).clone();
            result_map.insert(variable, CellArgument::Value(cell_value));
        }

        // Vector variables case
        else if vector_variable_regex.is_match(&variable).unwrap() {
            let mut vector_variables: Vec<CellValue> = Vec::new();
            let mut row = row_regex.find(&variable).unwrap().unwrap().as_str();
            let mut columns = Vec::new();
            for column in column_regex.find_iter(&variable) {
                columns.push(column.unwrap().as_str());
            }
            if columns.len() != 2 {
                let len = columns.len();
                return Err(format!("Invalid cell address format: {variable}"));
            }

            let start_idx = column_name_to_number(columns[0]);
            let finish_idx = column_name_to_number(columns[1]);
            for col in start_idx..=finish_idx {
                let current_cell = column_number_to_name(col) + row;
                if cells.contains_key(&current_cell) {
                    if let CellValue::Error(_) = cells[&current_cell].clone() {
                        return Err("Dependency error".to_string());
                    }
                }
                vector_variables.push(cells.entry(current_cell).or_insert(CellValue::None).clone());
            }
            result_map.insert(variable, CellArgument::Vector(vector_variables));
        }

        // Matrix variables case
        else if matrix_variable_regex.is_match(&variable).unwrap() {
            let mut matrix_variables: Vec<Vec<CellValue>> = Vec::new();
            let mut rows: Vec<u32> = Vec::new();
            let mut columns: Vec<&str> = Vec::new();
            for row in row_regex.find_iter(&variable) {
                let res = row.clone().unwrap().as_str().parse::<u32>().unwrap();
                rows.push(row.unwrap().as_str().parse::<u32>().unwrap());
            }
            for column in column_regex.find_iter(&variable) {
                columns.push(column.unwrap().as_str());
            }
            if columns.len() != 2 || rows.len() != 2 {
                let len = columns.len();
                return Err(format!("Invalid cell address format: {variable}"));
            }

            let start_row_idx = rows[0];
            let finish_row_idx = rows[1];
            let start_col_idx = column_name_to_number(columns[0]);
            let finish_col_idx = column_name_to_number(columns[1]);
            for row in start_row_idx..=finish_row_idx {
                let mut vector_variables: Vec<CellValue> = Vec::new();
                for col in start_col_idx..=finish_col_idx {
                    let current_cell = column_number_to_name(col) + row.to_string().as_str();
                    if cells.contains_key(&current_cell) {
                        if let CellValue::Error(_) = cells[&current_cell].clone() {
                            return Err("Dependency error".to_string());
                        }
                    }
                    vector_variables.push(cells.entry(current_cell).or_insert(CellValue::None).clone());
                }
                matrix_variables.push(vector_variables);
            }
            result_map.insert(variable, CellArgument::Matrix(matrix_variables));
        }
        else {
            return Err(format!("Invalid cell address format: {variable}"));
        }
    }
    Ok(result_map)
}
