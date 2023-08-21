use log::debug;
use rand::{rngs::ThreadRng, Rng};
use std::io::{self, BufReader, Read};
const START_ADDRESS: u16 = 0x200;

pub struct Chip {
    pub memory: [u8; 4096], // 4kb
    pub general_purpose_reg: [u8; 16],
    pub i_reg: u16,
    pub delay_reg: u8, // 8bit
    pub audio_reg: u8,
    pub program_counter: u16,
    pub stack_pointer: u8,
    pub stack: [u16; 16],
    pub keyboard: [u16; 16],
    pub video: [u32; 64 * 32],
    pub rng: ThreadRng,
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
            rng: rand::thread_rng(),
        }
    }
}
impl Chip {
    pub fn load_rom(&mut self, file: &str) -> io::Result<()> {
        let file = std::fs::File::open(file)?;
        // let file = std::fs::File::open("1-chip8-logo")?;

        let mut reader = BufReader::new(file);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;

        buffer
            .iter()
            .enumerate()
            .for_each(|(idx, &byte)| self.memory[START_ADDRESS as usize + idx] = byte);

        return Ok(());
    }

    pub fn cycle(&mut self) {
        // fetch
        let high = self.memory[self.program_counter as usize];
        // debug!("HIGH {:X}", high);
        let low = self.memory[self.program_counter as usize + 1];
        // debug!("LOW {:?}", self.memory);
        self.program_counter += 2;

        // decode
        let opcode = ((high as u16) << 8) | (low as u16);
        // execute

        self.execute_opcode(opcode);
        if self.delay_reg > 0 {
            self.delay_reg -= 1;
        }
        if self.audio_reg > 0 {
            self.audio_reg -= 1;
        }
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

        debug!("EXECUTING OP CODE {:X}", opcode);
        // get the 4 significatn bits
        match opcode & 0xF000 {
            0x0000 => match opcode & 0x00FF {
                0x00E0 => self.clear_display(),
                0x00EE => self.ret(),
                _ => panic!("Unknown opcode: {:04x}", opcode),
            },
            0x1000 => self.jmp_addr(nnn),
            0x2000 => self.call_addr(nnn),
            0x3000 => self.op_3xkk(vx, constant),
            0x4000 => self.op_4xkk(vx, constant),
            0x5000 => self.op_5xy0(vx, vy),
            0x6000 => self.op_6xkk(vx, constant),
            0x7000 => self.op_7xkk(vx, constant),
            0x8000 => match opcode & 0x000F {
                0x0000 => self.op_8xy0(vx, vy),
                0x0001 => self.op_8xy1(vx, vy),
                0x0002 => self.op_8xy2(vx, vy),
                0x0003 => self.op_8xy3(vx, vy),
                0x0004 => self.op_8xy4(vx, vy),
                0x0005 => self.op_8xy5(vx, vy),
                0x0006 => self.op_8xy6(vx),
                0x0007 => self.op_8xy7(vx, vy),
                0x000E => self.op_8xye(vx),
                _ => panic!("Unknown opcode: {:04x}", opcode),
            },
            0x9000 => self.op_9xy0(vx, vy),
            0xA000 => self.op_annn(nnn),
            0xB000 => self.op_bnnn(nnn),
            0xC000 => self.op_cxkk(vx, constant),
            0xD000 => self.op_dxyn(vx, vy, n),
            0xE000 => match opcode & 0x00FF {
                0x009E => self.op_ex9e(vx),
                0x00A1 => self.op_exa1(vx),
                _ => panic!("Unknown opcode: {:04x}", opcode),
            },
            0xF000 => match opcode & 0x00FF {
                0x0007 => self.op_fx07(vx),
                0x000A => self.op_fx0a(vx),
                0x0015 => self.op_fx15(vx),
                0x0018 => self.op_fx18(vx),
                0x001E => self.op_fx1e(vx),
                0x0029 => self.op_fx29(vx),
                0x0033 => self.op_fx33(vx),
                0x0055 => self.op_fx55(vx),
                0x0065 => self.op_fx65(vx),
                _ => panic!("Unknown opcode: {:04x}", opcode),
            },

            _ => panic!("Unknown opcode: {:04x}", opcode),
        }
    }

    fn clear_display(&mut self) {
        println!("clearing display");
        for pixel in self.video.iter_mut() {
            *pixel = 0;
        }
    }

    fn ret(&mut self) {
        self.stack_pointer -= 1;
        self.program_counter = self.stack[self.stack_pointer as usize] as u16;
    }
    fn jmp_addr(&mut self, nnn: u16) {
        self.program_counter = nnn;
    }
    fn call_addr(&mut self, nnn: u16) {
        self.stack[self.stack_pointer as usize] = self.program_counter;
        self.stack_pointer += 1;
        self.program_counter = nnn;
    }

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
        let (result, _) = self.general_purpose_reg[vx].overflowing_add(constant);
        self.general_purpose_reg[vx] = result;
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

    // 8xy5 - SUB Vx, Vy
    fn op_8xy5(&mut self, vx: usize, vy: usize) {
        let vxval = self.general_purpose_reg[vx];
        let vyval = self.general_purpose_reg[vy];
        self.general_purpose_reg[vx] = vxval.wrapping_sub(vyval); // Do the subtraction first
        self.general_purpose_reg[15] = if vxval > vyval { 1 } else { 0 }; // Then set vF
    }

