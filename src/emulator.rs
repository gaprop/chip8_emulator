use std::io::prelude::*;
use std::fs::File;
use rand::rngs::ThreadRng;
use rand::Rng;

pub static FONT_SET: [u8; 80] = [
    0xf0, 0x90, 0x90, 0x90, 0xf0,
    0x20, 0x60, 0x20, 0x20, 0x70,
    0xf0, 0x10, 0xf0, 0x80, 0xf0,
    0xf0, 0x10, 0xf0, 0x10, 0xf0,
    0x90, 0x90, 0xf0, 0x10, 0x10,
    0xf0, 0x80, 0xf0, 0x10, 0xf0,
    0xf0, 0x80, 0xf0, 0x90, 0xf0,
    0xf0, 0x10, 0x20, 0x40, 0x40,
    0xf0, 0x90, 0xf0, 0x90, 0xf0,
    0xf0, 0x90, 0xf0, 0x10, 0xf0,
    0xf0, 0x90, 0xf0, 0x90, 0x90,
    0xe0, 0x90, 0xe0, 0x90, 0xe0,
    0xf0, 0x80, 0x80, 0x80, 0xf0,
    0xe0, 0x90, 0x90, 0x90, 0xe0,
    0xf0, 0x80, 0xf0, 0x80, 0xf0,
    0xf0, 0x80, 0xf0, 0x80, 0x80,
];

#[derive(Debug)]
pub enum Action<'a> { 
    DisplayScreen(&'a [u32; WIDTH * HEIGHT]),
    WaitForKeyPress,
}

#[derive(Debug)]
pub enum Event {
    KeyPress(u8),
    WaitingKeyPress(u8),
}

pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 32;

#[allow(non_snake_case)]
pub struct Chip8 { 
    screen: [u32; WIDTH * HEIGHT],
    v: [u8; 16],
    I: u16,
    pc: usize,
    sp: usize,
    memory: [u8; 0xfff], // 4k memory
    stack: [usize; 16],
    DT: u8, 
    ST: u8, 
    rng: ThreadRng,
}

impl Chip8 {
    pub fn new(path: &str) -> Self {
        let mut memory = [0; 0xfff];
        let mut f = File::open(path).unwrap();
        f.read_exact(&mut memory[0x200..]);
        for i in 0..80 {
            memory[i] = FONT_SET[i];
        }
        Chip8 {
            screen: [0; WIDTH * HEIGHT],
            v: [0; 16],
            I: 0,
            pc: 0x200, // Programs start at 0x200 (512)
            sp: 0,
            memory: memory,
            stack: [0; 16],
            DT: 0,
            ST: 0,
            rng: rand::thread_rng(),
        }
    }

    fn get_font_location(&self, x: usize) -> usize {
        if x > 0x0f {
            panic!("Not a location for fonts")
        }
        (x * 5).into()
    }

    fn jump(&mut self, nnn: u16) {
        self.pc = nnn.into();
    }

    fn push_stack(&mut self, addr: usize) {
        self.sp += 1;
        self.stack[self.sp] = addr;
    }

    fn get_screen_pos(&self, x: usize, y: usize) -> usize {
        (x % WIDTH) + ((y % HEIGHT) * WIDTH)
        // x + y * WIDTH
    }

    pub fn decreament_timer(&mut self) {
        self.DT = self.DT.wrapping_sub(1);
    }

