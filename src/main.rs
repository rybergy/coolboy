#[macro_use]
extern crate bitflags;
extern crate sdl2;

mod emulator;
mod graphics;

use emulator::Emulator;
use std::time::{Duration, SystemTime};

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;

fn main() -> Result<(), String> {
    let sdl = sdl2::init()?;
    let mut screen = graphics::Screen::new(&sdl).map_err(|e| e.to_string())?;

    // Can we move texture to the Screen struct?
    /*let mut texture = texture_creator.create_texture_streaming(
        PixelFormatEnum::RGB24, 
        graphics::WIDTH * graphics::PIXEL_SIZE, 
        graphics::HEIGHT * graphics::PIXEL_SIZE)
        .map_err(|e| e.to_string())?;*/

    let mut texture = screen.make_texture();
        
    let mut emulator = Emulator::from_file("roms/tetris.gb").map_err(|e| e.to_string())?;

    let timestep = Duration::from_secs(1) / 60;
    let mut event_pump = sdl.event_pump().map_err(|e| e.to_string())?;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    println!("breaking!");
                    break 'running
                },
                _ => {}
            }
        }
        screen.draw();

        ::std::thread::sleep(timestep);
    }

    // loop {
    //     let begin = SystemTime::now();

    //     // Run the current cycle
    //     // emulator.update();
    //     screen.update_buffer(&emulator);
    //     screen.draw();

    //     // Spin wait until 1/60 of a second has passed
    //     while SystemTime::now() < begin + timestep {}
    // }

    Ok(())
}
