use rand::rngs::ThreadRng;
use rand::Rng;

enum Action<'a> {
    ClearDisplay,
    DisplayScreen,
    KeyIsPressed(bool),
    KeyPress(&'a u16),
}

#[allow(non_snake_case)]
struct Chip8 { v: [u16; 16],
    I: u16,
    pc: usize,
    sp: usize,
    memory: [u8; 0xfff], // 4k memory
    stack: [usize; 16],
    // keyboard: u16,
    // display: [u8; 64 * 32]
    DT: u16, // Delay timer NOTE: Was this supposed to be 32 bits?
    ST: u16, // Sound timer NOTE: Was this supposed to be 32 bits?
    rng: ThreadRng,
}

impl Chip8 {
    pub fn new() -> Self {
        Chip8 {
            v: [0; 16],
            I: 0,
            pc: 0x200, // Programs start at 0x200 (512)
            sp: 0,
            memory: [0; 0xfff],
            stack: [0; 16],
            // keyboard: 0,
            DT: 0,
            ST: 0,
            rng: rand::thread_rng(),
        }
    }

    fn font_map(font: u8) -> Font {

    }

    fn jump(&mut self, nnn: u16) {
        self.pc = nnn.into();
    }

    fn push_stack(&mut self, addr: usize) {
        self.sp += 1;
        self.stack[self.sp] = addr;
    }

