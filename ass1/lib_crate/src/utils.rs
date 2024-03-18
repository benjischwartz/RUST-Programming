use std::collections::HashMap;
use std::fmt::Error;
use std::fs::File;
use std::hash::Hash;
use std::io::BufReader;
use unsvg::{Image, Color, get_end_coordinates, COLORS};
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
                        if let Ok(value) = value.trim_matches('"').parse::<f32>() {
                            let procedure = parse_procedure(token, None, value).expect("Should be a valid command");
                            execute_procedure(image, procedure, cursor, variables);
                        }
                        else {
                            return Err("Couldn't parse arg!".to_string());
                        }
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
                                return Err("No matching variable found".to_string());
                            }
                        }
                    }
                    // TODO: Check if need to add Query case
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
                else { return Err("Not enough args!".to_string())};
            },
            "MAKE" | "ADDASSIGN" => {
                if let Some(name) = iter.next() {
                    if name.starts_with('"') {
                        if let Ok(name) = name.trim_matches('"').parse::<String>() {
                            if let Some(value) = iter.next() {
                                if value.starts_with('"') {
                                    if let Ok(value) = value.trim_matches('"') .parse::<f32>() {
                                        let procedure = parse_procedure(token, Some(name), value)
                                            .expect("Should be a valid command");
                                        execute_procedure(image, procedure, cursor, variables);
                                    }
                                    else {
                                        return Err("Couldn't parse second arg!".to_string());
                                    }
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