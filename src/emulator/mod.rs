mod cartridge;
mod cpu;
mod memory;
mod registers;

use cartridge::Cartridge;
use cpu::Cpu;
use memory::{Memory, RomBankMode};

use std::io;
use std::time::{Duration, SystemTime};

const TIMER_ADDRESS: usize = 0xFF05;
const TIMER_MODULATOR: usize = 0xFF06;
const TIMER_CONTROLLER: usize = 0xFF07;

const DIVIDER_REGISTER: usize = 0xFF04;

const INTERRUPT_REQUEST: usize = 0xFF0F;
const INTERRUPT_ENABLED: usize = 0xFFFF;

const SCANLINE_ADDRESS: usize = 0xFF44;
const LCD_STATUS_ADDRESS: usize = 0xFF41;
const LCD_CONTROL_ADDRESS: usize = 0xFF40;

const LCD_MODE2_BOUND: u16 = 376; // 456 scanlines - 80 cycles
const LCD_MODE3_BOUND: u16 = 204; // ^ 376 - 172 cycles

const DMA_ADDRESS: usize = 0xFF46;

const SCROLL_Y_ADDRESS: usize = 0xFF42;
const SCROLL_X_ADDRESS: usize = 0xFF43;

const WINDOW_Y_ADDRESS: usize = 0xFF4A;
const WINDOW_X_ADDRESS: usize = 0xFF4B;

const PALETTE_47_ADDRESS: usize = 0xFF47;
const PALETTE_48_ADDRESS: usize = 0xFF48;
const PALETTE_49_ADDRESS: usize = 0xFF49;

const SPRITE_ATTRIBUTE_TABLE: usize = 0xFE00;
const SPRITE_DATA_ADDRESS: usize = 0x8000;

const KEY_ADDRESS: usize = 0xFF00;

#[derive(Copy, Clone)]
enum Interrupt {
    VBlank = 0b00000001,
    LCD    = 0b00000010,
    Timer  = 0b00000100,
    Joypad = 0b00010000,
}

enum Color {
    White,
    LightGrey,
    DarkGrey,
    Black
}

impl Color {

    fn rgb(&self) -> (u8, u8, u8) {
        let value = match self {
            Color::White => 0xFF,
            Color::LightGrey => 0xCC,
            Color::DarkGrey => 0x77,
            Color::Black => 0x00
        };
        (value, value, value)
    }

}

bitflags! {
    pub struct Inputs: u8 {
        const RIGHT  = 0b00000001;
        const LEFT   = 0b00000010;
        const UP     = 0b00000100;
        const DOWN   = 0b00001000;
        const A      = 0b00010000;
        const B      = 0b00100000;
        const SELECT = 0b01000000;
        const START  = 0b10000000;
    }
}

const ALL_INPUTS: [Inputs; 8] = [Inputs::RIGHT, Inputs::LEFT, Inputs::UP, Inputs::DOWN, Inputs::A, Inputs::B, Inputs::SELECT, Inputs::START];

impl Inputs {

    fn bit_location(self) -> u8 {
        match self {
            Inputs::RIGHT | Inputs::A => 0,
            Inputs::LEFT | Inputs::B => 1,
            Inputs::UP | Inputs::SELECT => 2,
            Inputs::DOWN | Inputs::START => 3,
            _ => {
                warn!("Attempting to get bit location for non-singular input!");
                7
            }
        }
    }

    fn select_location(self) -> u8 {
        match self {
            Inputs::RIGHT | Inputs::LEFT | Inputs::UP | Inputs::DOWN => 4,
            Inputs::A | Inputs::B | Inputs::SELECT | Inputs::START => 5,
            _ => {
                warn!("Attempting to get selection location for non-singular input!");
                7
            }
        }
    }

}

const ALL_INTERRUPTS: [Interrupt; 4] = [Interrupt::VBlank, Interrupt::LCD, Interrupt::Timer, Interrupt::Joypad];

pub struct Emulator {
    cpu: Cpu,
    memory: Memory,

    timer_counter: i32,
    divider_counter: i32,

    interrupt_master: bool,

