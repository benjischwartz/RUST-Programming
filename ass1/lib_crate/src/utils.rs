use std::collections::HashMap;
use unsvg::{Image, get_end_coordinates, COLORS};
use crate::structs::{Cursor, Procedure};

fn parse_procedure(token: &str, name: Option<String>, value: f32) -> Option<Procedure> {
    match token {
        "FORWARD" => Some(Procedure::FORWARD(value)),
        "BACK" => Some(Procedure::BACK(value)),
        "LEFT" => Some(Procedure::LEFT(value)),
        "RIGHT" => Some(Procedure::RIGHT(value)),
        "SETPENCOLOR" => Some(Procedure::SETPENCOLOR(value)),
        "TURN" => Some(Procedure::TURN(value)),
        "SETHEADING" => Some(Procedure::SETHEADING(value)),
        "SETX" => Some(Procedure::SETX(value)),
        "SETY" => Some(Procedure::SETY(value)),
        "MAKE" => Some(Procedure::MAKE(name.unwrap(), value)),
        "XCOR" => Some(Procedure::XCOR),
        "YCOR" => Some(Procedure::YCOR),
        "HEADING" => Some(Procedure::HEADING),
        "COLOR" => Some(Procedure::COLOR),
        "ADDASSIGN" => Some(Procedure::ADDASSIGN(name.unwrap(), value)),
        _ => None
    }
}

pub fn handle_line(line: &str, image: &mut Image, cursor: &mut Cursor, variables: &mut HashMap<String, f32>) -> Result<usize, String> {
    if line.starts_with("//") {
        println!("Skipping... comment");
        return Ok(0);
    }

    // TOKEN PROCESSING
    let tokens: Vec<& str> = line.split_whitespace().collect();
    let mut iter = tokens.iter().peekable();
    while let Some(token) = iter.next() {
        match *token {
            "PENUP" => {
                execute_procedure(image, Procedure::PENUP, cursor, variables);
            },
            "PENDOWN" => {
                execute_procedure(image, Procedure::PENDOWN, cursor, variables);
            },
            "FORWARD" | "BACK" | "LEFT" | "RIGHT" | "SETPENCOLOR" |
            "TURN" | "SETHEADING" | "SETX" | "SETY" => {
                if let Some(value) = iter.next() {
                    // VALUE CASE
                    if value.starts_with('"') {
                        let trimmed_token = value.trim_matches('"');
                        let res = match get_bool_as_f32(trimmed_token) {
                            Some(res) => res,
                            None => {
                                match trimmed_token.parse::<f32>(){
                                    Ok(res) => res,
                                    Err(_) => return Err("Couldn't parse value!".to_string()),
                                }
                            }
                        };
                        let procedure = parse_procedure(token, None, res).expect("Should be a valid command");
                        execute_procedure(image, procedure, cursor, variables);
                    }
                    // VARIABLE CASE
                    else if value.starts_with(':') {
                        let name = value.trim_matches(':').to_string();
                        match variables.get(&name) {
                            Some(value) => {
                                let procedure = parse_procedure(token, None, *value)
                                    .expect("Should be a valid command");
                                execute_procedure(image, procedure, cursor, variables);
                            },
                            None => {
                                return Err("No matching variable found!".to_string());
                            }
                        }
                    }
                    else {
                        match get_query(value, cursor) {
                            Some(value) => {
                                let procedure = parse_procedure(token, None, value)
                                    .expect("Should be a valid command");
                                execute_procedure(image, procedure, cursor, variables);
                            },
                            None => {return Err("Expected arg!".to_string()); }
                        }
                    }
                }
                else {
                    return Err("Expected arg!".to_string());
                }
            },
            "MAKE" | "ADDASSIGN" => {
                if let Some(name) = iter.next() {
                    if name.starts_with('"') {
                        if let Ok(name) = name.trim_matches('"').parse::<String>() {
                            if let Some(value) = iter.next() {
                                if value.starts_with('"') {
                                    let trimmed_token = value.trim_matches('"');
                                    let res = match get_bool_as_f32(trimmed_token) {
                                        Some(res) => res,
                                        None => {
                                            match trimmed_token.parse::<f32>(){
                                                Ok(res) => res,
                                                Err(_) => return Err("Couldn't parse second arg!".to_string()),
                                            }
                                        }
                                    };
                                    let procedure = parse_procedure(token, Some(name), res)
                                        .expect("Should be a valid command");
                                    execute_procedure(image, procedure, cursor, variables);
                                }
                                else if value.starts_with(':') {
                                    let variable = value.trim_matches(':').to_string();
                                    match variables.get(&variable) {
                                        Some(value) => {
                                            let procedure = parse_procedure(token, Some(name), *value)
                                                .expect("Should be a valid command");
                                            execute_procedure(image, procedure, cursor, variables);
                                        },
                                        None => {
                                            return Err("No matching variable found".to_string());
                                        }
                                    }
                                }
                                else {
                                    match get_query(value, cursor) {
                                        Some(value) => {
                                            let procedure = parse_procedure(token, Some(name), value)
                                                .expect("Should be a valid command");
                                            execute_procedure(image, procedure, cursor, variables);
                                        },
                                        None => {return Err("Expected arg!".to_string()); }
                                    }
                                }
                            }
                        }
                    }
                    else {
                        return Err("Couldn't parse variable name!".to_string());
                    }
                }
                else {
                    return Err("Expected arg!".to_string())
                };
            }
            value => {
                if value.starts_with('"') {
                    value.trim_matches('"').parse::<f32>();
                    return Err("Too many args!".to_string());
                }
                else {
                    return Err("Not implemented yet!".to_string());
                }
            }
        }
    }
    Ok(0)
}

