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
use regex::Regex;
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
                if cell_address_regex.is_match(&remainder) {
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
                if cell_address_regex.is_match(addr) {
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
            if cells.contains_key(&addr) {
                let cell_value = cells[&addr].clone();
                Some(Reply::Value(addr, cell_value))
            } else {
                Some(Reply::Value(addr, CellValue::None))
            }
        }
        Command::Set(addr, expression) => {
            let runner = CommandRunner::new(&expression);
            let variables = runner.find_variables();

            cells.insert(addr, CommandRunner::new(&expression).run(&Default::default()));
            None
        }
        Command::None => None
    }
}

// converts from Vec<String> to HashMap<String, CellArgument>
fn convert_variables(variables: Vec<String>, cells: &HashMap<String, CellValue>) -> HashMap<String, CellArgument> {

    let scalar_variable_regex: Regex = Regex::new(r"^[A-Z]+\d+$").unwrap();

    // let vector_variable_regex: Regex = Regex::new(r"^[A-Z]+\d+_[A-Z]+\d+$").unwrap();
    // let matrix_variable_regex: Regex = Regex::new(r"^[A-Z]+\d+_[A-Z]+\d+$").unwrap();

    let mut result_map: HashMap<String, CellArgument> = HashMap::new();
    for variable in variables {
        // Simple case of scalar variable
        if scalar_variable_regex.is_match(&variable) {
            let cell_value = cells[&variable].clone();
            result_map.insert(variable, CellArgument::Value(cell_value));
        }
    }
    todo!()
}