    scanline_count: u16,

    screen_buffer: [[[u8; 3]; 144]; 160],

    pressed_inputs: Inputs,
}

impl Emulator {
    pub fn from_file(filename: &str) -> Result<Self, io::Error> {
        Ok(Emulator {
            cpu: Cpu::new(),
            memory: Memory::from_file(filename)?,
            timer_counter: 0,
            divider_counter: 0,
            interrupt_master: true,
            scanline_count: 0,
            screen_buffer: [[[0; 3]; 144]; 160],
            pressed_inputs: Inputs::empty(),
           })
    }

    pub fn update(&mut self) {
        let mut elapsed_cycles = 0;

        // 69905 CPU cycles per frame
        while elapsed_cycles < 69905 {
            let cycles = self.cpu.execute(&mut self.memory);
            elapsed_cycles += cycles;
        }
    }

    fn read_memory(&self, address: usize) -> u8 {
        match address {
            0xFF00 => self.joypad_state(),
            _ => self.memory.read(address)
        }
    }

    fn write_memory(&mut self, address: usize, data: u8) {
        match address {
            TIMER_CONTROLLER => {
                let current_freq = self.get_clock_freq();
                self.memory.write(address, data);
                let new_freq = self.get_clock_freq();

                if current_freq != new_freq {
                    self.set_clock_freq();
                }
            },
            DMA_ADDRESS => {
                self.dma_transfer(data);
            }
            _ => self.memory.write(address, data)
        }
    }

    fn push_stack8(&mut self, data: u8) {

    }

    fn push_stack16(&mut self, data: u16) {

    }

    fn dma_transfer(&mut self, data: u8) {
        let address = (data as usize) << 8;
        for i in 0..0xA0 {
            let data = self.read_memory(address + i);
            self.write_memory(0xFE00 + i, data);
        }
    }

    fn update_timers(&mut self, cycles: u16) {
        self.handle_divider_register(cycles);

        if self.clock_enabled() {
            self.timer_counter -= cycles as i32;

            if self.timer_counter <= 0 {
                self.set_clock_freq();

                match self.read_memory(TIMER_ADDRESS) {
                    0xFF => {
                        self.write_memory(TIMER_ADDRESS, 0xFF);
                        self.request_interrupt(Interrupt::Timer);
                    },
                    value => {
                        self.write_memory(TIMER_ADDRESS, value + 1);
                    }
                }
            }
        }
    }

    fn set_clock_freq(&mut self) {
        self.timer_counter = match self.get_clock_freq() {
            0 => 1024,
            1 => 16,
            2 => 64,
            3 => 256,
            _ => unreachable!()
        };
    }

    fn get_clock_freq(&self) -> u8 {
        self.read_memory(TIMER_CONTROLLER) & 0b00000011
    }

    fn clock_enabled(&self) -> bool {
        (self.read_memory(TIMER_CONTROLLER) & 0b00000100) != 0
    }

    fn handle_divider_register(&mut self, cycles: u16) {
        self.divider_counter += cycles as i32;
        if self.divider_counter >= 255 {
            self.divider_counter = 0;
            let byte = self.read_memory(DIVIDER_REGISTER);
            self.memory.write_force(DIVIDER_REGISTER, byte);
        }
    }

    fn handle_interrupts(&mut self) {
        if self.interrupt_master {
            let req = self.read_memory(INTERRUPT_REQUEST);
            let enabled = self.read_memory(INTERRUPT_ENABLED);

            for interrupt in ALL_INTERRUPTS.iter() {
                // If interrupt request register is set and is enabled
                if ((req | *interrupt as u8) != 0) && ((enabled | *interrupt as u8) != 0) {
                    self.service_interrupt(*interrupt);
                }
            }
        }
    }

    fn request_interrupt(&mut self, interrupt: Interrupt) {
        let req = self.read_memory(INTERRUPT_REQUEST);
        let req_set = req | interrupt as u8;
        self.write_memory(INTERRUPT_REQUEST, req_set);
    }

