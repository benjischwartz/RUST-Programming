mod structs;
use std::collections::HashMap;
use std::env::var;
use rsheet_lib::connect::{Manager, Reader, Writer};
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
        let msg = recv.read_message()?;
        let command = match parse_command(msg, &mut cells) {
            Ok(command) => command,
            Err(err) => {
                send.write_message(Reply::Error(err));
                Command::None
            },
        };
        match execute_command(command, &mut cells) {
            None => {},
            Some(reply) => {
                send.write_message(reply);
            }
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

fn execute_command(command: Command, cells: &mut HashMap<String, CellValue>) -> Option<Reply> {
    match command {
        Command::Get(addr) => {
            // Handles case where cells doesn't contain the address
            Some(Reply::Value(addr.clone(), cells.entry(addr).or_insert(CellValue::None).clone()))
        }
        Command::Set(addr, expression) => {
            let runner = CommandRunner::new(&expression);
            let variables = runner.find_variables();
            let map = convert_variables(variables, cells);
            cells.insert(addr, CommandRunner::new(&expression).run(&Default::default()));
            None
        }
        Command::None => None
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
            println!("{variable} is a scalar variable");

            // Handles case where cells doesn't contain the variable
            let cell_value = cells.entry(variable.clone()).or_insert(CellValue::None).clone();
            result_map.insert(variable, CellArgument::Value(cell_value));
        }
        else if vector_variable_regex.is_match(&variable).unwrap() {
            println!("{variable} is a vector variable");
            let mut vector_variables: Vec<CellValue> = Vec::new();
            let mut row = row_regex.find(&variable).unwrap().unwrap().as_str();
            let mut columns = Vec::new();
            for column in column_regex.find_iter(&variable) {
                columns.push(column.unwrap().as_str());
            }
            if columns.len() != 2 {
                println!("Invalid cell address format: {variable}");
                let len = columns.len();
                return Err(format!("Invalid cell address format: {variable}"));
            }

            let start_idx = column_name_to_number(columns[0]);
            let finish_idx = column_name_to_number(columns[1]);
            for col in start_idx..=finish_idx {
                let current_cell = column_number_to_name(col) + row;
                vector_variables.push(cells.entry(current_cell).or_insert(CellValue::None).clone());
            }
            println!("DONE: {:?}", vector_variables);
            result_map.insert(variable, CellArgument::Vector(vector_variables));
        }
        else if matrix_variable_regex.is_match(&variable).unwrap() {
            println!("{variable} is a matrix variable");
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
                println!("Invalid cell address format: {variable}");
                let len = columns.len();
                return Err(format!("Invalid cell address format: {variable}"));
            }

            let start_row_idx = rows[0];
            println!("start row: {start_row_idx}");
            let finish_row_idx = rows[1];
            println!("finish row: {finish_row_idx}");
            let start_col_idx = column_name_to_number(columns[0]);
            let finish_col_idx = column_name_to_number(columns[1]);
            for row in start_row_idx..=finish_row_idx {
                let mut vector_variables: Vec<CellValue> = Vec::new();
                for col in start_col_idx..=finish_col_idx {
                    let current_cell = column_number_to_name(col) + row.to_string().as_str();
                    let clone = current_cell.clone();
                    println!("On current cell: {clone}");
                    vector_variables.push(cells.entry(current_cell).or_insert(CellValue::None).clone());
                }
                matrix_variables.push(vector_variables);
            }
            println!("DONE: {:?}", matrix_variables);
            result_map.insert(variable, CellArgument::Matrix(matrix_variables));
        }
        else {
            println!("Unknown variable type");
        }
    }
    Ok(result_map)
}
