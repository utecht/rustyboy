#![allow(clippy::upper_case_acronyms, dead_code)]
mod cpu;
use raylib::prelude::*;

fn main() {
    println!("Hello, world!");
    let mut cpu = cpu::Cpu::new(None, vec![0; 0xFFFF]);
    let (mut rl, thread) = raylib::init().size(640, 480).title("Hello, World").build();

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::WHITE);
        d.draw_text("Hello, world!", 12, 12, 20, Color::BLACK);
        cpu.step();
    }
}
