// use chip::Chip;
// use minifb::{Key, Window, WindowOptions};
// use simple_logger::SimpleLogger;

// const CYCLES_PER_FRAME: usize = 10; // Adjust as needed
// const FRAMES_PER_TIMER_DECREMENT: usize = 10; // At 600Hz, every 10 frames we decrement the timers
// mod chip;
// fn main() {
//     SimpleLogger::new().init().unwrap();

//     let mut chip = Chip::default();
//     let _ = chip.load_rom();

//     let mut buffer = chip.video;

//     let mut window = Window::new("Test - ESC to exit", 64, 32, WindowOptions::default())
//         .unwrap_or_else(|e| {
//             panic!("{}", e);
//         });

//     window.limit_update_rate(Some(std::time::Duration::from_micros(16000))); // Adjust for 60 FPS

//     let mut frame_count = 0;
//     while window.is_open() && !window.is_key_down(Key::Escape) {
//         for _ in 0..CYCLES_PER_FRAME {
//             chip.cycle();
//         }

//         // Decrement timers approximately at 60Hz
//         // if frame_count % FRAMES_PER_TIMER_DECREMENT == 0 {
//         //     chip.decrement_timers();
//         // }
//         println!("{:?}", chip.video);
//         // Render
//         buffer.copy_from_slice(&chip.video);
//         window.update_with_buffer(&buffer, 64, 32).unwrap();

//         frame_count += 1;
//     }
// }
extern crate minifb;

use chip::Chip;
mod chip;
use log::debug;
use minifb::{Key, Window, WindowOptions};
use simple_logger::SimpleLogger;

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

fn main() {
    // SimpleLogger::new().init().unwrap();
    let mut chip = Chip::default();
    let args: Vec<String> = std::env::args().collect();
    if args.len() <= 1 {
        let _ = chip.load_rom("1-chip8-logo.ch8");
    } else {
        let _ = chip.load_rom(&args[1]);
    }
    debug!("{:?}", chip.video);
    print_instructions(&chip.memory);
    let mut window = Window::new(
        "Render 1",
        WIDTH,
        HEIGHT,
        WindowOptions {
            resize: true,
            ..WindowOptions::default()
        },
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });
    // window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
    while window.is_open() && !window.is_key_down(Key::Escape) {
        // draw_number_1(&mut buffer, WIDTH);
        chip.cycle();
        let buffer: Vec<u32> = chip
            .video
            .iter()
            .map(|&pixel| if pixel == 1 { 0x00FF66 } else { 0x0 })
            .collect();

        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}

// fn draw(buffer: &mut [u32]) {
//     for value in buffer.iter_mut() {
//         if *value == 1 {
//             *value = 0xFFFFFF;
//         }
//     }
// }

fn print_instructions(memory: &[u8; 4096]) {
    let start = 0x200;
    for mut i in (start..memory.len()).step_by(2) {
        let high = memory[i as usize];
        let low = memory[i as usize + 1];
        let opcode = ((high as u16) << 8) | (low as u16);

        if opcode == 0x000 {
            break;
        }
        println!("{:04x}", opcode);
    }
}
