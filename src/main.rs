extern crate minifb;

use std::collections::HashMap;

use audio::Audio;
use chip::Chip;
mod audio;
mod chip;
use minifb::{Key, Window, WindowOptions};
// use simple_logger::SimpleLogger;

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

fn main() {
    // SimpleLogger::new().init().unwrap();
    let audio = Audio::new();
    let mut chip = Chip::default();
    let args: Vec<String> = std::env::args().collect();
    if args.len() <= 1 {
        let _ = chip.load_rom("1-chip8-logo.ch8");
    } else {
        let _ = chip.load_rom(&args[1]);
    }

    let mut window = Window::new(
        "Render 1",
        WIDTH,
        HEIGHT,
        WindowOptions {
            resize: true,
            scale: minifb::Scale::X16,
            ..WindowOptions::default()
        },
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });
    let chip8_key_map = get_chip8_key_map();
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let keys_pressed = window.get_keys_pressed(minifb::KeyRepeat::Yes);
        for &key in &keys_pressed {
            if let Some(&chip_key) = chip8_key_map.get(&key) {
                chip.keyboard[chip_key] = 1;
                println!("PRESSED")
            }
        }

        // Handle key releases
        let keys_released = window.get_keys_released();
        for &key in &keys_released {
            if let Some(&chip_key) = chip8_key_map.get(&key) {
                chip.keyboard[chip_key] = 0;
            }
        }

        if chip.audio_reg > 0 {
            audio.beep()
        } else {
            audio.stop()
        }

        chip.cycle();

        let buffer: Vec<u32> = chip
            .video
            .iter()
            .map(|&pixel| if pixel == 1 { 0x00FF66 } else { 0x0 })
            .collect();

        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}

fn get_chip8_key_map() -> HashMap<Key, usize> {
    use minifb::Key::*;
    [
        (Key1, 0x1),
        (Key2, 0x2),
        (Key3, 0x3),
        (Key4, 0xC),
        (Q, 0x4),
        (W, 0x5),
        (E, 0x6),
        (R, 0xD),
        (A, 0x7),
        (S, 0x8),
        (D, 0x9),
        (F, 0xE),
        (Z, 0xA),
        (X, 0x0),
        (C, 0xB),
        (V, 0xF),
    ]
    .iter()
    .cloned()
    .collect()
}
