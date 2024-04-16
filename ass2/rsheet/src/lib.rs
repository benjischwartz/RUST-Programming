mod structs;
use std::collections::{HashMap, HashSet};
use rsheet_lib::connect::{Manager, Reader, Writer};
use rsheet_lib::replies::Reply;

use std::error::Error;
use std::sync::{Arc, mpsc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

use log::info;
use rsheet_lib::cell_value::CellValue;
use rsheet_lib::command_runner::{CellArgument, CommandRunner};
use fancy_regex::Regex;
use rsheet_lib::cells::{column_name_to_number, column_number_to_name};
use crate::structs::{Command, DependencyNode};

pub fn start_server<M>(mut manager: M) -> Result<(), Box<dyn Error>>
where
    M: Manager,
{
    let cells: Arc<RwLock<HashMap<String, CellValue>>> = Arc::new(RwLock::new(HashMap::new()));
    let dependencies: Arc<RwLock<HashMap<String, DependencyNode>>> = Arc::new(RwLock::new(HashMap::new()));

    let (tx, rx) = mpsc::channel();

    let handle = {
        let cells = cells.clone();
        thread::spawn(move || handle_dependency_updates(&cells, rx, &dependencies))
    };

    thread::scope(|s| {
        while let Ok((recv, send)) = manager.accept_new_connection() {
            let cells = cells.clone();
            let tx = tx.clone();
            s.spawn(|| handle_connection(Box::new(recv), Box::new(send), cells, tx));
        }
    });

    drop(tx);
    handle.join().unwrap();
    Ok(())
}

fn handle_connection(mut recv: Box<dyn Reader>,
                     mut send: Box<dyn Writer>,
                     mut cells: Arc<RwLock<HashMap<String, CellValue>>>,
                     tx: Sender<(String, String, HashSet<String>)>) -> Result<(), ()>
{
    loop {
        info!("Just got message");
        let msg = match recv.read_message() {
            Ok(msg) => msg,
            Err(_) => return Ok(())
        };
        let command = match parse_command(msg) {
            Ok(command) => command,
            Err(err) => {
                let _ = send.write_message(Reply::Error(err));
                Command::None
            },
        };
        let tx = tx.clone();
        match execute_command(command, &mut cells, tx) {
            Err(err) => {
                let _ = send.write_message(Reply::Error(err));
            },
            Ok(Some(reply)) => {
                let _ = send.write_message(reply);
            },
            Ok(None) => {},
        };
    }
}

/*
For each variable in formula, create if !exists and draw node to current.
Update current node formula.
Update all downstream nodes.
 */
fn handle_dependency_updates(cells: &Arc<RwLock<HashMap<String, CellValue>>>,
                             rx: Receiver<(String, String, HashSet<String>)>,
                             dependencies: &Arc<RwLock<HashMap<String, DependencyNode>>>)
{
    loop {
       match rx.recv() {
            Ok((cell_address, formula, upstream_dependencies)) => {
                let mut dependencies = dependencies.write().unwrap();

                // CREATE OR UPDATE THE CURRENT
                if dependencies.contains_key(&cell_address) {
                    let existing_neighbors = dependencies[&cell_address].neighbors.clone();
                    let updated_node = DependencyNode{ address: cell_address.clone(), formula, neighbors: existing_neighbors };
                    dependencies.insert(cell_address.clone(),  updated_node);
                } else {
                    let new_node = DependencyNode { address: cell_address.clone(), formula, neighbors: Default::default() };
                    dependencies.insert(cell_address.clone(), new_node);
                }

                // CREATE OR UPDATE THE NEIGHBORS
                for upstream_dep in &upstream_dependencies {
                    if !dependencies.contains_key(upstream_dep) {
                        let new_node = DependencyNode{ address: upstream_dep.clone(), formula: "".to_string(), neighbors: HashSet::from([cell_address.clone()])};
                        dependencies.insert(upstream_dep.clone(), new_node);
                    } else {
                        let mut existing_neighbors = dependencies[upstream_dep].neighbors.clone();
                        existing_neighbors.insert(cell_address.clone());
                        let existing_formula = dependencies[upstream_dep].formula.clone();
                        let updated_node = DependencyNode{ address: upstream_dep.clone(), formula: existing_formula, neighbors: existing_neighbors};
                        dependencies.insert(upstream_dep.clone(), updated_node);
                    }
                }

                // REMOVE OLD CONNECTIONS
                for (addr, node) in dependencies.iter_mut() {
                    if node.neighbors.contains(&cell_address) && !upstream_dependencies.contains(addr) {
                        node.neighbors.remove(&cell_address);
                    }
                }

                // CYCLE DETECT
                let mut rec_stack: HashSet<String> = HashSet::new();
                let mut visited: HashSet<String> = HashSet::new();
                if detect_cycle(&dependencies, &dependencies[&cell_address], &mut rec_stack, &mut visited) {
                    let mut cells = cells.write().unwrap();
                    for node in rec_stack {
                        cells.insert(node, CellValue::Error("Circular dependency error".to_string()));
                        continue;
                    }
                }

                // UPDATE ALL DOWNSTREAM CELLS
                for neighbor in dependencies[&cell_address].neighbors.clone() {
                    dfs_update_dependencies(cells, &dependencies, &neighbor);
                }

            }
            Err(_) => {return;}
        };
    }
}

fn parse_command(msg: String) -> Result<Command, String>
{
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
                    if expression.is_empty() {
                        return Err("Must provide expression for set".to_string());
                    }
                    Ok(Command::Set(addr.to_string(), expression))
                } else {
                    Err("Invalid cell reference in set command".to_string())
                }
            },
            _ => Err("Invalid operation: ".to_string() + first_word),
        }
    } else {
        Err("No operation specified.".to_string())
    }
}

