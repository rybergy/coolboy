#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate log;
extern crate sdl2;
extern crate chrono;

#[macro_use]
pub mod macros {
    macro_rules! tbit {
        ($value:expr, $bit:expr) => (($value & (1 << $bit)) != 0)
    }

    macro_rules! sbit {
        ($value:expr, $bit:expr) => ($value | (1 << $bit))
    }

    macro_rules! ubit {
        ($value:expr, $bit:expr) => ($value & !(1 << $bit))
    }
    
    macro_rules! gbit {
        ($value:expr, $bit:expr) => (($value & (1 << $bit)) >> $bit)
    }
}

mod emulator;
mod graphics;
mod logging;

use emulator::{Emulator, Inputs};

use sdl2::pixels::PixelFormatEnum;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use std::time::Duration;

fn main() -> Result<(), String> {
    println!("here");
    logging::init().map_err(|e| e.to_string())?;
    
    let sdl = sdl2::init()?;

    let (mut screen, texture_creator) = graphics::Screen::new(&sdl).map_err(|e| e.to_string())?;
    
    let mut texture = texture_creator.create_texture_streaming(
        PixelFormatEnum::RGB24, 
        graphics::WIDTH * graphics::PIXEL_SIZE, 
        graphics::HEIGHT * graphics::PIXEL_SIZE)
        .map_err(|e| e.to_string())?;

    
    let mut emulator = Emulator::from_file("roms/tetris.gb").map_err(|e| e.to_string())?;
    // info!("fuck");
    let timestep = Duration::from_secs(1) / 60;
    let mut event_pump = sdl.event_pump().map_err(|e| e.to_string())?;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => break 'running,
                Event::KeyDown { keycode: Some(keycode), .. } => {
                    match keycode {
                        Keycode::W => emulator.input_down(Inputs::UP),
                        Keycode::A => emulator.input_down(Inputs::LEFT),
                        Keycode::S => emulator.input_down(Inputs::DOWN),
                        Keycode::D => emulator.input_down(Inputs::RIGHT),
                        Keycode::M => emulator.input_down(Inputs::A),
                        Keycode::N => emulator.input_down(Inputs::B),
                        Keycode::Return => emulator.input_down(Inputs::SELECT),
                        Keycode::Escape => emulator.input_down(Inputs::START),
                        _ => ()
                    }
                },
                Event::KeyUp { keycode: Some(keycode), .. } => {
                    match keycode {
                        Keycode::W => emulator.input_up(Inputs::UP),
                        Keycode::A => emulator.input_up(Inputs::LEFT),
                        Keycode::S => emulator.input_up(Inputs::DOWN),
                        Keycode::D => emulator.input_up(Inputs::RIGHT),
                        Keycode::M => emulator.input_up(Inputs::A),
                        Keycode::N => emulator.input_up(Inputs::B),
                        Keycode::Return => emulator.input_up(Inputs::SELECT),
                        Keycode::Escape => emulator.input_up(Inputs::START),
                        _ => ()
                    }
                },
                _ => ()
            }
        }
        screen.draw(&texture);

        ::std::thread::sleep(timestep);
    };

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

#[cfg(test)]
mod test {

    #[test]
    fn test_tbit() {
        assert!(tbit!(0b100, 2));
    }

    #[test]
    fn test_sbit() {
        assert_eq!(sbit!(0, 2), 0b100);
    }

    #[test]
    fn test_ubit() {
        assert_eq!(ubit!(0b100, 2), 0);
    }

    #[test]
    fn test_gbit() {
        assert_eq!(gbit!(0b100, 2), 1);
    }
}