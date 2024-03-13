use unsvg::{Color, get_end_coordinates};

#[derive(Debug)]
pub enum Procedure {
    PENUP,
    PENDOWN,
    FORWARD(f32),
    BACK(f32),
    LEFT(f32),
    RIGHT(f32),
    SETPENCOLOR(f32),
    TURN(f32),
    SETHEADING(f32),
    SETX(f32),
    SETY(f32),
}

pub struct Token {
    procedure: Option<Procedure>,
    variable: Option<String>,
    value: Option<String>,
}

pub struct Cursor {
    pub pen_status: PenStatus,
    pub pen_color: Color,
    pub x_coord: f32,
    pub y_coord: f32,
    pub direction: i32,
}

impl Cursor {
    pub fn new(x: f32, y: f32) -> Cursor {
        Cursor{
            pen_status: PenStatus::PENUP,
            pen_color: Color::white(),
            x_coord: x,
            y_coord: y,
            direction: 0,
        }
    }
    pub fn penup(&mut self) {
        self.pen_status = PenStatus::PENUP
    }

    pub fn pendown(&mut self) {
        self.pen_status = PenStatus::PENDOWN
    }

    pub fn isdown(&self) -> bool {
        return self.pen_status == PenStatus::PENDOWN
    }

    pub fn moveforward(&mut self, value: f32) {
       let temp = get_end_coordinates(self.x_coord, self.y_coord, self.direction, value);
    }

    pub fn moveback(&mut self, value: f32) {
        self.y_coord += value;
    }

    pub fn moveleft(&mut self, value: f32) {
        self.x_coord -= value;
    }

    pub fn moveright(&mut self, value: f32) {
        self.x_coord += value;
    }
}

#[derive(PartialEq)]
pub enum PenStatus {
    PENUP,
    PENDOWN
}