    // Set Vx = Vx SHR 1.
    fn op_8xy6(&mut self, vx: usize) {
        let vxval = self.general_purpose_reg[vx];
        self.general_purpose_reg[vx] = vxval >> 1;
        self.general_purpose_reg[15] = vxval & 0x01;
    }

    // 8xy7 - SUBN Vx, Vy
    fn op_8xy7(&mut self, vx: usize, vy: usize) {
        let vyval = self.general_purpose_reg[vy];
        let vxval = self.general_purpose_reg[vx];
        self.general_purpose_reg[vx] = vyval.wrapping_sub(vxval); // Do the subtraction first
        self.general_purpose_reg[15] = if vyval > vxval { 1 } else { 0 }; // Then set vF
    }

    // Set Vx = Vx SHL 1.
    fn op_8xye(&mut self, vx: usize) {
        let vxval = self.general_purpose_reg[vx];
        self.general_purpose_reg[vx] = vxval << 1;
        self.general_purpose_reg[15] = (vxval & 0x80) >> 7;
    }

    //  9xy0 - SNE Vx, Vy
    fn op_9xy0(&mut self, vx: usize, vy: usize) {
        if self.general_purpose_reg[vx] != self.general_purpose_reg[vy] {
            self.program_counter += 2;
        }
    }
    // Set I = nnn.
    fn op_annn(&mut self, nnn: u16) {
        self.i_reg = nnn;
    }

    //  Jump to location nnn + V0.
    fn op_bnnn(&mut self, nnn: u16) {
        self.program_counter = nnn + self.general_purpose_reg[0] as u16;
    }

    // Set Vx = random byte AND kk.
    fn op_cxkk(&mut self, vx: usize, constant: u8) {
        self.general_purpose_reg[vx] = rand_byte(&mut self.rng) & constant;
    }

    //  Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
    fn op_dxyn(&mut self, vx: usize, vy: usize, n: u16) {
        debug!("DRAWING TO CANVAS");
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

                // collision
                if sprite_pixel == 1 && self.video[video_pos] == 1 {
                    self.general_purpose_reg[15] = 1;
                }

                self.video[video_pos] ^= sprite_pixel as u32;
            }
        }
        // debug!("{:?}", self.video)
    }

    // Skip next instruction if key with the value of Vx is pressed.
    fn op_ex9e(&mut self, vx: usize) {
        let key = self.general_purpose_reg[vx];
        if self.keyboard[key as usize] == 1 {
            self.program_counter += 2;
        }
    }

    //  Skip next instruction if key with the value of Vx is not pressed.
    fn op_exa1(&mut self, vx: usize) {
        let key = self.general_purpose_reg[vx];
        if self.keyboard[key as usize] != 1 {
            self.program_counter += 2;
        }
    }

    //  Set Vx = delay timer value.
    fn op_fx07(&mut self, vx: usize) {
        self.general_purpose_reg[vx] = self.delay_reg
    }

    //  Wait for a key press, store the value of the key in Vx.
    fn op_fx0a(&mut self, vx: usize) {
        let pressed_key: Option<usize> = self.keyboard.iter().position(|&x| x == 1);

        match pressed_key {
            Some(key) => {
                self.general_purpose_reg[vx] = key as u8;
            }
            None => {
                self.program_counter -= 2;
            }
        }
    }

    // Set delay timer = Vx.
    fn op_fx15(&mut self, vx: usize) {
        self.delay_reg = self.general_purpose_reg[vx];
    }

    // Set sound timer = Vx.
    fn op_fx18(&mut self, vx: usize) {
        self.audio_reg = self.general_purpose_reg[vx];
    }

    // Set I = I + Vx.
    fn op_fx1e(&mut self, vx: usize) {
        self.i_reg += self.general_purpose_reg[vx] as u16;
    }

    //  Set I = location of sprite for digit Vx.
    fn op_fx29(&mut self, vx: usize) {
        let digit = self.general_purpose_reg[vx];
        self.i_reg = 5 * digit as u16;
    }

    //  Store BCD representation of Vx in memory locations I, I+1, and I+2.
    fn op_fx33(&mut self, vx: usize) {
        let mut value = self.general_purpose_reg[vx];
        // Ones-place
        self.memory[self.i_reg as usize + 2] = value % 10;
        value /= 10;

        // Tens-place
        self.memory[self.i_reg as usize + 1] = value % 10;
        value /= 10;

        // Hundreds-place
        self.memory[self.i_reg as usize] = value % 10;
    }

    //  Store registers V0 through Vx in memory starting at location I.
    fn op_fx55(&mut self, vx: usize) {
        for i in 0..=vx {
            self.memory[self.i_reg as usize + i] = self.general_purpose_reg[i];
        }
    }

    //  Read registers V0 through Vx from memory starting at location I.
    fn op_fx65(&mut self, vx: usize) {
        for i in 0..=vx {
            self.general_purpose_reg[i] = self.memory[self.i_reg as usize + i]
        }
    }
}

fn rand_byte(rng: &mut ThreadRng) -> u8 {
    let random = rng.gen_range(0..=255);
    return random;
}
