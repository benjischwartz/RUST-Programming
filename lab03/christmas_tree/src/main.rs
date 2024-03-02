use serde::Deserialize;
use std::collections::VecDeque;
use std::io;

#[derive(Debug, Deserialize, PartialEq)]
enum Instruction {
    Set(i32),
    Left,
    Right,
    Reset,
}

#[derive(Debug)]
struct Tree {
    root: Box<Light>,
    total_lights: i32,
    total_brightness: i32,
}

impl Tree {
    fn new() -> Self {
        Tree {
            root: Box::new(Light::new(0)),
            total_lights: 1,
            total_brightness: 0,
        }
    }
}

#[derive(Debug)]
struct Light{
    left: Option<Box<Light>>,
    right: Option<Box<Light>>,
    brightness: i32,
}
impl Light {
    fn new(brightness: i32) -> Self {
        Light {
            left: None,
            right: None,
            brightness,
        }
    }
    fn set(&mut self, brightness: i32) {
        self.brightness = brightness;
    }

    fn create_left(&mut self, brightness: i32) {
        self.left = Light::new(0).into()
    }
    fn create_right(&mut self, brightness: i32) {
        self.right = Light::new(0).into()
    }
}

impl From<Light> for Option<Box<Light>> {
    fn from(value: Light) -> Self {
        Some(Box::new(value))
    }
}

fn get_instructions_from_stdin() -> VecDeque<Instruction> {
    let mut instructions = String::new();
    io::stdin().read_line(&mut instructions).unwrap();
    ron::from_str(&instructions).unwrap()
}

fn main() {
    let instructions = get_instructions_from_stdin();
    let mut tree = Tree::new();

    println!("{instructions:?}");
    println!("{tree:?}");

    let mut curr = tree.root.as_mut();
    for instruction in instructions.into_iter() {
        match instruction {
            Instruction::Set(value) => {
                tree.total_brightness += (value - curr.brightness);
                curr.set(value)
            }
            Instruction::Left => {
                tree.total_lights += 1;
                curr.create_left(0);
                curr = curr.left.as_mut().unwrap()
            }
            Instruction::Right => {
                tree.total_lights += 1;
                curr.create_right(0);
                curr = curr.right.as_mut().unwrap()
            }
            Instruction::Reset => {
                curr = tree.root.as_mut()
            }
        }
    }
    println!("{:?}", tree.total_brightness / tree.total_lights);
}
