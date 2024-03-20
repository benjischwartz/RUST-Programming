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

    // Work line by line, parsing then executing program
    let file = match File::open(&file_path) {
        Ok(f) => f,
        Err(err) => {
            eprintln!("Error opening file: {err}");
            return Err(1);
        }
    };

    let reader = BufReader::new(file);

    let mut lines = reader.lines().map(|l| l.unwrap()).collect::<Vec<_>>();
    let mut line_number = 0;
    while line_number < lines.len()
    {
        let line = lines[line_number].trim();
        println!("Processing line: {line}");
        if line.starts_with("IF EQ") {
            match utils::check_equality(line.strip_prefix("IF EQ").unwrap(), &mut cursor, &mut variables) {
                Ok(result) => {
                    if result {
                        println!("CONDITION IS TRUE");
                        line_number = line_number + 1;
                        continue;
                    }
                    else {
                        println!("CONDITION IS FALSE");
                        line_number = utils::jump_to_matching_bracket(line_number + 1, &lines);
                        continue
                    }
                }
                Err(err) => {
                    eprintln!("{err}");
                    return Err(1)
                }
            }
            todo!()
        }
        else if line.starts_with("WHILE") {
            todo!()
        }
        else {
            if !line.starts_with(']') {
                match utils::handle_line(line, &mut image, &mut cursor, &mut variables) {
                    Ok(_) => {},
                    Err(err) => {
                        eprintln!("{err}");
                        return Err(1)
                    }
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