fn execute_procedure(image: &mut Image, procedure: Procedure, cursor: &mut Cursor, variables: &mut HashMap<String, f32>) -> Result<(), String>
{
    println!("Procedure is {:?}", procedure);
    match procedure {
        Procedure::PENUP => {
            cursor.penup();
        },
        Procedure::PENDOWN => {
            println!("Putting pen down");
            cursor.pendown();
        },
        Procedure::FORWARD(value) => {
            move_cursor(image, cursor, cursor.direction, value);
        },
        Procedure::BACK(value) => {
            move_cursor(image, cursor, cursor.direction + 180, value);
        },
        Procedure::LEFT(value) => {
            move_cursor(image, cursor, cursor.direction + 270, value);
        },
        Procedure::RIGHT(value) => {
            move_cursor(image, cursor, cursor.direction + 90, value);
        },
        Procedure::SETPENCOLOR(value) => {
            // Error if not integer or between 0 and 15
            if value.fract() != 0.0 || value < 0.0 || value > 15.0 {
                return Err("Pen Color not valid".to_string());
            }
            cursor.pen_color = COLORS[value as usize];
        },
        Procedure::TURN(value) => {
            if value.fract() != 0.0 {
                return Err("Turn Value must be i32".to_string());
            }
            cursor.direction += value as i32;
        },
        Procedure::SETHEADING(value) => {
            if value.fract() != 0.0 {
                return Err("Set Heading Value must be i32".to_string());
            }
            cursor.direction = value as i32;
        },
        Procedure::SETX(value) => {
            cursor.x_coord = value;
        },
        Procedure::SETY(value) => {
            cursor.y_coord = value;
        },
        Procedure::ADDASSIGN(name, value) => {
            println!("Updating variables...");
            println!("Adding {value} to {name}");
            match variables.get_mut(&name) {
                Some(val) => {
                    println!("old {val}");
                    *val = *val + value;
                    println!("new {val}");
                },
                None => {
                    return Err("Variable does not exist".to_string());
                }
            }
        },
        Procedure::MAKE(name, value) => {
            println!("Updating variables...");
            println!("Adding {name}, {value}");
            variables.insert(name, value);
        }
        _ => { todo!()}
    };
    Ok(())
}

fn move_cursor(image: &mut Image, cursor: &mut Cursor, direction: i32, length: f32) {
    println!("In move cursor, cursor down {}", cursor.isdown());
    if cursor.isdown() {
        println!("Drawing!");
        image.draw_simple_line(cursor.x_coord, cursor.y_coord, direction, length, cursor.pen_color);
    }
    let coords = get_end_coordinates(cursor.x_coord, cursor.y_coord, direction, length);
    cursor.x_coord = coords.0;
    cursor.y_coord = coords.1;
}

fn get_query(query: &str, cursor: &mut Cursor) -> Option<f32> {
    match query {
        "XCOR" => Some(cursor.x_coord),
        "YCOR" => Some(cursor.y_coord),
        "HEADING" => Some(cursor.direction as f32),
        "COLOR" => Some(cursor.color_as_f32()),
        _ => None,
    }
}

// Assumes line in form "<value1> <value2>"
// values can be raw values, queries, or variables
pub fn check_equality(line: &str, cursor: &mut Cursor, variables: &mut HashMap<String, f32>) -> Result<bool, String>
{
    // Get first arg
    let tokens: Vec<& str> = line.split_whitespace().collect();
    let a: f32;
    let b: f32;
    if tokens[0].starts_with('"') {
        let trimmed_token = tokens[0].trim_matches('"');
        a = match get_bool_as_f32(trimmed_token) {
            Some(a) => a,
            None => {
                match trimmed_token.parse::<f32>(){
                    Ok(a) => a,
                    Err(_) => return Err("Failed to parse first arg!".to_string()),
                }
            }
        }
    } else if tokens[0].starts_with(':') {
        let name = tokens[0].trim_matches(':').to_string();
        a =  match variables.get(&name) {
            Some(value) => *value,
            None => return Err("No matching variable found".to_string()),
        };
    } else {
        a = match get_query(tokens[0], cursor) {
            Some(value) => value,
            None => return Err("Expected arg!".to_string())
        };
    }
    if tokens[1].starts_with('"') {
        let trimmed_token = tokens[1].trim_matches('"');
        b = match get_bool_as_f32(trimmed_token) {
            Some(b) => b,
            None => {
                match trimmed_token.parse::<f32>(){
                    Ok(b) => b,
                    Err(_) => return Err("Failed to parse first arg!".to_string()),
                }
            }
        }
    } else if tokens[1].starts_with(':') {
        let name = tokens[1].trim_matches(':').to_string();
        b =  match variables.get(&name) {
            Some(value) => *value,
            None => return Err("No matching variable found".to_string()),
        };
    } else {
        b = match get_query(tokens[1], cursor) {
            Some(value) => value,
            None => return Err("Expected arg!".to_string())
        };
    }
    Ok(a == b)
}

pub fn jump_to_matching_bracket(mut line_number: usize, lines: &Vec<String>) -> usize
{
    let mut condition_count = 1;
    while condition_count != 0 && line_number < lines.len() {
        let line = lines[line_number].trim();
        if line.ends_with("[") {
            condition_count = condition_count + 1
        }
        else if line.starts_with("]") {
            condition_count = condition_count - 1;
        }
        line_number = line_number + 1;
    }
    line_number
}

fn get_bool_as_f32(value: &str) -> Option<f32>
{
    println!("in get bool function");
    println!("value: {value}");
    if value == "TRUE" {
        Some(1.0)
    }
    else if value == "FALSE" {
        Some(0.0)
    }
    else {
        None
    }
}