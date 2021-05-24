pub mod emulator;
extern crate minifb;

use minifb::{Key, Window, WindowOptions};

use crate::emulator::{Chip8, WIDTH, HEIGHT};

use std::{thread, time};

fn main() {
    
    let mut chip8 = Chip8::new("./../rom/PONG");
    let mut window = Window::new(
        "Chip8 Interperter",
        WIDTH,
        HEIGHT,
        WindowOptions::default()
    ).unwrap();

    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    // // LD v[0], 0xff
    // chip8.memory[0x200] = 0x60;
    // chip8.memory[0x201] = 0xff;

    // // LD v[1], 0x1f
    // chip8.memory[0x202] = 0x61;
    // chip8.memory[0x203] = 0x00;
      
    // // LD [I], v[0]
    // chip8.memory[0x204] = 0xf0;
    // chip8.memory[0x205] = 0x55;
      
    // // LD v[0], 0x00
    // chip8.memory[0x206] = 0x60;
    // chip8.memory[0x207] = 0x40;

    // // DRW
    // chip8.memory[0x208] = 0xd0;
    // chip8.memory[0x209] = 0x11;

    // chip8.emulate_op();
    // chip8.emulate_op();
    // chip8.emulate_op();
    // chip8.emulate_op();
    // chip8.emulate_op();
    while window.is_open() && !window.is_key_down(Key::Escape) {
        chip8.emulate_op();
        chip8.decreament_timer();
        window
            .update_with_buffer(&chip8.screen, WIDTH, HEIGHT)
            .unwrap();

        // // thread::sleep(time::Duration::from_millis(500));
        while window.is_key_down(Key::Space) {
            window
                .update_with_buffer(&chip8.screen, WIDTH, HEIGHT)
                .unwrap();
        }
    }
}
