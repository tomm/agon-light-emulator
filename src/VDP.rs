
use std::sync::mpsc::{Sender, Receiver};

use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::Canvas;
use sdl2::video::Window;
mod font;
use font::font::FONT_BYTES;
struct Cursor {
    position_x: i32,
    position_y: i32,
    screen_width: i32,
    screen_height: i32,
    font_width: i32,
    font_height: i32
}

impl Cursor {
    fn new(screen_width: i32 , screen_height: i32, font_width: i32, font_height: i32) -> Cursor {
        Cursor {
            position_x: 0,
            position_y: 0,
            screen_width: screen_width,
            screen_height: screen_height,
            font_width: font_width,
            font_height: font_height
        }
    }

    fn home(&mut self) {
        self.position_x = 0;
    }

    fn down(&mut self) {
        self.position_y += self.font_height;
    }

    fn up(&mut self) {
        self.position_y -= self.font_height;
        if self.position_y < 0 {
            self.position_y = 0;
        }
    }

    fn left(&mut self) {
        self.position_x -= self.font_width;
        if self.position_x < 0 {
            self.position_x = 0;
        }
    }

    fn right(&mut self) {
        self.position_x += self.font_width;
        if self.position_x >= self.screen_width {
            self.home();
            self.down();
        }
    }
}

pub struct VDP {
    cursor: Cursor,
    canvas: Canvas<Window>,
    tx: Sender<u8>,
    rx: Receiver<u8>
}

impl VDP {
    pub fn new(canvas: Canvas<Window>, tx : Sender<u8>, rx : Receiver<u8>) -> VDP {
        VDP {
            cursor: Cursor::new(canvas.window().drawable_size().0 as i32, canvas.window().drawable_size().1 as i32, 8, 8),
            canvas: canvas,
            tx: tx,
            rx: rx
        }
    }

    fn get_points(bytes : Vec<u8>) -> Vec<Point>
    {
        let mut points: Vec<Point> = Vec::new();
        let mut y = 0;
        for byte in bytes.iter()
        {
            for bit in 0..7
            {
                if byte & (1 << bit) != 0
                {
                    points.push(Point::new(7 - bit, y));
                }
            }
            y = y + 1;
        }
        points
    }
    
    fn render_char(&mut self, ascii : u8)
    {
        if ascii >= 32
        {
            let shifted_ascii = ascii - 32;
            if shifted_ascii < (FONT_BYTES.len() / 8) as u8
            {
                let start = 8*shifted_ascii as usize;
                let end = start+8 as usize;
                let points = Self::get_points(FONT_BYTES[start..end].to_vec());
                self.canvas.set_draw_color(Color::RGB(255, 255, 255));
                self.canvas.set_viewport(Rect::new(self.cursor.position_x as i32, self.cursor.position_y as i32, 8, 8));
                self.canvas.draw_points(&points[..]);
                self.canvas.present();
            }
        }
    }
    
    pub fn cls(&mut self) {
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();
        self.cursor.position_x = 0;
        self.cursor.position_y = 0;
    }

    pub fn run(&mut self) {
        match self.rx.try_recv() {
            Ok(n) => {
                match n {
                    n if n >= 0x20 && n != 0x7F => {
                        println!("Received character: {}", n as char);
                        self.render_char(n);
                        self.cursor.right();  
                    },
                    0x08 => {println!("Cursor left."); self.cursor.left();},
                    0x09 => {println!("Cursor right."); self.cursor.right();},
                    0x0A => {println!("Cursor down."); self.cursor.down();},
                    0x0B => {println!("Cursor up."); self.cursor.up();},
                    0x0C => {
                        println!("CLS.");
                        self.cls();
                    },
                    0x0D => {println!("Cursor home."); self.cursor.home();},
                    0x0E => {println!("PageMode ON?");},
                    0x0F => {println!("PageMode OFF?");},
                    0x10 => {println!("CLG?");},
                    0x11 => {println!("COLOUR?");},
                    0x12 => {println!("GCOL?");},
                    0x13 => {println!("Define Logical Colour?");},
                    0x16 => {println!("MODE?");},
                    0x17 => {
                        println!("VDU23.");
                        match self.rx.recv().unwrap() {
                            0x00 => {
                                println!("Video System Control.");
                                match self.rx.recv().unwrap() {
                                    0x80 => println!("VDP_GP"),
                                    0x81 => println!("VDP_KEYCODE"),
                                    n => println!("Unknown VSC command: {:#02X?}.", n),
                                }
                            },
                            0x01 => println!("Cursor Control?"),
                            0x07 => println!("Scroll?"),
                            0x1B => println!("Sprite Control?"),
                            n => println!("Unknown VDU command: {:#02X?}.", n),
                        }
                    },
                    0x19 => {println!("PLOT?");},
                    0x1D => {println!("VDU_29?");},
                    0x1E => {println!("Home."); self.cursor.home();},
                    0x1F => {println!("TAB?");},
                    0x7F => {println!("BACKSPACE?");},
                    n => println!("Unknown Command {:#02X?} received!", n),
                }
            },
            Err(_e) => ()
        }
    }
}
