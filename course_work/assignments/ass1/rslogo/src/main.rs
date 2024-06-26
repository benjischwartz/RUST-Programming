use std::collections::HashMap;
use unsvg::Image;
use std::fs::File;
use std::hash::Hash;
use clap::Parser;
use lib_crate::utils;
use std::io::{BufRead, BufReader};
use lib_crate::structs::{Cursor};

/// A simple program to parse four arguments using clap.

#[derive(Debug, Parser)]
pub struct Args
{
    /// Path to a file
    pub file_path: std::path::PathBuf,
    /// Path to an svg or png image
    pub image_path: std::path::PathBuf,
    /// Height
    pub height: u32,
    /// Width
    pub width: u32,
}

fn main() -> Result<(), i32>
{
    let args: Args = Args::parse();

    // Access the parsed arguments
    let file_path = args.file_path;
    let image_path = args.image_path;
    let height = args.height;
    let width = args.width;

    let mut image = Image::new(width, height);
    let mut cursor = Cursor::new((width / 2) as f32, (height / 2) as f32);
    let mut variables: HashMap<String, f32> = HashMap::new();
    let mut return_map: HashMap<usize, usize> = HashMap::new();

    // Work line by line, parsing then executing program
    let file = match File::open(&file_path) {
        Ok(f) => f,
        Err(err) => {
            eprintln!("Error opening file: {err}");
            return Err(1);
        }
    };

    let reader = BufReader::new(file);

    let lines = reader.lines().map(|l| l.unwrap()).collect::<Vec<_>>();
    let mut line_number = 0;
    while line_number < lines.len()
    {
        let line = lines[line_number].trim();
        println!("Processing line: {line}");
        if line.starts_with("IF") {
            match utils::check_condition(line.strip_prefix("IF ").unwrap(), &mut cursor, &mut variables) {
                Ok((result, adv)) => {
                    if result {
                        println!("CONDITION IS TRUE");
                        line_number = line_number + 1;
                        continue;
                    }
                    else {
                        println!("CONDITION IS FALSE");
                        println!("Start line: {line_number}");
                        line_number = match utils::jump_to_matching_bracket(line_number + 1, &lines) {
                            Ok(line_number) => line_number,
                            Err(err) => {
                                eprintln!("{err}");
                                return Err(1)
                            }
                        };
                        println!("After jumping line: {line_number}");
                        continue
                    }
                }
                Err(err) => {
                    eprintln!("{err}");
                    return Err(1)
                }
            }
        }
        else if line.starts_with("WHILE") {
            match utils::check_condition(line.strip_prefix("WHILE ").unwrap(), &mut cursor, &mut variables) {
                Ok((result, adv)) => {
                    if result {
                        println!("CONDITION IS TRUE");
                        // Add return line number
                        let return_line = match utils::jump_to_matching_bracket(line_number + 1, &lines) {
                            Ok(line) => line - 1,
                            Err(err) => {
                                eprintln!("{err}");
                                return Err(1)
                            }
                        };
                        println!("adding return line {return_line} for statement on line {line_number}");
                        return_map.insert(utils::jump_to_matching_bracket(line_number + 1, &lines).unwrap() - 1, line_number);
                        line_number = line_number + 1;
                        continue;
                    }
                    else {
                        println!("CONDITION IS FALSE");
                        line_number = match utils::jump_to_matching_bracket(line_number + 1, &lines) {
                            Ok(line) => line,
                            Err(err) => {
                                eprintln!("{err}");
                                return Err(1)
                            }
                        };
                        continue
                    }
                }
                Err(err) => {
                    eprintln!("{err}");
                    return Err(1)
                }
            }
        }
        else if line.starts_with(']') {
            // Check if this is the end of a while loop
            if return_map.contains_key(&line_number) {
                line_number = *return_map.get(&line_number).expect("Key exists");
            }
            else {
                line_number = line_number + 1;
            }
        }
        else {
            match utils::handle_line(line, &mut image, &mut cursor, &mut variables) {
                Ok(_) => {},
                Err(err) => {
                    eprintln!("{err}");
                    return Err(1)
                }
            }
            line_number = line_number + 1;
        }
    }


    match image_path.extension().map(|s| s.to_str()).flatten() {
        Some("svg") => {
            let res = image.save_svg(&image_path);
            if let Err(e) = res {
                eprintln!("Error saving svg: {e}");
                return Err(1);
            }
        }
        Some("png") => {
            let res = image.save_png(&image_path);
            if let Err(e) = res {
                eprintln!("Error saving png: {e}");
                return Err(1);
            }
        }
        _ => {
            eprintln!("File extension not supported");
            return Err(1);
        }
    }

    Ok(())
}