struct Chip8 {
    v: [u16; 16],
    I: u16,
    pc: u16,
    sp: u16,
    stack: [u16; 16],
    keyboard: u16,
    
}

fn main() {
    println!("Hello, world!");
}
