use rand::Rng;
use std::io::{self, BufReader, Read};
const START_ADDRESS: u16 = 0x200;
struct Chip {
    memory: [u8; 4096], // 4kb
    general_purpose_reg: [u8; 16],
    i_reg: u16,
    delay_reg: u8, // 8bit
    audio_reg: u8,
    program_counter: u16,
    stack_pointer: u8,
    stack: [u16; 16],
    keyboard: [u16; 16],
    video: [u32; 64 * 32],
}

impl Default for Chip {
    fn default() -> Self {
        Chip {
            memory: Chip::init_memory(),
            i_reg: 0,
            general_purpose_reg: [0; 16],
            delay_reg: 0,
            audio_reg: 0,
            program_counter: START_ADDRESS,
            stack_pointer: 0,
            stack: [0; 16],
            keyboard: [0; 16],
            video: [0; 64 * 32],
        }
    }
}
impl Chip {
    fn load_rom(&mut self) -> io::Result<()> {
        let file = std::fs::File::open("test_opcode.ch8")?;
        let mut reader = BufReader::new(file);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;

        buffer
            .iter()
            .enumerate()
            .for_each(|(idx, _)| self.memory[START_ADDRESS as usize + idx] = buffer[idx]);
        return Ok(());
    }

    fn init_memory() -> [u8; 4096] {
        let mut mem = [0; 4096];
        let font_set = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80, // F
        ];
        for i in 0..80 {
            mem[i] = font_set[i]
        }
        return mem;
    }

    fn execute_opcode(&mut self, opcode: u16) {
        let vx = ((opcode & 0x0F00) >> 8) as usize;
        let vy = ((opcode & 0x00F0) >> 4) as usize;
        let constant = (opcode & 0x00FF) as u8;
        let nnn = opcode & 0x0FFF;
        let n = opcode & 0x000F;

        // get the 4 significatn bits
        match opcode & 0xF000 {
            0x0000 => match opcode & 0x00FF {
                0x00E0 => self.clear_display(),
                0x00EE => self.ret(),
                _ => panic!("Unknown opcode: {:04x}", opcode),
            },
            0x1000 => self.jmp_addr(),
            0x2000 => self.call_addr(),
            0x3000 => self.op_3xkk(vx, constant),
            0x4000 => self.op_4xkk(vx, constant),
            0x5000 => self.op_5xy0(vx, vy),
            0x6000 => self.op_6xkk(vx, constant),
            0x7000 => self.op_7xkk(vx, constant),
            0x8000 => match opcode & 0x000F {
                0x0000 => self.op_8xy1(vx, vy),
                0x0001 => self.op_8xy1(vx, vy),
                0x0002 => self.op_8xy2(vx, vy),
                0x0003 => self.op_8xy3(vx, vy),
                0x0004 => self.op_8xy4(vx, vy),
                0x0005 => self.op_8xy5(vx, vy),
                0x0006 => self.op_8xy6(vx, vy),
                0x0007 => self.op_8xy7(vx, vy),
                0x000E => self.op_8xyE(vx, vy),
                _ => panic!("Unknown opcode: {:04x}", opcode),
            },
            0x9000 => self.op_9xy0(vx, vy),
            0xA000 => self.op_Annn(nnn),

            _ => panic!("Unknown opcode: {:04x}", opcode),
        }
    }

    fn clear_display(&mut self) {
        // ...
    }

    fn ret(&mut self) {
        // ...
    }
    fn jmp_addr(&mut self) {}
    fn call_addr(&mut self) {}

    //  Skip next instruction if Vx = kk.
    fn op_3xkk(&mut self, vx: usize, constant: u8) {
        if self.general_purpose_reg[vx] == constant {
            self.program_counter += 2;
        }
    }
    //  Skip next instruction if Vx != kk.
    fn op_4xkk(&mut self, vx: usize, constant: u8) {
        if self.general_purpose_reg[vx] != constant {
            self.program_counter += 2;
        }
    }
    //  Skip next instruction if Vx = Vy.
    fn op_5xy0(&mut self, vx: usize, vy: usize) {
        if self.general_purpose_reg[vx] == self.general_purpose_reg[vy] {
            self.program_counter += 2;
        }
    }

    //  Set Vx = kk.
    fn op_6xkk(&mut self, vx: usize, constant: u8) {
        self.general_purpose_reg[vx] = constant;
    }

    //   Set Vx = Vx + kk.
    fn op_7xkk(&mut self, vx: usize, constant: u8) {
        self.general_purpose_reg[vx] += constant
    }

    //    Set Vx = Vy.
    fn op_8xy0(&mut self, vx: usize, vy: usize) {
        self.general_purpose_reg[vx] = self.general_purpose_reg[vy]
    }

    //  Set Vx = Vx OR Vy.
    fn op_8xy1(&mut self, vx: usize, vy: usize) {
        self.general_purpose_reg[vx] = self.general_purpose_reg[vx] | self.general_purpose_reg[vy]
    }

    fn op_8xy2(&mut self, vx: usize, vy: usize) {
        self.general_purpose_reg[vx] = self.general_purpose_reg[vx] & self.general_purpose_reg[vy]
    }
    //  Set Vx = Vx XOR Vy.
    fn op_8xy3(&mut self, vx: usize, vy: usize) {
        self.general_purpose_reg[vx] = self.general_purpose_reg[vx] ^ self.general_purpose_reg[vy]
    }

    //  8xy4 - ADD Vx, Vy
    fn op_8xy4(&mut self, vx: usize, vy: usize) {
        let (result, overflow) =
            self.general_purpose_reg[vx].overflowing_add(self.general_purpose_reg[vy]);
        self.general_purpose_reg[vx] = result;
        self.general_purpose_reg[15] = if overflow { 1 } else { 0 };
    }

    //  8xy5 - SUB Vx, Vy
    fn op_8xy5(&mut self, vx: usize, vy: usize) {
        self.general_purpose_reg[15] =
            if self.general_purpose_reg[vx] > self.general_purpose_reg[vy] {
                1
            } else {
                0
            };
        self.general_purpose_reg[vx] =
            self.general_purpose_reg[vx].wrapping_sub(self.general_purpose_reg[vy]);
    }

    //  Set Vx = Vx SHR 1.
    fn op_8xy6(&mut self, vx: usize, vy: usize) {
        self.general_purpose_reg[15] = self.general_purpose_reg[vx] & 0x01;
        self.general_purpose_reg[vx] >>= 2;
    }
    //  8xy7 - SUBN Vx, Vy
    fn op_8xy7(&mut self, vx: usize, vy: usize) {
        self.general_purpose_reg[15] =
            if self.general_purpose_reg[vy] > self.general_purpose_reg[vx] {
                1
            } else {
                0
            };
        self.general_purpose_reg[vx] =
            self.general_purpose_reg[vy].wrapping_sub(self.general_purpose_reg[vx]);
    }

    fn op_8xyE(&mut self, vx: usize, vy: usize) {
        self.general_purpose_reg[15] = self.general_purpose_reg[vx] & 0x01;
        self.general_purpose_reg[vx] <<= 2;
    }

    fn op_9xy0(&mut self, vx: usize, vy: usize) {
        if self.general_purpose_reg[vx] != self.general_purpose_reg[vy] {
            self.program_counter += 2;
        }
    }

    fn op_Annn(&mut self, nnn: u16) {
        self.i_reg = nnn;
    }
    fn op_Bnnn(&mut self, nnn: u16) {
        self.program_counter = nnn + self.general_purpose_reg[0] as u16;
    }
    fn op_Cxkk(&mut self, vx: usize, constant: u8) {
        self.general_purpose_reg[vx] = rand_byte() & constant;
    }

    fn op_Dxyn(&mut self, vx: usize, vy: usize, n: u8) {
        let xpos = self.general_purpose_reg[vx] as usize;
        let ypos = self.general_purpose_reg[vy] as usize;
        self.general_purpose_reg[15] = 0;

        for byte in 0..n {
            let y = (ypos + byte as usize) % 32;
            let sprite_byte = self.memory[self.i_reg as usize + byte as usize];

            for bit in 0..8 {
                let x = (xpos + bit) % 64;
                let sprite_pixel = (sprite_byte >> (7 - bit)) & 0x1;
                let video_pos = y * 64 + x;

                if sprite_pixel == 1 && self.video[video_pos] == 1 {
                    self.general_purpose_reg[15] = 1;
                }

                self.video[video_pos] ^= sprite_pixel as u32;
            }
        }
    }
}

fn rand_byte() -> u8 {
    let mut rng = rand::thread_rng();
    let random = rng.gen_range(0..=255);
    return random;
}

fn main() {
    let mut chip = Chip::default();
    let _ = chip.load_rom();
    // chip.execute_opcode(0x00F0);
    println!("{:?}", chip.memory);
}
