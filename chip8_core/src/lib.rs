use rand::random;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const RAM_SIZE: usize = 4096;
const NUM_REGS: usize = 16;
const NUM_KEYS: usize = 16;
const FONTSET_SIZE: usize = 80;

const FONTSET: [u8; FONTSET_SIZE] = [
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

pub struct Emu {
    pc: u16,                                      // Program Counter
    ram: [u8; RAM_SIZE],                          // 4KB of RAM
    screen: [bool; SCREEN_HEIGHT * SCREEN_WIDTH], // 64x32 monochrome screen
    v_regs: [u8; NUM_REGS],                       // V0 - VF
    i_reg: u16,                                   // I register
    sp: u16,                                      // Stack Pointer
    stack: [u16; 16],                             //
    keys: [bool; NUM_KEYS],
    dt: u8, // Delay Timer
    st: u8, // Sound Timer, emits a beep when it reaches 0
}

const START_ADDRES: u16 = 0x200; // Because the first 512 bytes are reserved for CHIP-8 interpreter

impl Emu {
    pub fn new() -> Self {
        let mut new_emu = Self {
            pc: START_ADDRES,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_HEIGHT * SCREEN_WIDTH],
            keys: [false; NUM_KEYS],
            v_regs: [0; NUM_REGS],
            i_reg: 0,
            sp: 0,
            stack: [0; 16],
            dt: 0,
            st: 0,
        };
        new_emu.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
        new_emu
    }
    pub fn reset(&mut self) {
        self.pc = START_ADDRES;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_HEIGHT * SCREEN_WIDTH];
        self.v_regs = [0; NUM_REGS];
        self.i_reg = 0;
        self.sp = 0;
        self.stack = [0; 16];
        self.dt = 0;
        self.st = 0;
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }
    pub fn tick(&mut self) {
        let op = self.fetch();
        // decode
        self.execute(op);
    }
    fn execute(&mut self, op: u16) {
        let digit1 = ((op & 0xF000) >> 12) as u8;
        let digit2 = ((op & 0x0F00) >> 8) as u8;
        let digit3 = ((op & 0x00F0) >> 4) as u8;
        let digit4 = (op & 0x000F) as u8;

        match (digit1, digit2, digit3, digit4) {
            (_, _, _, _) => unimplemented!("Opcode {:X} not implemented", op),
            (0x0, 0x0, 0x0, 0x0) => return, // 0000: Do Nothing
            (0, 0, 0xE, 0) => {
                // 00E0: Clear Screen
                self.screen = [false; SCREEN_HEIGHT * SCREEN_WIDTH];
            }
            (0, 0, 0xE, 0xE) => {
                // 00EE: Return from subroutine.
                let ret_addr = self.pop();
                self.pc = ret_addr;
            }
            (1, _, _, _) => {
                let nnn = op & 0x0FFF;
                self.pc = nnn;
            }
            (2, _, _, _) => {
                let nnn = op & 0x0FFF;
                self.push(self.pc);
                self.pc = nnn;
            }
            (3, _, _, _) => {
                // http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#3xkk
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_regs[x] == nn {
                    self.pc += 2;
                }
            }
            (4, _, _, _) => {
                //  Skip next if VX != NN
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_regs[x] != nn {
                    self.pc += 2;
                }
            }
            (5, _, _, 0) => {
                // Skip next if VX == VY
                let x = digit2 as usize;
                let y = digit3 as usize;

                if self.v_regs[x] == self.v_regs[y] {
                    self.pc += 2;
                }
            }
            (6, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                self.v_regs[x] = nn;
            }
            (7, _, _, _) => {
                // 7xnn - ADD Vx, byte
                // Set Vx = Vx + nn.
                // Adds the value nn to the value of register Vx, then stores the result in Vx.
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                self.v_regs[x] = self.v_regs[x].wrapping_add(nn); // May overflow, dont't panic. Use wrapping add instead.
            }
            (8, _, _, 0) => {
                // Set Vx = Vy.
                // Stores the value of register Vy in register Vx.
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_regs[x] = self.v_regs[y];
            }
            (8, _, _, 1) => {
                // 8xy1 - OR Vx, Vy
                // Set Vx = Vx OR Vy.
                // Performs a bitwise OR on the values of Vx and Vy, then stores the result in Vx. A bitwise OR compares the corrseponding bits from two values, and if either bit is 1, then the same bit in the result is also 1. Otherwise, it is 0.
                let x = digit2 as usize;
                let y = digit3 as usize;

                self.v_regs[x] |= self.v_regs[y];
            }
            (8, _, _, 2) => {
                // 8xy2 - AND Vx, Vy
                // Set Vx = Vx AND Vy.
                // Performs a bitwise AND on the values of Vx and Vy, then stores the result in Vx. A bitwise AND compares the corrseponding bits from two values, and if both bits are 1, then the same bit in the result is also 1. Otherwise, it is 0.
                let x = digit2 as usize;
                let y = digit3 as usize;

                self.v_regs[x] &= self.v_regs[y];
            }
            (8, _, _, 3) => {
                // 8xy3 - XOR Vx, Vy
                // Set Vx = Vx XOR Vy.
                // Performs a bitwise exclusive OR on the values of Vx and Vy, then stores the result in Vx. An exclusive OR compares the corrseponding bits from two values, and if the bits are not both the same, then the corresponding bit in the result is set to 1. Otherwise, it is 0.
                let x = digit2 as usize;
                let y = digit3 as usize;

                self.v_regs[x] ^= self.v_regs[y];
            }
            (8, _, _, 4) => {
                // 8xy4 - ADD Vx, Vy
                // Set Vx = Vx + Vy, set VF = carry.
                // The values of Vx and Vy are added together. If the result is greater than 8 bits (i.e., > 255,) VF is set to 1, otherwise 0. Only the lowest 8 bits of the result are kept, and stored in Vx.
                let x = digit2 as usize;
                let y = digit3 as usize;

                // Set carry 1 if overflow.
                let (new_vx, carry) = self.v_regs[x].overflowing_add(self.v_regs[y]);
                let new_vf = if carry { 1 } else { 0 };

                self.v_regs[x] = new_vx;
                self.v_regs[0xF] = new_vf;
            }
            (8, _, _, 5) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                let (new_vx, borrow) = self.v_regs[x].overflowing_sub(self.v_regs[y]);
                let new_vf = if borrow { 0 } else { 1 };

                self.v_regs[x] = new_vx;
                self.v_regs[0xF] = new_vf;
            }
            (8, _, _, 6) => {
                let x = digit2 as usize;
                let lsb = self.v_regs[x] & 1;
                self.v_regs[x] >> 1;
                self.v_regs[0xF] = lsb;
            }
            (8, _, _, _) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                let (new_vx, borrow) = self.v_regs[y].overflowing_sub(self.v_regs[x]);
                let new_vf = if borrow { 0 } else { 1 };

                self.v_regs[x] = new_vx;
                self.v_regs[0xF] = new_vf;
            }
            (8, _, _, 0xE) => {
                let x = digit2 as usize;
                let msb = (self.v_regs[x] >> 7) & 1;
                self.v_regs[x] <<= 1;
                self.v_regs[0xE] = msb;
            }
            (9, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                if self.v_regs[x] != self.v_regs[y] {
                    self.pc += 2;
                }
            }
            (0xA, _, _, _) => {
                let nnn = op & 0xFFF;
                self.i_reg = nnn;
            }
            (0xB, _, _, _) => {
                let nnn = op & 0xFFF;
                self.pc = (self.v_regs[0] as u16) + nnn;
            }
            (0xC, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                let rng: u8 = random();
                self.v_regs[x] = rng & nn;
            }
            (0xD, _, _, _) => {
                // Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
                let x_coord = self.v_regs[digit2 as usize] as u16;
                let y_coord = self.v_regs[digit3 as usize] as u16;
                let num_rows = digit4 as u16;
                let mut flipped = false;

                for y_line in 0..num_rows {
                    let addr = self.i_reg + y_line as u16;
                    let pixels = self.ram[addr as usize];
                    for x_line in 0..8 {
                        if (pixels & (0b1000_0000 >> x_line)) != 0 {
                            let x = (x_coord + x_line) as usize % SCREEN_HEIGHT;
                            let y = (y_coord + y_line) as usize % SCREEN_HEIGHT;

                            let idx = x + SCREEN_WIDTH * y;
                            flipped |= self.screen[idx];
                            self.screen[idx] ^= true;
                        }
                    }
                }
                if flipped {
                    self.v_regs[0xF] = 1;
                } else {
                    self.v_regs[0xF] = 0;
                }
            }
            (0xE, _, 9, 0xE) => {
                let x = digit2 as usize;
                let vx = self.v_regs[x];
                let key = self.keys[vx as usize];
                if key {
                    self.pc += 2;
                }
            }
            (0xE, _, 0xA, 1) => {
                let x = digit2 as usize;
                let vx = self.v_regs[x];
                let key = self.keys[vx as usize];
                if !key {
                    self.pc += 2;
                }
            }
            (0xF, _, 0, 7) => {
                let x = digit2 as usize;
                self.v_regs[x] = self.dt;
            }
            (0xF, _, 0, 0xA) => {
                let x = digit2 as usize;
                let mut pressed = false;

                for i in 0..self.keys.len() {
                    if self.keys[i] {
                        self.v_regs[x] = i as u8;
                        pressed = true;
                        break;
                    }
                }
                if !pressed {
                    // Redo operator.
                    self.pc -= 2;
                }
            }
            (0xF, _, 1, 5) => {
                let x = digit2 as usize;
                self.dt = self.v_regs[x];
            }
            (0xF, _, 1, 8) => {
                let x = digit2 as usize;
                self.st = self.v_regs[x]
            }
            (0xF, _, 1, 0xE) => {
                //Set I = I + Vx.
                // The values of I and Vx are added, and the results are stored in I.
                let x = digit2 as usize;
                let vx = self.v_regs[x] as u16;
                self.i_reg = self.i_reg.wrapping_add(vx);
            }
            (0xF, _, 2, 9) => {
                let x = digit2 as usize;
                let c = self.v_regs[x] as u16;
                self.i_reg = c * 5;
            }
            (0xF, _, 3, 3) => {
                let x = digit2 as usize;
                let vx = self.v_regs[x] as f32;

                let hundreds = (vx / 100.0).floor() as u8;
                let tens = ((vx / 10.0) % 10.0).floor() as u8;
                let ones = (vx % 10.0) as u8;

                self.ram[self.i_reg as usize] = hundreds;
                self.ram[(self.i_reg + 1) as usize] = tens;
                self.ram[(self.i_reg + 2) as usize] = ones;
            }
            (0xF, _, 5, 5) => {
                let x = digit2 as usize;
                let i = self.i_reg as usize;
                for idx in 0..=x {
                    self.ram[i + idx] = self.v_regs[idx];
                }
            }
            (0xF, _, 6, 5) => {
                let x = digit2 as usize;
                let i = self.i_reg as usize;

                for idx in 0..=x {
                    self.v_regs[idx] = self.ram[i + idx];
                }
            }
        }
    }
    pub fn tick_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }
        if self.st > 0 {
            self.st -= 1;
        }
    }
    pub fn fetch(&mut self) -> u16 {
        let higher = self.ram[self.pc as usize] as u16;
        let lower = self.ram[(self.pc + 1) as usize] as u16;
        let op = (higher << 8) | lower;
        self.pc += 2;
        op
    }
    fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val; // Push value to stack
        self.sp += 1; // Increment stack pointer
    }
    fn pop(&mut self) -> u16 {
        self.sp -= 1; // Decrement stack pointer
        self.stack[self.sp as usize] // Return value from stack
    }
    pub fn get_display(&self) -> &[bool] {
        &self.screen
    }
    pub fn keypress(&mut self, idx: usize, pressed: bool) {
        self.keys[idx] = pressed;
    }

    pub fn load(&mut self, data: &[u8]) {
        let start = START_ADDRES as usize;
        let end = (START_ADDRES as usize) + data.len();
        self.ram[start..end].copy_from_slice(data);
    }
}
