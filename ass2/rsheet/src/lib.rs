mod structs;
use std::collections::HashMap;
use std::env::var;
use rsheet_lib::connect::{ConnectionError, Manager, Reader, ReaderWriter, Writer};
use rsheet_lib::replies::Reply;

use std::error::Error;
use std::fmt::format;
use std::sync::{Arc, mpsc, RwLock, RwLockReadGuard};
use std::thread;

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
    let mut cells: Arc<RwLock<HashMap<String, CellValue>>> = Arc::new(RwLock::new(HashMap::new()));

    thread::scope(|s| {
        while let Ok((recv, send)) = manager.accept_new_connection() {
            let cells = cells.clone();
            s.spawn(|| handle_connection(Box::new(recv), Box::new(send), cells));
        }
    });
    Ok(())
}

fn handle_connection(mut recv: Box<dyn Reader>, mut send: Box<dyn Writer>, mut cells: Arc<RwLock<HashMap<String, CellValue>>>) -> Result<(), ()> {
    loop {
        info!("Just got message");
        let msg = match recv.read_message() {
            Ok(msg) => msg,
            Err(_) => continue
        };
        let command = match parse_command(msg) {
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
            },
            Ok(None) => continue,
        };
    }
}

fn parse_command(msg: String) -> Result<Command, String> {
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

fn execute_command(command: Command, cells: &mut Arc<RwLock<HashMap<String, CellValue>>>) -> Result<Option<Reply>, String> {
    match command {
        Command::Get(addr) => {
            // Handle case where cell contains dependency error
            let mut cells = cells.read().unwrap();
            if cells.contains_key(&addr) {
                match cells[&addr].clone() {
                    CellValue::Error(err) => {
                        if err.eq("Dependency error") { return Err(err) };
                    },
                    _ => {},
                };
                return Ok(Some(Reply::Value(addr.clone(), cells[&addr].clone())))
            }
            Ok(Some(Reply::Value(addr.clone(), CellValue::None)))
        }
        Command::Set(addr, expression) => {
            let runner = CommandRunner::new(&expression);
            let variables = runner.find_variables();
            let result_map = match convert_variables(variables, cells) {
                Ok(result_map) => result_map,
                Err(err) => {
                    cells.write().unwrap().insert(addr, CellValue::Error(err));
                    return Ok(None);
                }
            };
            cells.write().unwrap().insert(addr, CommandRunner::new(&expression).run(&result_map));
            //println!("Cells is now {:?}", cells.read().unwrap());
            return Ok(None);

        }
        Command::None => return Ok(None)
    }
}

// converts from Vec<String> to HashMap<String, CellArgument>
fn convert_variables(variables: Vec<String>, cells: &mut Arc<RwLock<HashMap<String, CellValue>>>) -> Result<HashMap<String, CellArgument>, String> {

    let scalar_variable_regex: Regex = Regex::new(r"^[A-Z]+\d+$").unwrap();
    let vector_variable_regex = Regex::new(r"^[A-Z]+(\d+)_[A-Z]+\1$").unwrap();
    let matrix_variable_regex = Regex::new(r"^[A-Z]+\d+_[A-Z]+\d+$").unwrap();
    let column_regex = Regex::new(r"[A-Z]+").unwrap();
    let row_regex = Regex::new(r"\d+").unwrap();

    let mut result_map: HashMap<String, CellArgument> = HashMap::new();
    let cells = cells.read().unwrap();
    for variable in variables {

        // Simple case of scalar variable
        if scalar_variable_regex.is_match(&variable).unwrap() {
            let current_cell = variable;
            match get_cell_value(&current_cell, &cells) {
                Ok(value) => { result_map.insert(current_cell, CellArgument::Value(value)) },
                Err(err) => { return Err(err)}
            };
        }

        // Vector variables case
        else if vector_variable_regex.is_match(&variable).unwrap() {
            let mut vector_variables: Vec<CellValue> = Vec::new();
            let row = row_regex.find(&variable).unwrap().unwrap().as_str();
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
                match get_cell_value(&current_cell, &cells) {
                    Ok(value) => { vector_variables.push(value) }
                    Err(err) => { return Err(err)}
                };
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
                    match get_cell_value(&current_cell, &cells) {
                        Ok(value) => { vector_variables.push(value) }
                        Err(err) => { return Err(err)}
                    };
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

// TODO: FIX THIS
fn get_cell_value(current_cell: &String, cells: &RwLockReadGuard<HashMap<String, CellValue>>) -> Result<CellValue, String> {
    if cells.contains_key(current_cell) {
        match cells[current_cell].clone() {
            CellValue::Error(_) => {
                return Err("Dependency error".to_string());
            },
            _ => {},
        };
        return Ok(cells[current_cell].clone());
    }
    return Ok(CellValue::None);
}
