pub mod emulator;
extern crate minifb;

use minifb::{Key, Window, WindowOptions};

use crate::emulator::{Chip8, WIDTH, HEIGHT, Event, Action};

use std::env;

fn main() {
    
    let filename = env::args().nth(1).expect("Needs a file");
    let mut chip8 = Chip8::new(&filename);
    let mut window = Window::new(
        "Chip8 Interperter",
        WIDTH,
        HEIGHT,
        WindowOptions::default()
    ).unwrap();

    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let key = window
            .get_keys()
            .unwrap_or_else(|| vec![])
            .pop();
        let key = to_valid_key(key).map(|k| Event::KeyPress(k));
        match chip8.emulate_op(key) {
            Some(Action::DisplayScreen(screen)) => {
                window
                    .update_with_buffer(&screen[..], WIDTH, HEIGHT)
                    .unwrap();
            },
            Some(Action::WaitForKeyPress) => {
                loop {
                    let key = window.get_keys().unwrap_or_else(|| vec![]).pop();
                    match to_valid_key(key) {
                        Some(key) => {
                            chip8.emulate_op(Some(Event::WaitingKeyPress(key)));
                            break;
                        }
                        _ => window.update(),
                    }
                }
            }
            _ => (),
        }
        chip8.decreament_timer();
    }
}

fn to_valid_key(key: Option<Key>) -> Option<u8> {
    match key {
        Some(Key::Key1) => Some(0x1),
        Some(Key::Key2) => Some(0x2),
        Some(Key::Key3) => Some(0x3),
        Some(Key::Key4) => Some(0xc),
        Some(Key::Q)    => Some(0x4),
        Some(Key::W)    => Some(0x5),
        Some(Key::E)    => Some(0x6),
        Some(Key::R)    => Some(0xd),
        Some(Key::A)    => Some(0x7),
        Some(Key::S)    => Some(0x8),
        Some(Key::D)    => Some(0x9),
        Some(Key::F)    => Some(0xe),
        Some(Key::Z)    => Some(0xa),
        Some(Key::X)    => Some(0x0),
        Some(Key::C)    => Some(0xb),
        Some(Key::V)    => Some(0xf),
        _               => None,
    }
}
