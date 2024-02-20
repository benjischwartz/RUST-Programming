use bmp::{Image, Pixel};

fn main() {
    let path = std::env::args().nth(1).expect("You must provide a path.");
    let operation = std::env::args().nth(2).expect("You must provide an operation.");

    if operation.as_str() == "pixel" {
        draw_pixel(path.as_str());
    } else if operation.as_str() == "diagonal" {
        draw_diag(path.as_str());
    } else if operation.as_str() == "x" {
        draw_x(path.as_str());
    } else if operation.as_str() == "triangle" {
        draw_triangle(path.as_str());
    } else if operation.as_str() == "aboriginal" {
        draw_aboriginal(path.as_str());
    }else {
        eprintln!("The operation {operation} was not recognised!");
    }
}

fn draw_pixel(path: &str) {
    let mut image = Image::new(100, 100);
    image.set_pixel(50, 50, Pixel::new(255, 255, 255));
    image.save(path).expect("This should save correctly.");
}

fn draw_diag(path: &str) {
    let mut image = Image::new(100, 100);
    for x in 1..100 {
        for y in 1..100 {
            if x == y {
                image.set_pixel(x, y, Pixel::new(255, 255, 255));
            }
        }
    }
    image.save(path).expect("This should save correctly.");
}

fn draw_x(path: &str) {
    let mut image = Image::new(100, 100);
    for x in 1..100 {
        for y in 1..100 {
            if (100 - x) == y || x == y {
                image.set_pixel(x, y, Pixel::new(255, 255, 255));
            }
        }
    }
    image.save(path).expect("This should save correctly.");
}

fn draw_triangle(path: &str) {
    let mut image = Image::new(100, 100);
    for x in 20..100 {
        for y in 20..80 {
            if x < y {
                image.set_pixel(x, y, Pixel::new(255, 255, 255));
            }
        }
    }
    image.save(path).expect("This should save correctly.");
}

fn draw_aboriginal(path: &str) {
    let mut image = Image::new(200, 100);
    for x in 0..200 {
        for y in 0..50 {
            image.set_pixel(x, y, Pixel::new(0, 0, 0));
        }
    }
    for x in 0..200 {
        for y in 51..100 {
            image.set_pixel(x, y, Pixel::new(255, 0, 0));
        }
    }
    let centre = (100, 50);
    for x in 0..200 {
        for y in 0..100 {
            if within_radius(20, centre, (x, y)) {
                image.set_pixel(x, y, Pixel::new(255, 255, 0));
            }
        }
    }
    image.save(path).expect("This should save correctly.");
}

fn within_radius(radius: u32, centre: (u32, u32), coords: (u32, u32)) -> bool {
    let x_dist = centre.0 as i64 - coords.0 as i64;
    let y_dist = centre.1 as i64 - coords.1 as i64;
    let distance_squared = x_dist.pow(2) as f64 + y_dist.pow(2) as f64;
    let distance = distance_squared.sqrt() as f64;
    if distance <= radius as f64 {
        return true;
    }
    return false;
}
















