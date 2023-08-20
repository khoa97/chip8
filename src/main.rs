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
use log::info;
use minifb::{Key, Window, WindowOptions};
use simple_logger::SimpleLogger;

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

fn main() {
    SimpleLogger::new().init().unwrap();

    let mut chip = Chip::default();
    let _ = chip.load_rom();

    // let mut buffer = chip.video;
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    println!("{:?}", buffer);
    chip.cycle();
    chip.cycle();
    chip.cycle();
    chip.cycle();
    chip.cycle();
    chip.cycle();
    chip.cycle();
    chip.cycle();
    info!("{:?}", chip.video);

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

    while window.is_open() && !window.is_key_down(Key::Escape) {
        draw_number_1(&mut buffer, WIDTH);
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}

fn draw_number_1(buffer: &mut [u32], width: usize) {
    let mid_x = width / 2;
    let start_y = 4;
    let end_y = 28;

    // Draw vertical line representing 1
    for y in start_y..end_y {
        buffer[y * width + mid_x] = 0xffffff;
    }

    // Optional: draw a small horizontal line at the top for styling the number "1"
    for x in (mid_x - 3)..mid_x {
        buffer[start_y * width + x] = 0xffffff;
    }
}