    pub fn emulate_op(&mut self) -> Option<Action> {
        let hi = self.memory[self.pc];
        let lo = self.memory[self.pc.wrapping_add(1)];
        let op: u16 = ((hi << 4) | lo).into();
        // self.pc += 2;
        self.pc = self.pc.wrapping_add(2);
        match op {
            n if (n & 0xf000) == 0x0000 => { // 0nnn - SYS addr
                let addr = n & 0x0fff;
                self.jump(addr);
                None
            },
            0x00e0 => { // CLS
                Some(Action::ClearDisplay)
            },
            0x00ee => { // RET
                self.pc = self.stack[self.sp];
                self.sp - 1;
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
                let kk = ((n & 0x00ff) >> 8).into();
                if self.v[x] == kk {
                    // self.pc += 2;
                    self.pc = self.pc.wrapping_add(2);
                }
                None
            },
            n if (n & 0xf000) == 0x4000 => { // SNE Vx, kk
                let x: usize =  ((n & 0x0f00) >> 8).into();
                let kk = ((n & 0x00ff) >> 8).into();
                if self.v[x] != kk {
                    // self.pc += 2;
                    self.pc = self.pc.wrapping_add(2);
                }
                None
            },
            n if (n & 0xf00f) == 0x5000 => { // SE Vx, Vy
                let x: usize = ((n & 0x0f00) >> 8).into();
                let y: usize = ((n & 0x00f0) >> 8).into();
                if self.v[x] == self.v[y] {
                    // self.pc += 2;
                    self.pc = self.pc.wrapping_add(2);
                }
                None
            },
            n if (n & 0xf000) == 0x6000 => { // LD Vx, Byte
                let x: usize = ((n & 0x0f00) >> 8).into();
                let byte = ((n & 0x00ff) >> 8).into();
                self.v[x] = byte;
                None
            },
            n if (n & 0xf000) == 0x7000 => { // ADD Vx, Byte
                let x: usize = ((n & 0x0f00) >> 8).into();
                let byte: u16 = ((n & 0x00ff) >> 8).into();
                // self.v[x] = self.v[x] + byte;
                self.v[x] = self.v[x].wrapping_add(byte);
                None
            },
            n if (n & 0xf00f) == 0x8000 => { // LD Vx, Vy
                let x: usize = ((n & 0x0f00) >> 8).into();
                let y: usize = ((n & 0x00f0) >> 8).into();

                self.v[x] = self.v[y];
                None
            },
            n if (n & 0xf00f) == 0x8001 => { // OR Vx, Vy
                let x: usize = ((n & 0x0f00) >> 8).into();
                let y: usize = ((n & 0x00f0) >> 8).into();

                self.v[x] |= self.v[y];
                None
            },
            n if (n & 0xf00f) == 0x8002 => { // AND Vx, Vy
                let x: usize = ((n & 0x0f00) >> 8).into();
                let y: usize = ((n & 0x00f0) >> 8).into();

                self.v[x] &= self.v[y];
                None
            },
            n if (n & 0xf00f) == 0x8003 => { // XOR Vx, Vy
                let x: usize = ((n & 0x0f00) >> 8).into();
                let y: usize = ((n & 0x00f0) >> 8).into();

                self.v[x] ^= self.v[y];
                None
            },
            n if (n & 0xf00f) == 0x8004 => { // ADD Vx, Vy
                let x: usize = ((n & 0x0f00) >> 8).into();
                let y: usize = ((n & 0x00f0) >> 8).into();

                // let byte = self.v[x] + self.v[y];
                let byte = self.v[x].wrapping_add(self.v[y]);
                if (byte & 0xff00) != 0x0000 {
                    self.v[0xf] = 1;
                } else {
                    self.v[0xf] = 0;
                }

                self.v[x] = byte & 0x00ff;
                None
            },
            n if (n & 0xf00f) == 0x8005 => { // SUB Vx, Vy
                let x: usize = ((n & 0x0f00) >> 8).into();
                let y: usize = ((n & 0x00f0) >> 8).into();

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

                if self.v[x] & 0x0001 == 0x0001 {
                    self.v[0xf] = 1;
                } else {
                    self.v[0xf] = 0;
                }

                self.v[x] = self.v[x] >> 1; 
                None
            },
            n if (n & 0xf00f) == 0x8007 => { // SUBN Vx, Vy
                let x: usize = ((n & 0x0f00) >> 8).into();
                let y: usize = ((n & 0x00f0) >> 8).into();

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

                if self.v[x] & 0x8000 == 0x8000 {
                    self.v[0xf] = 1;
                } else {
                    self.v[0xf] = 0;
                }

                self.v[x] = self.v[x] << 1; 
                None
            },
            n if (n & 0xf00f) == 0x9000 => { // SNE Vx, Vy
                let x: usize =  ((n & 0x0f00) >> 8).into();
                let y: usize = ((n & 0x00f0) >> 8).into();
                if self.v[x] != self.v[y] {
                    // self.pc += 2;
                    self.pc = self.pc.wrapping_add(2);
                }
                None
            },
            n if (n & 0xf000) == 0xa000 => { // LD I, addr
                let byte = ((n & 0x00ff) >> 8).into();

                self.I = byte;
                None
            },
            n if (n & 0xf000) == 0xb000 => { // JP V0, addr
                let addr = n & 0x0fff;

                self.jump(addr + self.v[0]);
                None
            },
            n if (n & 0xf000) == 0xc000 => { // RND Vx, byte
                let x: usize =  ((n & 0x0f00) >> 8).into();
                let kk: u16 = ((n & 0x00ff) >> 8).into();

                let r: u16 = self.rng.gen_range(0..256);
                
                self.v[x] = (r & kk).into();
                None
            },
            n if (n & 0xf000) == 0xd000 => { // DRW Vx, Vy, nibble
                Some(Action::DisplayScreen)
            },
            n if (n & 0xf0ff) == 0xe09e => { // SKP Vx
                Some(Action::KeyIsPressed(true))
            },
            n if (n & 0xf0ff) == 0xe0a1 => { // SKNP Vx
                Some(Action::KeyIsPressed(false))
            },
            n if (n & 0xf0ff) == 0xf007 => { // LD Vx, DT
                let x: usize =  ((n & 0x0f00) >> 8).into();
                self.v[x] = self.DT;
                None
            },
            n if (n & 0xf0ff) == 0xf007 => { // LD Vx, DT
                let x: usize =  ((n & 0x0f00) >> 8).into();
                self.v[x] = self.DT;
                None
            },
            n if (n & 0xf0ff) == 0xf00A => { // LD Vx, K
                let x: usize =  ((n & 0x0f00) >> 8).into();
                Some(Action::KeyPress(&self.v[x]))
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
                self.I = self.I.wrapping_add(self.v[x]);
                None
            },
            n if (n & 0xf0ff) == 0xf029 => { // LD F, Vx
                panic!("Implement this...")
            },
            n if (n & 0xf0ff) == 0xf033 => { // LD B, Vx

            },
            _ => panic!("Opcode not implemented: {}", op),
        }
    }
}

fn main() {
    let mut chip8 = Chip8::new();
    chip8.emulate_op();
}