    fn service_interrupt(&mut self, interrupt: Interrupt) {
        self.interrupt_master = false;

        let req = self.read_memory(INTERRUPT_REQUEST);

        let req_unset = req & !(interrupt as u8);
        self.write_memory(INTERRUPT_REQUEST, req_unset);

        self.push_stack16(self.cpu.pc);

        self.cpu.pc = match interrupt {
            Interrupt::VBlank => 0x40,
            Interrupt::LCD => 0x48,
            Interrupt::Timer => 0x50,
            Interrupt::Joypad => 0x60,
        };
    }

    fn update_graphics(&mut self, cycles: u16) {
        self.set_lcd_status();

        if self.lcd_enabled() {
            self.scanline_count -= cycles;

            if self.scanline_count <= 0 {
                let old_line = self.read_memory(SCANLINE_ADDRESS);

                let new_line = old_line + 1;
                self.memory.write_force(SCANLINE_ADDRESS, new_line);

                self.scanline_count = 456;

                if new_line == 144 {
                    self.request_interrupt(Interrupt::VBlank);
                } else if new_line > 153 {
                    self.memory.write_force(SCANLINE_ADDRESS, 0);
                } else if new_line < 144 {
                    self.draw_scanline();
                }
            }
        }
    }

    fn set_lcd_status(&mut self) {
        let status = self.read_memory(LCD_STATUS_ADDRESS);

        if !self.lcd_enabled() {
            self.scanline_count = 456;
            self.memory.write_force(SCANLINE_ADDRESS, 0);
            let masked_status = (status & 0b11111100) | 0b00000001;
            self.write_memory(LCD_STATUS_ADDRESS, masked_status);
        } else {
            let current_line = self.read_memory(SCANLINE_ADDRESS);
            let current_mode = status & 0b00000011;

            let mode = 
                if current_line >= 144 {
                    1
                } else {
                    match self.scanline_count {
                        LCD_MODE2_BOUND..=std::u16::MAX => 2,
                        LCD_MODE3_BOUND..=LCD_MODE2_BOUND => 3,
                        _ => 0
                    }
                };

            let masked_status = status & 0b11111100 | mode;
            // Mode 0 sets bit 3, 1 sets bit 4, 2 sets bit 5
            // So just set mode + 3 bits from the right
            let req_int = match mode {
                0 | 1 | 2 => (status & (1 << (3 + mode))) != 0,
                _ => false
            };

            if req_int && (mode != current_mode) {
                self.request_interrupt(Interrupt::LCD);
            }

            let game_scanline = self.read_memory(0xFF45);

            let cncd_status = 
                if current_line == game_scanline {
                    let new_status = status | 0b00000100;
                    if (new_status & 0b01000000) != 0 {
                        self.request_interrupt(Interrupt::LCD);
                    }
                    new_status
                } else {
                    status & 0b11111011
                };

            self.write_memory(LCD_STATUS_ADDRESS, cncd_status);
        }
    }

    fn lcd_enabled(&mut self) -> bool {
        let byte = self.read_memory(LCD_CONTROL_ADDRESS);
        (byte & 0b10000000) != 0 
    }

    fn draw_scanline(&mut self) {
        let control = self.read_memory(LCD_CONTROL_ADDRESS);

        if tbit!(control, 0) {
            self.render_tiles();
        }
        if tbit!(control, 1) {
            self.render_sprites();
        }
    }

