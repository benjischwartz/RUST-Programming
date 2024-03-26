use std::collections::HashMap;
use std::hash::Hash;
use unsvg::{Image, get_end_coordinates, COLORS};
use crate::structs::{Cursor, Procedure, Operator};

pub fn handle_line(line: &str, image: &mut Image, cursor: &mut Cursor, variables: &mut HashMap<String, f32>) -> Result<usize, String>
{
    if line.starts_with("//") {
        println!("Skipping... comment");
        return Ok(0);
    }

    // TOKEN PROCESSING
    let tokens: Vec<& str> = line.split_whitespace().collect();
    let mut iter = tokens.iter().peekable();
    let mut advance_by = 0usize;
    while let Some(token) = iter.nth(advance_by) {
        println!("Processing token {token}");
        match *token {
            "PENUP" => {
                execute_procedure(image, Procedure::PENUP, cursor, variables);
            },
            "PENDOWN" => {
                execute_procedure(image, Procedure::PENDOWN, cursor, variables);
            },
            "FORWARD" | "BACK" | "LEFT" | "RIGHT" | "SETPENCOLOR" |
            "TURN" | "SETHEADING" | "SETX" | "SETY" => {
                if let Some(maybe_value) = iter.next() {
                    let value = match get_value(maybe_value, &tokens, cursor, variables) {
                        Ok((value, adv)) => {
                            advance_by = adv;
                            let procedure = parse_procedure(token, None, value)
                                .expect("Should be a valid command");
                            execute_procedure(image, procedure, cursor, variables);
                        },
                        Err(err) => return Err(err),
                    };
                }
                else {
                    return Err("Not enough args!".to_string());
                }
            },
            "MAKE" | "ADDASSIGN" => {
                if let Some(name) = iter.next() {
                    if name.starts_with('"') {
                        if let Ok(name) = name.trim_matches('"').parse::<String>() {
                            if let Some(maybe_value) = iter.next() {
                                let value = match get_value(maybe_value, &tokens, cursor, variables) {
                                    Ok((value, adv)) => {
                                        advance_by = adv;
                                        let procedure = parse_procedure(token, None, value)
                                            .expect("Should be a valid command");
                                        execute_procedure(image, procedure, cursor, variables);
                                    },
                                    Err(err) => return Err(err),
                                };
                            }
                        }
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

pub fn check_condition(line: &str, cursor: &mut Cursor, variables: &mut HashMap<String, f32>) -> Result<bool, String>
{
    let mut tokens: Vec<& str> = line.split_whitespace().collect();

    println!("Checking condition!");

    // Remove trailing {
    tokens.pop();

    let operator = parse_operator(tokens[0]).unwrap();
    let operands = get_operands(&tokens, 1usize, cursor, variables).unwrap();
    match compare(operator, (operands.0, operands.1)).unwrap() {
        0.0 => Ok(false),
        1.0 => Ok(true),
        _ => return Err("Failed to evalatue condition!".to_string())
    }
}

pub fn jump_to_matching_bracket(mut line_number: usize, lines: &Vec<String>) -> Result<usize, String>
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
        println!("Condition count: {condition_count}");
        println!("line_number: {line_number}");
        println!("line: {line}");

        line_number = line_number + 1;
    }
    if condition_count > 0 {
        return Err("No matching bracket found!".to_string());
    }
    Ok(line_number)
}

fn parse_procedure(token: &str, name: Option<String>, value: f32) -> Option<Procedure>
{
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

fn parse_operator(token: &str) -> Option<Operator>
{
    match token {
        "EQ" => Some(Operator::EQ),
        "NE" => Some(Operator::NE),
        "GT" => Some(Operator::GT),
        "LT" => Some(Operator::LT),
        "AND" => Some(Operator::AND),
        "OR" => Some(Operator::OR),
        _ => None,
    }

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

fn move_cursor(image: &mut Image, cursor: &mut Cursor, direction: i32, length: f32)
{
    println!("In move cursor, cursor down {}", cursor.isdown());
    if cursor.isdown() {
        println!("Drawing!");
        image.draw_simple_line(cursor.x_coord, cursor.y_coord, direction, length, cursor.pen_color);
    }
    let coords = get_end_coordinates(cursor.x_coord, cursor.y_coord, direction, length);
    cursor.x_coord = coords.0;
    cursor.y_coord = coords.1;
}

fn get_value(token: &str, tokens: &Vec<&str>, cursor: &mut Cursor, variables: &mut HashMap<String, f32>) -> Result<(f32, usize), String>
{
    // VALUE CASE
    if token.starts_with('"')
    {
        let trimmed_token = token.trim_matches('"');
        match get_bool_as_f32(trimmed_token) {
            Some(bool) => Ok((bool, 0usize)),
            None => {
                match trimmed_token.parse::<f32>(){
                    Ok(value) => Ok((value, 0usize)),
                    Err(_) => return Err("Invalid value!".to_string()),
                }
            }
        }
    }

    // VARIABLE CASE
    else if token.starts_with(':')
    {
        let name = token.trim_matches(':').to_string();
        match variables.get(&name) {
            Some(value) => Ok((*value, 0usize)),
            None => return Err("No matching variable found".to_string()),
        }
    }

    //  PREFIX OR QUERY CASE
    else
    {
        match token {
            "+" | "-" | "*" | "/" => {
                let position = tokens.iter().position(|&x| x == token).unwrap();
                let res = process_prefix(&tokens, position, cursor, variables).unwrap();
                Ok((res.0, res.1))
            }
            _ => {
                match get_query(token, cursor) {
                    Some(value) => Ok((value, 0usize)),
                    None => return Err("Value not found!".to_string())
                }
            }
        }
    }
}

fn get_query(query: &str, cursor: &mut Cursor) -> Option<f32>
{
    match query {
        "XCOR" => Some(cursor.x_coord),
        "YCOR" => Some(cursor.y_coord),
        "HEADING" => Some(cursor.direction as f32),
        "COLOR" => Some(cursor.color_as_f32()),
        _ => None,
    }
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

fn process_prefix(tokens: & Vec<&str>, position: usize, cursor: &mut Cursor, variables: &mut HashMap<String, f32>) -> Result<(f32, usize), String>
{
    println!("In prefix!");
    let mut num_ops = 0usize;
    let mut cur = position;
    while cur < tokens.len() {
        match tokens[cur] {
            "+" | "-" | "*" | "/" => {
                num_ops += 1;
                cur += 1;
            }
            _ => {
                break;
            }
        }
    }

    println!("Num ops: {num_ops}");

    let mut stack = Vec::new();
    let end_pos = cur + num_ops;
    let advance_by = 2 * num_ops;
    // PUSH VALUES ONTO STACK IN REVERSE
    for i in 0..(num_ops + 1) {
        let value = get_value(tokens[end_pos - i], &tokens, cursor, variables).unwrap().0;
        stack.push(value);
        println!("pushing {:?} to stack", value)
    }

    // APPLY OPERATORS TO STACK VALUES
    for i in 1..=num_ops {
        let op1 = stack.pop().unwrap();
        let op2 = stack.pop().unwrap();
        let token = tokens[cur - i];
        println!("token is: {token}");
        let res = match tokens[cur - i] {
            "+" => op1 + op2,
            "-" => op1 - op2,
            "*" => op1 * op2,
            "/" => {
                if op2 == 0.0 { return Err("Division by zero!".to_string()) }
                op1 / op2
            },
            _ => return Err("Invalid token!".to_string())
        };
        stack.push(res);
    }

    if stack.len() > 1 { return Err("Invalid prefix expression!".to_string()) }

    Ok((stack.pop().unwrap(), advance_by))
}

fn compare(operator: Operator, operands: (f32, f32)) -> Result<f32, String>
{
    println!("Comparing! Operator is {:?}", operator);
    let mut res;
    match operator {
        Operator::EQ => {
            res = (operands.0 == operands.1) as i32 as f32;
        },
        Operator::NE => {
            res = (operands.0 != operands.1) as i32 as f32;
        }
        Operator::GT => {
            res = (operands.0 > operands.1) as i32 as f32;
        },
        Operator::LT => {
            res = (operands.0 < operands.1) as i32 as f32;
        },
        Operator::AND => {
            res = (operands.0 == 1.0 && operands.1 == 1.0) as i32 as f32;
        },
        Operator::OR => {
            res = (operands.0 == 1.0 || operands.1 == 1.0) as i32 as f32;
        },
        _ => { return Err("Invalid operator!".to_string()) }
    }
    println!("Res is {res}");
    Ok(res)
}

fn get_operands(tokens: & Vec<&str>, position: usize, cursor: &mut Cursor, variables: &mut HashMap<String, f32>) -> Result<(f32, f32, usize), String>
{
    let mut split: usize;
    let end_pos: usize;

    let operand_1 = match tokens[position] {
        "EQ" | "NE" | "GT" | "LT" | "AND" | "OR" => {
            let res = get_operands(&tokens, position + 1, cursor, variables).unwrap();
            split = res.2;
            compare(parse_operator(tokens[position]).unwrap(), (res.0, res.1)).unwrap()
        }
        "+" | "-" | "*" | "/" => {
            let res = process_prefix(&tokens, position, cursor, variables).unwrap();
            split = res.1;
            res.0
        }
        _ => {
            let (res, advance_by) = get_value(tokens[position], &tokens, cursor, variables).unwrap();
            split = position + advance_by + 1;
            res
        }
    };

    let operand_2 = match tokens[split] {
        "EQ" | "NE" | "GT" | "LT" | "AND" | "OR" => {
            let res = get_operands(&tokens, split + 1, cursor, variables).unwrap();
            end_pos = res.2;
            compare(parse_operator(tokens[split]).unwrap(), (res.0, res.1)).unwrap()
        }
        "+" | "-" | "*" | "/" => {
            let res = process_prefix(&tokens, split, cursor, variables).unwrap();
            end_pos = res.1;
            res.0
        }
        _ => {
            let (res, advance_by) = get_value(tokens[split], &tokens, cursor, variables).unwrap();
            end_pos = split + advance_by + 1;
            res
        }
    };
    Ok((operand_1, operand_2, end_pos))
}