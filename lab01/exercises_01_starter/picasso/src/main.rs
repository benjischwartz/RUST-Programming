use bmp::{Image, Pixel};
use core::fmt;

fn main() {
    for argument in std::env::args().skip(1) {
        println!("===== {argument} =====");
        //let img = bmp::open(argument).unwrap_or_else(|e| {
        //    panic!("Error! BmpError {{ kind: {:?}, details: \"{}\" }}", e.kind, e.details);
        //});
        let img_result = bmp::open(argument);
        let mut img = match img_result {
            Ok(img) => img,
            Err(e) => {
                println!("Error! BmpError {{ kind: {:?}, details: \"{}\" }}", e.kind, e.details);
                continue;
            }
        };

       print_img(img);
    }
}

fn print_img(img: Image) {
    let width = img.get_width();
    let height = img.get_height();
    for y in 0..height {
        for x in 0..width {
            let pix = img.get_pixel(x, y);
            match pix {
                bmp::consts::RED => print!(" R "),
                bmp::consts::BLUE => print!(" B "),
                bmp::consts::LIME => print!(" G "),
                bmp::consts::WHITE => print!(" W "),
                _ => print!("X"),
            }
            if x == width - 1 {
                println!();
            }
        }
    }
}