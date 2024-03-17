use std::collections::HashMap;
use unsvg::Image;
use std::fs::File;
use std::hash::Hash;
use clap::Parser;
use lib_crate::utils;
use std::io::{BufRead, BufReader};
use lib_crate::structs::{Cursor, PenStatus};

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

    for line_result in reader.lines() {
        let line = match line_result {
            Ok(l) => l,
            Err(err) => {
                eprintln!("Error reading line: {err}");
                return Err(1);
            }
        };
        match utils::handle_line(&line, &mut image, &mut cursor, &mut variables) {
            Ok(_) => {},
            Err(err) => {
                eprintln!("{err}");
                return Err(1)
            }
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