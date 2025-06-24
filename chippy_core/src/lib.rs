use core::borrow;
use rand::random;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const RAM_SIZE: usize = 4096;
const REG_AMT: usize = 16;
const STACK_SIZE:usize = 16;
const NUM_OF_KEYS:usize = 16;
const START_ADDRESS:u16 = 0x200;

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
0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];


pub struct emu{
    pc:u16,
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_HEIGHT * SCREEN_WIDTH],
    v_registers: [u8; REG_AMT],
    i_register: u16,
    sp: u16,
    stack: [u16; STACK_SIZE],
    keys: [bool; NUM_OF_KEYS],
    dt: u8,
    st: u8
}

impl emu {
    pub fn new() -> Self{
        let mut new_emulator = Self {
            pc: START_ADDRESS,
            ram: [0;RAM_SIZE],
            screen: [false; SCREEN_HEIGHT * SCREEN_WIDTH],
            v_registers: [0; REG_AMT],
            i_register: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_OF_KEYS],
            dt: 0,
            st: 0
        };
        new_emulator.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
        new_emulator
    }

    pub fn get_display(&self) -> &[bool]{
        &self.screen
    }

    pub fn keypress(&mut self, idx:usize, pressed: bool){
        self.keys[idx] = pressed;
    }

    pub fn load(&mut self, data: &[u8]){
        let start = START_ADDRESS as usize;
        let end = (START_ADDRESS as usize) + data.len();
        self.ram[start..end].copy_from_slice(data);
    }

    pub fn push(&mut self, val:u16){
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    pub fn pop(&mut self) -> u16{
        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    pub fn tick(&mut self){
        // Fetch
        let op = self.fetch();
        // Decode and execute
        self.execute(op);
    }

    fn fetch(&mut self) -> u16{
        let higher_byte = self.ram[self.pc as usize] as u16;
        let lower_byte = self.ram[(self.pc + 1) as usize] as u16;
        let op = (higher_byte << 8) | lower_byte;
        self.pc += 2;
        op
    }

    pub fn tick_timers(&mut self){
        if self.dt > 0{
            self.dt -= 1;
        }
        if self.st > 0 {
            if self.st == 1{
                // beep
            }
            self.st -= 1;
        }
    }

    fn execute(&mut self, op: u16) {
        let digit1 = (op & 0xF000) >> 12;
        let digit2 = (op & 0x0F00) >> 8;
        let digit3 = (op & 0x00F0) >> 4;
        let digit4 = op & 0x000F;

        // Opcodes
        match (digit1, digit2, digit3, digit4) {
            // 0000 - Stop
            (0,0,0,0) => return,

            // 00E0 - Clear screen
            (0,0,0xE,0) => {
                self.screen = [false; SCREEN_HEIGHT * SCREEN_WIDTH]
            },

            // 00EE - Return from subroutine
            (0,0,0xE,0xE) => {
                let return_adress = self.pop();
                self.pc = return_adress;
            },

            // 1NNN - Jump 
            (1,_,_,_) => {
                let nnn = op & 0xFFF;
                self.pc = nnn;
            }

            // 2NNN - Jump to subroutine
            (2,_,_,_) => {
                let nnn = op & 0xFFF;
                self.push(self.pc);
                self.pc = nnn;
            }

            // 3XNN - Skip if VX == NN
            (3,_,_,_) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_registers[x] == nn {
                    self.pc += 2;
                }
            },

            // 4XNN - Skip if VX != NN
            (4,_,_,_) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_registers[x] != nn {
                    self.pc += 2;
                }
            },

            // 5XY0 - skip if VX == VY
            (5,_,_,0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                if self.v_registers[x] == self.v_registers[y]{
                    self.pc += 2;
                }
            },

            // 6XNN - VX = NN
            (6,_,_,_) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                self.v_registers[x] = nn;
            },

            // 7XNN - VX += NN
            (7,_,_,_) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                self.v_registers[x] = self.v_registers[x].wrapping_add(nn);
            }

            // 8XYO - VX = VY
            (8,_,_,0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_registers[x] = self.v_registers[y];
            }

            // 8XY1, 8XY2, 8XY3 - Bitwise operators
            (8,_,_,1) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_registers[x] |= self.v_registers[y];
            },

            (8,_,_,2) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_registers[x] &= self.v_registers[y];
            },

            (8,_,_,3) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_registers[x] ^= self.v_registers[y];
            },

            // 8XY4 - VX += VY
            (8,_,_,4) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, carry) = self.v_registers[x].overflowing_add(self.v_registers[y]);
                let new_vf = if carry {1} else {0};
                self.v_registers[x] = new_vx;
                self.v_registers[0xF] = new_vf;
            },

            // 8XY5 - VX -= VY
            (8,_,_,5) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, carry) = self.v_registers[x].overflowing_sub(self.v_registers[y]);
                let new_vf = if carry {0} else {1};
                self.v_registers[x] = new_vx;
                self.v_registers[0xF] = new_vf;
            },

            // 8XY6 - VX »= 1
            (8,_,_,6) => {
                let x = digit2 as usize;
                let lsb = self.v_registers[x] & 1;
                self.v_registers[x] >>= 1;
                self.v_registers[0xF] = lsb;
            },

            // 8XY7 - VX = VY - VX
            (8,_,_,7) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, borrow) = self.v_registers[y].overflowing_sub(self.v_registers[x]);
                let new_vf = if borrow {0} else {1};

                self.v_registers[x] = new_vx;
                self.v_registers[0xF] = new_vf;
            },

            // 8XYE - VX «= 1
            (8,_,_,E) => {
                let x = digit2 as usize;
                let msb = (self.v_registers[x] >> 7) & 1;
                self.v_registers[x] <<= 1;
                self.v_registers[0xF] = msb;
            },

            // 9XY0 - Skip if VX != VY
            (9,_,_,0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.v_registers[x] != self.v_registers[y]{
                    self.pc += 2;
                }
            },

            // ANNN - I = NNN
            (0xA,_,_,_) => {
                let nnn = op & 0xFFF;
                self.i_register = nnn;
            },

            // BNNN - Jump to V0 + NNN
            (0xB,_,_,_) => {
                let nnn = op & 0xFFF;
                self.pc = (self.v_registers[0] as u16) + nnn;

            },

            // CXNN - VX = rand() & NN
            (0xC,_,_,_) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                let rng: u8 = rand::random::<u8>();
                self.v_registers[x] = rng & nn;

            },

            // DXYN - Draw Sprite
            (0xD,_,_,_) => {
                // Getting (x, y) coords for the sprite
                let x_coord = self.v_registers[digit2 as usize] as u16;
                let y_coord = self.v_registers[digit3 as usize] as u16;

                // Last digit says height of sprite
                let num_rows = digit4;

                // Check if pixels flipped
                let mut flipped = false;
                // Iterate over each row of our sprite
                for y_line in 0..num_rows{
                    // determine address of row's data
                    let addr = self.i_register + y_line as u16;
                    let pixels = self.ram[addr as usize];

                    // iterate over colum in the row
                    for x_line in 0..8{
                        // mask to fetch current pixels bit only flip if its 1
                        if (pixels & (0b1000_0000 >> x_line)) != 0{
                            // sprites should wrap around screen
                            let x = (x_coord + x_line) as usize % SCREEN_WIDTH;
                            let y = (y_coord + y_line) as usize % SCREEN_HEIGHT;

                            // Get pixels indec for id screen array
                            let idx = x + SCREEN_WIDTH * y;
                            // check if we gotta flip pixel and set
                            flipped |= self.screen[idx];
                            self.screen[idx] ^= true;
                        }
                    }
                }
                // Populate vf register
                if flipped{
                    self.v_registers[0xF] = 1;
                } else {
                    self.v_registers[0xF] = 0;
                }

            },

            // EX9E - Skip if Key Pressed
            (0xE,_,9,0xE) => {
                let x = digit2 as usize;
                let vx = self.v_registers[x];
                let key = self.keys[vx as usize];
                if key{
                    self.pc += 2;
                }
            },

            // EXA1 - Skip if Key Not Pressed
            (0xE,_,0xA,1) => {
                let x = digit2 as usize;
                let vx = self.v_registers[x];
                let key = self.keys[vx as usize];
                if !key{
                    self.pc += 2;
                }
            },

            // FX07 - VX = DT
            (0xF,_,0,7) => {
                let x = digit2 as usize;
                self.v_registers[x] = self.dt
            },

            // FX0A - Wait for Key Press
            (0xF,_,0,0xA) => {
                let x = digit2 as usize;
                let mut pressed = false;
                for i in 0..self.keys.len(){
                    if self.keys[i]{
                        self.v_registers[x] = i as u8;
                        pressed = true;
                        break;
                    }
                }
                if ! pressed {
                    self.pc -= 2;
                }
            },

            // FX15 - DT = VX
            (0xF,_,1,5) => {
                let x = digit2 as usize;
                self.dt = self.v_registers[x];
            },

            // FX18 - ST = VX
            (0xF,_,1,8) => {
                let x = digit2 as usize;
                self.st = self.v_registers[x];
            },

            // FX1E - I += VX
            (0xF,_,1,0xE) => {
                let x = digit2 as usize;
                let vx = self.v_registers[x] as u16;
                self.i_register = self.i_register.wrapping_add(vx);
            },

            // FX29 - Set I to Font Address
            (0xF,_,2,9) => {
                let x = digit2 as usize;
                let c = self.v_registers[x] as u16;
                self.i_register = c * 5;
            },

            // FX33 - I = BCD of VX
            (0xF,_,3,3) => {
                let x = digit2 as usize;
                let vx = self.v_registers[x] as f32;
                
                // Festh hundreds digit by divinding by 100 and tossing decimal
                let hundreds = (vx / 100.0).floor() as u8;
                // Same with 10
                let tens = ((vx / 10.0) % 10.0).floor() as u8;
                let ones = (vx % 10.0) as u8;

                self.ram[self.i_register as usize] = hundreds;
                self.ram[(self.i_register + 1) as usize] = tens;
                self.ram[(self.i_register + 2) as usize] = ones;

            },

            // FX55 - Store V0 - VX into I
            (0xF,_,5,5) => {
                let x = digit2 as usize;
                let i = self.i_register as usize;
                for idx in 0..=x {
                    self.ram[i+idx] = self.v_registers[idx];
                }
            },

            // FX65 - Load I into V0 - VX
            (0xF,_,6,5) => {
                let x = digit2 as usize;
                let i = self.i_register as usize;
                for idx in 0..=x {
                    self.v_registers[idx] = self.ram[i + idx];
                }
            },



            // Unknown opcode (shouldn't happen)
            (_,_,_,_) => unimplemented!("Unimplemented opcode: {}", op)
        }   
    }

    pub fn reset(&mut self){
        self.pc = START_ADDRESS;
        self.ram = [0; RAM_SIZE];
        self.screen =[false; SCREEN_HEIGHT * SCREEN_WIDTH];
        self.v_registers = [0; REG_AMT];
        self.i_register = 0;
        self.sp = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_OF_KEYS];
        self.dt = 0;
        self.st = 0;
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }
}
