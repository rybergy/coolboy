extern crate sdl2;
extern crate rand;

use crate::graphics;

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
    tex_gen: TextureCreator<WindowContext>,
}

impl Screen {
    pub fn new(sdl: &Sdl) -> Result<Self, String>{
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
            tex_gen: texture_creator,
        };

        Ok(screen)
    }

    pub fn update_buffer(&self, emu: &Emulator) {
        
    }

    pub fn make_texture(&self) -> Result<Texture, String> {
        let mut texture = self.tex_gen.create_texture_streaming(
            PixelFormatEnum::RGB24,
            graphics::WIDTH * graphics::PIXEL_SIZE,
            graphics::HEIGHT * graphics::PIXEL_SIZE)
            .map_err(|e| e.to_string())?;

        Ok(texture)
    }

    pub fn draw(&mut self) {
        println!("Drawing");
        self.canvas.clear();
        self.canvas.present();
    }
}
