pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const RAM_SIZE: usize = 4096;
const NUM_REGS: usize = 16;
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
    stack: [u16; 16],                             // Stack
    dt: u8,                                       // Delay Timer
    st: u8,                                       // Sound Timer, emits a beep when it reaches 0
}

const START_ADDRES: u16 = 0x200; // Because the first 512 bytes are reserved for CHIP-8 interpreter

impl Emu {
    pub fn new() -> Self {
        let mut new_emu = Self {
            pc: START_ADDRES,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_HEIGHT * SCREEN_WIDTH],
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
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_regs[x] != nn {
                    self.pc +=2;
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
}
