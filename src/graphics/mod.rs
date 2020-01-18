extern crate sdl2;
extern crate rand;

use super::emulator::Emulator;

use rand::prelude::*;
use sdl2::Sdl;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};
use sdl2::pixels::{Color, PixelFormatEnum};

pub const PIXEL_SIZE: u32 = 5;
pub const WIDTH: u32 = 160;
pub const HEIGHT: u32 = 144;

pub struct Screen {
    canvas: Canvas<Window>,
    buffer: Vec<u32>,
    rng: ThreadRng, // TEMP: random pixels
}

impl Screen {
    pub fn new(sdl: &Sdl) -> Result<(Self, TextureCreator<WindowContext>), String>{
        let video = sdl.video()?;
        let window = video.window("coolboy v1.0", WIDTH * PIXEL_SIZE, HEIGHT * PIXEL_SIZE)
            .position_centered()
            .opengl()
            .build()
            .map_err(|e| e.to_string())?;

        let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();

        let texture_creator = canvas.texture_creator();

        let screen = Screen { 
            canvas: canvas,
            buffer: vec![0; 160 * 144],
            rng: rand::thread_rng(),
        };

        Ok((screen, texture_creator))
    }

    pub fn update_buffer(&self, emu: &Emulator) {
        
    }

    pub fn draw(&mut self, texture: &Texture) {
        info!("Drawing to screen");
        self.canvas.clear();
        self.canvas.present();
    }
}