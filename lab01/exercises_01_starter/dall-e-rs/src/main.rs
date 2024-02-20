use bmp::Pixel;

// Lines.c conversion to Rust
const IMAGE_SIZE_PX: u32 = 256;

fn main() {
    let mut img = bmp::Image::new(IMAGE_SIZE_PX, IMAGE_SIZE_PX);
    let red = create_pixel(255, 0, 0);
    let green = create_pixel(0, 255, 0);
    let blue = create_pixel(0, 0, 255);
    let black = create_pixel(0, 0, 0);

    for x in 0..IMAGE_SIZE_PX {
        for y in 0..IMAGE_SIZE_PX {
            img.set_pixel(x, y, create_pixel(x as u8, y as u8, ((x + y) % 256) as u8));
        }
    }

    let img = draw_circle(30, (IMAGE_SIZE_PX/2, IMAGE_SIZE_PX/2), img, green);
    let img = draw_circle(7, (IMAGE_SIZE_PX/2 + 10, IMAGE_SIZE_PX/2 - 10), img, black);
    let img = draw_circle(7, (IMAGE_SIZE_PX/2 - 10, IMAGE_SIZE_PX/2 - 10), img, black);
    let img = draw_circle(4, (IMAGE_SIZE_PX/2, IMAGE_SIZE_PX/2 + 10), img, black);

    img.save("my_image.bmp")
        .expect("Failed to save image!");
}

fn create_pixel(r: u8, g: u8, b: u8) -> Pixel {
    return Pixel::new(r, g, b);
}

fn draw_circle(radius: u32, centre: (u32, u32), mut img: bmp::Image, colour: Pixel) -> bmp::Image {

    for x in 0..IMAGE_SIZE_PX {
        for y in 0..IMAGE_SIZE_PX {
            if within_radius(radius, centre, (x, y)) {
                img.set_pixel(x, y, colour);
            }
        }
    }
    img
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