    pub fn emulate_op(&mut self, event: Option<Event>) -> Option<Action> {
        let hi = self.memory[self.pc];
        let lo = self.memory[self.pc.wrapping_add(1)];
        let op: u16 = ((hi as u16) << 8) | (lo as u16);
        match event {
            Some(Event::WaitingKeyPress(_)) => (),
            _ => self.pc = self.pc.wrapping_add(2),
        }
        match op {
            0x00e0 => { // CLS
                self.screen = [0; WIDTH * HEIGHT];
                Some(Action::DisplayScreen(&self.screen))
            },
            0x00ee => { // RET
                self.pc = self.stack[self.sp];
                self.sp = self.sp.wrapping_sub(1);
                None
            },
            n if (n & 0xf000) == 0x0000 => { // 0nnn - SYS addr
                let addr = n & 0x0fff;
                self.jump(addr);
                None
            },
            n if (n & 0xf000) == 0x1000 => { // JP addr
                let addr = n & 0x0fff;
                self.jump(addr);
                None
            },
            n if (n & 0xf000) == 0x2000 => { // CALL addr
                self.push_stack(self.pc);

                let addr = n & 0x0fff;
                self.jump(addr);
                None
            },
            n if (n & 0xf000) == 0x3000 => { // SE Vx, kk
                let x: usize = ((n & 0x0f00) >> 8).into();
                let kk = (n & 0x00ff) as u8;
                if self.v[x] == kk {
                    self.pc = self.pc.wrapping_add(2);
                }
                None
            },
            n if (n & 0xf000) == 0x4000 => { // SNE Vx, kk
                let x: usize =  ((n & 0x0f00) >> 8).into();
                let kk = (n & 0x00ff) as u8;
                if self.v[x] != kk {
                    self.pc = self.pc.wrapping_add(2);
                }
                None
            },
            n if (n & 0xf00f) == 0x5000 => { // SE Vx, Vy
                let x: usize = ((n & 0x0f00) >> 8).into();
                let y: usize = ((n & 0x00f0) >> 4).into();
                if self.v[x] == self.v[y] {
                    // self.pc += 2;
                    self.pc = self.pc.wrapping_add(2);
                }
                None
            },
            n if (n & 0xf000) == 0x6000 => { // LD Vx, Byte
                let x: usize = ((n & 0x0f00) >> 8).into();
                let byte = (n & 0x00ff) as u8;
                self.v[x] = byte;
                None
            },
            n if (n & 0xf000) == 0x7000 => { // ADD Vx, Byte
                let x: usize = ((n & 0x0f00) >> 8).into();
                let byte: u8 = (n & 0x00ff) as u8;
                // self.v[x] = self.v[x] + byte;
                self.v[x] = self.v[x].wrapping_add(byte);
                None
            },
            n if (n & 0xf00f) == 0x8000 => { // LD Vx, Vy
                let x: usize = ((n & 0x0f00) >> 8).into();
                let y: usize = ((n & 0x00f0) >> 4).into();

                self.v[x] = self.v[y];
                None
            },
            n if (n & 0xf00f) == 0x8001 => { // OR Vx, Vy
                let x: usize = ((n & 0x0f00) >> 8).into();
                let y: usize = ((n & 0x00f0) >> 4).into();

                self.v[x] |= self.v[y];
                None
            },
            n if (n & 0xf00f) == 0x8002 => { // AND Vx, Vy
                let x: usize = ((n & 0x0f00) >> 8).into();
                let y: usize = ((n & 0x00f0) >> 4).into();

                self.v[x] &= self.v[y];
                None
            },
            n if (n & 0xf00f) == 0x8003 => { // XOR Vx, Vy
                let x: usize = ((n & 0x0f00) >> 8).into();
                let y: usize = ((n & 0x00f0) >> 4).into();

                self.v[x] ^= self.v[y];
                None
            },
            n if (n & 0xf00f) == 0x8004 => { // ADD Vx, Vy
                let x: usize = ((n & 0x0f00) >> 8).into();
                let y: usize = ((n & 0x00f0) >> 4).into();

                let vx: u16 = self.v[x].into();
                let vy: u16 = self.v[y].into();

                let byte = vx.wrapping_add(vy);
                if byte > 255 {
                    self.v[0xf] = 1;
                } else {
                    self.v[0xf] = 0;
                }

                self.v[x] = self.v[x].wrapping_add(self.v[y]);
                None
            },
            n if (n & 0xf00f) == 0x8005 => { // SUB Vx, Vy
                let x: usize = ((n & 0x0f00) >> 8).into();
                let y: usize = ((n & 0x00f0) >> 4).into();

                if self.v[x] > self.v[y] {
                    self.v[0xf] = 1;
                } else {
                    self.v[0xf] = 0;
                }

                self.v[x] = self.v[x].wrapping_sub(self.v[y]);
                None
            },
            n if (n & 0xf00f) == 0x8006 => { // SHR Vx{, Vy}
                let x: usize = ((n & 0x0f00) >> 8).into();

                self.v[0xf] = self.v[x] & 0x01;

                self.v[x] = self.v[x] >> 1; 
                None
            },
            n if (n & 0xf00f) == 0x8007 => { // SUBN Vx, Vy
                let x: usize = ((n & 0x0f00) >> 8).into();
                let y: usize = ((n & 0x00f0) >> 4).into();

                if self.v[y] > self.v[x] {
                    self.v[0xf] = 1;
                } else {
                    self.v[0xf] = 0;
                }

                self.v[x] = self.v[y].wrapping_sub(self.v[x]);
                None
            },
            n if (n & 0xf00f) == 0x800E => { // SHL Vx{, Vy}
                let x: usize = ((n & 0x0f00) >> 8).into();

                self.v[0xf] = (self.v[x] & 0x80) >> 7;
                self.v[x] = self.v[x] << 1; 
                None
            },
            n if (n & 0xf00f) == 0x9000 => { // SNE Vx, Vy
                let x: usize =  ((n & 0x0f00) >> 8).into();
                let y: usize = ((n & 0x00f0) >> 4).into();
                if self.v[x] != self.v[y] {
                    self.pc = self.pc.wrapping_add(2);
                }
                None
            },
            n if (n & 0xf000) == 0xa000 => { // LD I, addr
                let byte = (n & 0x0fff).into();

                self.I = byte;
                None
            },
            n if (n & 0xf000) == 0xb000 => { // JP V0, addr
                let addr = n & 0x0fff;
                self.jump(addr + (self.v[0] as u16));
                None
            },
            n if (n & 0xf000) == 0xc000 => { // RND Vx, byte
                let x: usize =  ((n & 0x0f00) >> 8).into();
                let kk: u8 = (n & 0x00ff) as u8;

                let r: u8 = self.rng.gen_range(0..=255);
                
                self.v[x] = r & kk;
                None
            },
            n if (n & 0xf000) == 0xd000 => { // DRW Vx, Vy, nibble
                let x: usize = ((n & 0x0f00) >> 8).into();
                let y: usize = ((n & 0x00f0) >> 4).into();
                let n: usize = (n & 0x000f).into();
                let mut collision = false;
                for i in 0..n {
                    let byte: u8 = self.memory[self.I as usize + i];
                    for j in 0..8 {
                        let bit: u32 = (byte.wrapping_shr(7 - j as u32) & 0x1).into();
                        let pixel = self.screen[self.get_screen_pos((self.v[x] as usize) + j, (self.v[y] as usize) + i)];
                        let new_pixel = (pixel & 0x01) ^ bit;
                        if new_pixel == (pixel & 0x01) { 
                            collision = true;
                        }
                        if new_pixel == 0x01 {
                            self.screen[self.get_screen_pos((self.v[x] as usize) + j, (self.v[y] as usize) + i)] = 0x00FFFFFF;
                        } else {                                                                         
                            self.screen[self.get_screen_pos((self.v[x] as usize) + j, (self.v[y] as usize) + i)] = 0x00000000;
                        }
                    }
                }
                if collision {
                    self.v[0xf] = 0x1;
                }
                Some(Action::DisplayScreen(&self.screen))
            },
            n if (n & 0xf0ff) == 0xe09e => { // SKP Vx
                let x: usize = ((n & 0x0f00) >> 8).into();
                match event {
                    Some(Event::KeyPress(n)) if self.v[x] == n => {
                        self.pc = self.pc.wrapping_add(2);
                    },
                    _ => (),
                }
                None
            },
            n if (n & 0xf0ff) == 0xe0a1 => { // SKNP Vx
                let x: usize = ((n & 0x0f00) >> 8).into();
                match event {
                    Some(Event::KeyPress(n)) if self.v[x] == n => (),
                    _ => self.pc = self.pc.wrapping_add(2),
                }
                None
            },
            n if (n & 0xf0ff) == 0xf007 => { // LD Vx, DT
                let x: usize =  ((n & 0x0f00) >> 8).into();
                self.v[x] = self.DT;
                None
            },
            n if (n & 0xf0ff) == 0xf00A => { // LD Vx, K
                let x: usize =  ((n & 0x0f00) >> 8).into();
                match event {
                    Some(Event::WaitingKeyPress(n)) => {
                        self.v[x] = n;
                        None
                    }
                    _ => Some(Action::WaitForKeyPress)
                }
            },
            n if (n & 0xf0ff) == 0xf015 => { // LD DT, Vx 
                let x: usize =  ((n & 0x0f00) >> 8).into();
                self.DT = self.v[x];
                None
            },
            n if (n & 0xf0ff) == 0xf018 => { // LD ST, Vx 
                let x: usize =  ((n & 0x0f00) >> 8).into();
                self.ST = self.v[x];
                None
            },
            n if (n & 0xf0ff) == 0xf01e => { // ADD I, Vx
                let x: usize =  ((n & 0x0f00) >> 8).into();
                self.I = self.I.wrapping_add(self.v[x].into());
                None
            },
            n if (n & 0xf0ff) == 0xf029 => { // LD F, Vx
                let x: usize =  ((n & 0x0f00) >> 8).into();
                let i = self.get_font_location(self.v[x] as usize);
                self.I = i as u16;
                None
            },
            n if (n & 0xf0ff) == 0xf033 => { // LD B, Vx
                let x: usize =  ((n & 0x0f00) >> 8).into();
                self.memory[self.I as usize]     = self.v[x] / 100;
                self.memory[self.I as usize + 1] = (self.v[x] / 10) % 10;
                self.memory[self.I as usize + 2] = (self.v[x] % 100) % 10;
                None
            },
            n if (n & 0xf0ff) == 0xf055 => { // LD [I], Vx
                let x: usize =  ((n & 0x0f00) >> 8).into();
                for i in 0..=x {
                    self.memory[(self.I as usize) + i] = self.v[i];
                }
                None
            },
            n if (n & 0xf0ff) == 0xf065 => { // LD Vx, [I]
                let x: usize =  ((n & 0x0f00) >> 8).into();
                for i in 0..=x {
                    self.v[i] = self.memory[(self.I as usize) + i];
                }
                None
            },
            _ => panic!("Opcode not implemented: {:#05x}", op),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sys_addr() {
        let mut memory = [0x00; 0xfff];
        memory[0x200] = 0x03;
        memory[0x201] = 0x01;
        let mut cpu = Chip8::new();
        cpu.memory = memory;
        cpu.emulate_op();
        assert_eq!(cpu.pc, 0x301);
    }

    #[test]
    fn ret() {
        let mut memory = [0x00; 0xfff];
        memory[0x200] = 0x00;
        memory[0x201] = 0xee;
        let mut cpu = Chip8::new();
        cpu.memory = memory;
        cpu.sp = 3;
        cpu.stack[3] = 0x0301;
        cpu.emulate_op();
        assert_eq!(cpu.pc, 0x0301);
        assert_eq!(cpu.sp, 2);
    }

    #[test]
    fn jp_addr() {
        let mut memory = [0x00; 0xfff];
        memory[0x200] = 0x14;
        memory[0x201] = 0x55;
        let mut cpu = Chip8::new();
        cpu.memory = memory;
        cpu.emulate_op();
        assert_eq!(cpu.pc, 0x0455);
    }

    #[test]
    fn call() {
        let mut memory = [0x00; 0xfff];
        memory[0x200] = 0x24;
        memory[0x201] = 0x55;
        let mut cpu = Chip8::new();
        let mut stack = [0; 16];
        stack[1] = 0x202;
        cpu.memory = memory;
        cpu.emulate_op();
        assert_eq!(cpu.pc, 0x0455);
        assert_eq!(cpu.sp, 1);
        assert_eq!(cpu.stack, stack);
    }

    #[test]
    fn se() {
        let mut memory = [0x00; 0xfff];
        memory[0x200] = 0x30;
        memory[0x201] = 0x55;
        let mut cpu = Chip8::new();
        cpu.memory = memory;
        cpu.v[0] = 0x55;
        cpu.emulate_op();
        assert_eq!(cpu.pc, 0x204);
    }

    #[test]
    fn sne_vx_byte() {
        let mut memory = [0x00; 0xfff];
        memory[0x200] = 0x40;
        memory[0x201] = 0x54;
        let mut cpu = Chip8::new();
        cpu.memory = memory;
        cpu.v[0] = 0x55;
        cpu.emulate_op();
        assert_eq!(cpu.pc, 0x204);
    }

    #[test]
    fn se_vx_vy() {
        let mut memory = [0x00; 0xfff];
        memory[0x200] = 0x50;
        memory[0x201] = 0x10;
        let mut cpu = Chip8::new();
        cpu.memory = memory;
        cpu.v[0] = 0x55;
        cpu.v[1] = 0x55;
        cpu.emulate_op();
        assert_eq!(cpu.pc, 0x204);
    }

    #[test]
    fn ld_vx_byte() {
        let mut memory = [0x00; 0xfff];
        memory[0x200] = 0x60;
        memory[0x201] = 0x10;
        let mut cpu = Chip8::new();
        cpu.memory = memory;
        cpu.emulate_op();
        assert_eq!(cpu.v[0], 0x10);
    }

    #[test]
    fn add_vx_byte() {
        let mut memory = [0x00; 0xfff];
        memory[0x200] = 0x70;
        memory[0x201] = 0x01;
        let mut cpu = Chip8::new();
        cpu.memory = memory;
        cpu.v[0] = 0x01;
        cpu.emulate_op();
        assert_eq!(cpu.v[0], 0x02);
    }

    #[test]
    fn ld_vx_vy() {
        let mut memory = [0x00; 0xfff];
        memory[0x200] = 0x80;
        memory[0x201] = 0x10;
        let mut cpu = Chip8::new();
        cpu.memory = memory;
        cpu.v[1] = 0x01;
        cpu.emulate_op();
        assert_eq!(cpu.v[0], 0x01);
    }

    #[test]
    fn or() {
        let mut memory = [0x00; 0xfff];
        memory[0x200] = 0x80;
        memory[0x201] = 0x11;
        let mut cpu = Chip8::new();
        cpu.memory = memory;
        cpu.v[0] = 0x02;
        cpu.v[1] = 0x01;
        cpu.emulate_op();
        assert_eq!(cpu.v[0], 0x03);
    }

    #[test]
    fn and() {
        let mut memory = [0x00; 0xfff];
        memory[0x200] = 0x80;
        memory[0x201] = 0x12;
        let mut cpu = Chip8::new();
        cpu.memory = memory;
        cpu.v[0] = 0x03;
        cpu.v[1] = 0x01;
        cpu.emulate_op();
        assert_eq!(cpu.v[0], 0x01);
    }

    #[test]
    fn xor() {
        let mut memory = [0x00; 0xfff];
        memory[0x200] = 0x80;
        memory[0x201] = 0x13;
        let mut cpu = Chip8::new();
        cpu.memory = memory;
        cpu.v[0] = 0x03;
        cpu.v[1] = 0x01;
        cpu.emulate_op();
        assert_eq!(cpu.v[0], 0x02);
    }

    #[test]
    fn add_vx_vy() {
        let mut memory = [0x00; 0xfff];
        memory[0x200] = 0x80;
        memory[0x201] = 0x14;
        let mut cpu = Chip8::new();
        cpu.memory = memory;
        cpu.v[0] = 0xff;
        cpu.v[1] = 0x01;
        cpu.emulate_op();
        assert_eq!(cpu.v[0], 0x00);
        assert_eq!(cpu.v[0xf], 0x01);
    }

    #[test]
    fn sub_vx_vy() {
        let mut memory = [0x00; 0xfff];
        memory[0x200] = 0x80;
        memory[0x201] = 0x15;
        let mut cpu = Chip8::new();
        cpu.memory = memory;
        cpu.v[0] = 0xff;
        cpu.v[1] = 0x01;
        cpu.emulate_op();
        assert_eq!(cpu.v[0], 0xfe);
        assert_eq!(cpu.v[0xf], 0x01);
    }

    #[test]
    fn shr() {
        let mut memory = [0x00; 0xfff];
        memory[0x200] = 0x80;
        memory[0x201] = 0x16;
        let mut cpu = Chip8::new();
        cpu.memory = memory;
        cpu.v[0] = 0xff;
        cpu.emulate_op();
        assert_eq!(cpu.v[0], 0x7f);
        assert_eq!(cpu.v[0xf], 0x01);
    }

    #[test]
    fn subn() {
        let mut memory = [0x00; 0xfff];
        memory[0x200] = 0x80;
        memory[0x201] = 0x17;
        let mut cpu = Chip8::new();
        cpu.memory = memory;
        cpu.v[0] = 0x01;
        cpu.v[1] = 0xff;
        cpu.emulate_op();
        assert_eq!(cpu.v[0], 0xfe);
        assert_eq!(cpu.v[0xf], 0x01);
    }

    #[test]
    fn shl() {
        let mut memory = [0x00; 0xfff];
        memory[0x200] = 0x80;
        memory[0x201] = 0x1e;
        let mut cpu = Chip8::new();
        cpu.memory = memory;
        cpu.v[0] = 0x7f;
        cpu.emulate_op();
        assert_eq!(cpu.v[0], 0xfe);
        assert_eq!(cpu.v[0xf], 0x00);
    }

    #[test]
    fn sne_vx_vy() {
        let mut memory = [0x00; 0xfff];
        memory[0x200] = 0x90;
        memory[0x201] = 0x10;
        let mut cpu = Chip8::new();
        cpu.memory = memory;
        cpu.v[0] = 0xff;
        cpu.v[1] = 0x01;
        cpu.emulate_op();
        assert_eq!(cpu.pc, 0x204);
    }

    #[test]
    fn ld_i_addr() {
        let mut memory = [0x00; 0xfff];
        memory[0x200] = 0xaf;
        memory[0x201] = 0xff;
        let mut cpu = Chip8::new();
        cpu.memory = memory;
        cpu.emulate_op();
        assert_eq!(cpu.I, 0xfff);
    }

    #[test]
    fn jp_v0_addr() {
        let mut memory = [0x00; 0xfff];
        memory[0x200] = 0xb3;
        memory[0x201] = 0x00;
        let mut cpu = Chip8::new();
        cpu.memory = memory;
        cpu.v[0] = 0x1;
        cpu.emulate_op();
        assert_eq!(cpu.pc, 0x301);
    }

    // #[test]
    // fn skp() {
        // let mut memory = [0x00; 0xfff];
        // memory[0x200] = 0xb3;
        // memory[0x201] = 0x00;
        // let mut cpu = Chip8::new();
        // cpu.memory = memory;
        // cpu.v[0] = 0x1;
        // cpu.emulate_op();
        // assert_eq!(cpu.pc, 0x301);
    // }

    // #[test]
    // fn sknp() {
        // let mut memory = [0x00; 0xfff];
        // memory[0x200] = 0xb3;
        // memory[0x201] = 0x00;
        // let mut cpu = Chip8::new();
        // cpu.memory = memory;
        // cpu.v[0] = 0x1;
        // cpu.emulate_op();
        // assert_eq!(cpu.pc, 0x301);
    // }
      
    #[test]
    fn ld_dt_vx() {
        let mut memory = [0x00; 0xfff];
        memory[0x200] = 0xf0;
        memory[0x201] = 0x15;
        let mut cpu = Chip8::new();
        cpu.memory = memory;
        cpu.v[0] = 0x5;
        cpu.emulate_op();
        assert_eq!(cpu.DT, 0x5);
    }

    #[test]
    fn ld_st_vx() {
        let mut memory = [0x00; 0xfff];
        memory[0x200] = 0xf0;
        memory[0x201] = 0x18;
        let mut cpu = Chip8::new();
        cpu.memory = memory;
        cpu.v[0] = 0x5;
        cpu.emulate_op();
        assert_eq!(cpu.ST, 0x5);
    }

    #[test]
    fn add_i_vx() {
        let mut memory = [0x00; 0xfff];
        memory[0x200] = 0xf0;
        memory[0x201] = 0x1e;
        let mut cpu = Chip8::new();
        cpu.memory = memory;
        cpu.v[0] = 0x5;
        cpu.emulate_op();
        assert_eq!(cpu.I, 0x5);
    }

    #[test]
    fn ld_f_vx() {
        let mut memory = [0x00; 0xfff];
        memory[0x200] = 0xf0;
        memory[0x201] = 0x29;
        let mut cpu = Chip8::new();
        cpu.memory = memory;
        cpu.v[0] = 0xf;
        cpu.I = 0x300;
        cpu.emulate_op();
        assert_eq!(cpu.I, 75);
    }

    #[test]
    fn ld_b_vx() {
        let mut memory = [0x00; 0xfff];
        memory[0x200] = 0xf0;
        memory[0x201] = 0x33;
        let mut cpu = Chip8::new();
        cpu.memory = memory;
        cpu.v[0] = 123;
        cpu.I = 0x300;
        cpu.emulate_op();
        assert_eq!(cpu.memory[0x300], 1);
        assert_eq!(cpu.memory[0x301], 2);
        assert_eq!(cpu.memory[0x302], 3);
    }

    #[test]
    fn ld_mem_i_vx() {
        let mut memory = [0x00; 0xfff];
        memory[0x200] = 0xf3;
        memory[0x201] = 0x55;
        let mut cpu = Chip8::new();
        cpu.memory = memory;
        cpu.v[0] = 0x01;
        cpu.v[1] = 0x02;
        cpu.v[2] = 0x03;
        cpu.I = 0x300;
        cpu.emulate_op();
        assert_eq!(cpu.memory[0x300], 0x1);
        assert_eq!(cpu.memory[0x301], 0x2);
        assert_eq!(cpu.memory[0x302], 0x3);
    }

    #[test]
    fn ld_vx_mem_i() {
        let mut memory = [0x00; 0xfff];
        memory[0x200] = 0xf3;
        memory[0x201] = 0x65;
        let mut cpu = Chip8::new();
        cpu.memory = memory;
        cpu.memory[0x300] = 0x1;
        cpu.memory[0x301] = 0x2;
        cpu.memory[0x302] = 0x3;
        cpu.I = 0x300;
        cpu.emulate_op();
        assert_eq!(cpu.v[0], 1);
        assert_eq!(cpu.v[1], 2);
        assert_eq!(cpu.v[2], 3);
    }
}