fn execute_command(command: Command,
                   cells: &mut Arc<RwLock<HashMap<String, CellValue>>>,
                   tx: Sender<(String, String, HashSet<String>)>) -> Result<Option<Reply>, String>
{
    match command {
        Command::Get(addr) => {
            // Handle case where cell contains dependency error
            let cells = cells.read().unwrap();
            if cells.contains_key(&addr) {
                if let CellValue::Error(err) = cells[&addr].clone() {
                    if err.eq("Dependency error") || err.eq("Circular dependency error") { return Err(err) };
                };
                return Ok(Some(Reply::Value(addr.clone(), cells[&addr].clone())))
            }
            Ok(Some(Reply::Value(addr.clone(), CellValue::None)))
        }
        Command::Set(addr, expression) => {
            let runner = CommandRunner::new(&expression);
            let variables = runner.find_variables();
            let mut upstream_dependencies: HashSet<String> = HashSet::new();
            let result_map = match convert_variables(variables, cells, &mut upstream_dependencies) {
                Ok(result_map) => result_map,
                Err(err) => {
                    cells.write().unwrap().insert(addr, CellValue::Error(err));
                    return Ok(None);
                }
            };
            cells.write().unwrap().insert(addr.clone(), CommandRunner::new(&expression).run(&result_map));
            match tx.send((addr, expression, upstream_dependencies)) {
                Ok(_) => {}
                Err(err) => {println!("{}", err)}
            };
            Ok(None)
        }
        Command::None => Ok(None)
    }
}

// converts from Vec<String> to HashMap<String, CellArgument>
fn convert_variables(variables: Vec<String>,
                     cells: &Arc<RwLock<HashMap<String, CellValue>>>,
                     dependencies: &mut HashSet<String>) -> Result<HashMap<String, CellArgument>, String>
{

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
            dependencies.insert(current_cell.clone());
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
                return Err(format!("Invalid cell address format: {variable}"));
            }

            let start_idx = column_name_to_number(columns[0]);
            let finish_idx = column_name_to_number(columns[1]);
            for col in start_idx..=finish_idx {
                let current_cell = column_number_to_name(col) + row;
                dependencies.insert(current_cell.clone());
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
                rows.push(row.unwrap().as_str().parse::<u32>().unwrap());
            }
            for column in column_regex.find_iter(&variable) {
                columns.push(column.unwrap().as_str());
            }
            if columns.len() != 2 || rows.len() != 2 {
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
                    dependencies.insert(current_cell.clone());
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

fn get_cell_value(current_cell: &String,
                  cells: &RwLockReadGuard<HashMap<String, CellValue>>) -> Result<CellValue, String>
{
    if cells.contains_key(current_cell) {
        if let CellValue::Error(_) = cells[current_cell].clone() {
            return Err("Dependency error".to_string());
        };
        return Ok(cells[current_cell].clone());
    }
    Ok(CellValue::None)
}

fn dfs_update_dependencies(cells: &Arc<RwLock<HashMap<String, CellValue>>>,
                           dependencies: &RwLockWriteGuard<HashMap<String, DependencyNode>>,
                           cell_address: &String)
{
    let dep_node = &dependencies[cell_address];
    // Update cell
    let runner = CommandRunner::new(&dep_node.formula);
    let variables = runner.find_variables();
    let result_map = match convert_variables(variables, cells, &mut Default::default()) {
        Ok(result_map) => result_map,
        Err(_) => return
    };
    cells.write().unwrap().insert(cell_address.clone(), runner.run(&result_map));

    // Recursively update neighbors
    for neighbor in &dep_node.neighbors {
        dfs_update_dependencies(cells, dependencies, neighbor);
    }
}

fn detect_cycle(dependencies: &RwLockWriteGuard<HashMap<String, DependencyNode>>,
                node: &DependencyNode,
                rec_stack: &mut HashSet<String>,
                visited: &mut HashSet<String>) -> bool
{
    //println!("In cycle detect");
    if !visited.contains(&node.address) {
        visited.insert(node.address.clone());
        rec_stack.insert(node.address.clone());
        for neighbour in &node.neighbors {
            if !visited.contains(neighbour) && detect_cycle(dependencies, &dependencies[neighbour], rec_stack, visited) || rec_stack.contains(neighbour) {
                return true
            }
        }
    }
    rec_stack.remove(&node.address);
    false
}