    fn render_tiles(&mut self) {
        // Each tile is 8x8 pixels
        // Resolution has 256x256 real pixels (32x32 tiles)
        // 160x144 viewing space 

        // 2*: Sprite layer
        // 1*: Window layer
        // 0: Background layer (256x256) (32x32 tiles)
        // * Sprite and window..

        // Position of background to start drawing viewing area
        let scroll_y = self.read_memory(SCROLL_Y_ADDRESS);
        let scroll_x = self.read_memory(SCROLL_X_ADDRESS);
        // Position of viewing area to start drawing window
        let window_y = self.read_memory(WINDOW_Y_ADDRESS);
        let window_x = self.read_memory(WINDOW_X_ADDRESS);

        let scanline = self.read_memory(SCANLINE_ADDRESS);
        let lcd_control = self.read_memory(LCD_CONTROL_ADDRESS);

        // Bit 5 - whether or not game is drawing the window layer
        let using_window = ((lcd_control & 0b00100000) != 0) 
                       && (window_y <= scanline);
        
        // Bit 4 - which tile data bank to use
        //   If using 0x8800, signed integers; else 0x8000
        let (tile_data, signed) = match tbit!(lcd_control, 4) {
            true => (0x8000, false),
            false => (0x8800, true)
        };

        // Which background memory to use
        let background_memory = 
            if (using_window && tbit!(lcd_control, 3)) || (!using_window && tbit!(lcd_control, 6)) {
                0x9C00
            } else {
                0x9800
            };

        // Get the current y position of the scanline we're drawing
        let pos_y = 
            if using_window {
                scanline - window_y
            } else {
                scroll_y + scanline
            };

        // Current row we're drawing - 32 tiles in each row
        let tile_row = (pos_y as u16 / 8) * 32;

        // Start drawing all the pixels on the screen
        for pixel in 0..160 {
            // The position of 
            let pos_x = 
                if using_window && pixel >= window_x {
                    pixel - window_x
                } else {
                    pixel + scroll_x
                };

            // Current column we're drawing
            let tile_column = pos_x / 8;
            
            // Find number identifier of the tile we want to draw
            let tile_address: usize = (background_memory + tile_row + tile_column as u16) as usize;
            let tile_num: i16 = 
                if signed {
                    // Signed: interpret as i8 and convert to 
                    i16::from(self.read_memory(tile_address) as i8)
                } else {
                    i16::from(self.read_memory(tile_address) as u8)
                };

            // Find tile in memory
            let tile_location = 
                if signed {
                    (tile_num + 128) * 16
                } else {
                    tile_num * 16
                };

            // Get which of 8 vertical lines we're drawing
            // Remember each tile is 2 bytes
            let line_offset = (pos_y % 8) * 2;
            let data1 = self.read_memory((tile_location + i16::from(line_offset)) as usize);
            let data2 = self.read_memory((tile_location + i16::from(line_offset)) as usize + 1);

            // Data1 : 7 6 5 4 3 2 1 0
            // Data2 : 7 6 5 4 3 2 1 0
            // X position indexes the bit position
            // Data 2 is bit 1 of the color ID, data 1 is bit 0
            // BUT pixel 1 is in bit 7, pixel 2 in bit 6, etc. so we need to invert
            let color_bit = -((pos_x % 8) as i16 - 7);
            let color_num = (gbit!(data2, color_bit) << 1) | gbit!(data1, color_bit);

            let color = self.get_color(color_num, PALETTE_47_ADDRESS);
            let (red, green, blue) = color.rgb();

            if scanline < 0 || scanline > 143 {
                warn!("Attempting to write scanline {} which is out of bounds!", scanline);
            } else {
                self.screen_buffer[pixel as usize][scanline as usize][0] = red;
                self.screen_buffer[pixel as usize][scanline as usize][1] = green;
                self.screen_buffer[pixel as usize][scanline as usize][2] = blue;
            }
        }
    }

