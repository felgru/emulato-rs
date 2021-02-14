use std::env;
use std::fs::File;

use emulato_rs::chip8;

fn main() {
    let mut chip8 = chip8::Chip8::new();
    let filename = match env::args().into_iter().nth(1) {
        Some(s) => s,
        None => panic!("missing filename"),
    };
    println!("loading {}", filename);
    let f = File::open(filename).unwrap();
    chip8.load_rom(f).unwrap();
    chip8.run();
}