    fn render_sprites(&mut self) {
        // Sprite data from 0x8000 to 0x8FFF

        // Sprite attribute table from 0xFE00 to 0xFE9F
        // This holds 40 sprites of 4 bytes each
        // Byte 0: y position (minus 16)
        // Byte 1: x position (minus 8)
        // Byte 2: pattern number - used to look up in  0x8000
        // Byte 3: attributes - 
        //   Bit 7: sprite / background priority; 0 is sprite above, 1 is sprite behind unless BG is white
        //   Bit 6: y flip
        //   Bit 5: x flip
        //   Bit 4: palette number; 0 is 0xFF48, 1 is 0xFF49
        //   Bits 3-0: unused

        let lcd_control = self.read_memory(LCD_CONTROL_ADDRESS);
        let double_height = tbit!(lcd_control, 2);

        let size_x = 8;
        let size_y = if double_height { 16 } else { 8 };

        let scanline = self.read_memory(SCANLINE_ADDRESS);

        // Check all sprites in memory 0xFE00-0xFE9F
        for sprite in 0..40 {
            let base_address = SPRITE_ATTRIBUTE_TABLE + (sprite * 4);

            let pos_y = self.read_memory(base_address);
            let pos_x = self.read_memory(base_address + 1);
            let location = self.read_memory(base_address + 2);
            let attributes = self.read_memory(base_address + 3);

            let flip_y = tbit!(attributes, 6);
            let flip_x = tbit!(attributes, 5);

            // If current scanline intercepts sprite
            if scanline >= pos_y && scanline < (pos_y + size_y) {
                let sprite_line = scanline - pos_y;
                let line = 
                    if flip_y {
                        2 * -(sprite_line as i16 - size_y as i16)
                    } else {
                        2 * sprite_line as i16
                    };

                let address = (SPRITE_DATA_ADDRESS + (location * 16) as usize) + line as usize;
                let data1 = self.read_memory(address);
                let data2 = self.read_memory(address + 1);

                for tile_pixel in 7..0 {
                    let color_bit = 
                        if flip_x {
                            -(tile_pixel as i16 - 7)
                        } else {
                            tile_pixel
                        };
                    
                    let color_num = (gbit!(data2, color_bit) << 1) | gbit!(data1, color_bit);

                    let color_address =
                        if tbit!(attributes, 4) {
                            PALETTE_49_ADDRESS
                        } else {
                            PALETTE_48_ADDRESS
                        };
                    
                    let color = self.get_color(color_num, color_address);

                    let transparent = match color {
                        Color::White => true,
                        _ => false
                    };

                    if !transparent {
                        let (red, green, blue) = color.rgb();
                        let pixel = 7 + pos_x as i16 - tile_pixel as i16;

                        if scanline < 0 || scanline > 143 {
                            warn!("Attempting to write scanline {} which is out of bounds!", scanline);
                        } else {
                            self.screen_buffer[pixel as usize][scanline as usize][0] = red;
                            self.screen_buffer[pixel as usize][scanline as usize][1] = green;
                            self.screen_buffer[pixel as usize][scanline as usize][2] = blue;
                        }
                    }
                }
            }
        }
    }

    fn get_color(&self, color_num: u8, address: usize) -> Color {
        let palette = self.read_memory(address);
        let (hi, lo) = match color_num {
            0 => (1, 0),
            1 => (3, 2),
            2 => (5, 4),
            3 => (7, 6),
            _ => panic!("Unknown color number {}!", color_num)
        };

        let color = (gbit!(palette, hi) << 1) | gbit!(palette, lo);

        match color {
            0 => Color::White,
            1 => Color::LightGrey,
            2 => Color::DarkGrey,
            3 => Color::Black,
            _ => unreachable!()
        }
    }

    pub fn input_down(&mut self, input: Inputs) {
        let was_unset = (self.pressed_inputs & input) != Inputs::empty();
        self.pressed_inputs |= input;
        let keys = self.memory.read(KEY_ADDRESS);

        // Only need interrupt if it wasn't already set and the input's select is set
        let need_interrupt = was_unset && tbit!(keys, input.select_location());

        if need_interrupt {
            self.request_interrupt(Interrupt::Joypad);
        }
    }

    pub fn input_up(&mut self, input: Inputs) {
        self.pressed_inputs &= !input;
    }

    fn joypad_state(&self) -> u8 {
        // Current status
        let old_state = self.memory.read(KEY_ADDRESS);
        let mut new_state = 0xFF;

        let active_select = if tbit!(old_state, 4) {5} else {4};
        // Unset the active select bit
        new_state = ubit!(new_state, active_select);

        // Check if any of the inputs are pressed
        for input in ALL_INPUTS.iter() {
            let select_bit = input.select_location();

            // If this input's select bit is the one that is currently active
            if select_bit == active_select && self.pressed_inputs.contains(*input) {

                // Unset the input since it's pressed
                new_state = ubit!(new_state, input.bit_location());
            }
        }

        return new_state;
    }
}
