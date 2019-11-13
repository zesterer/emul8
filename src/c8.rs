use std::time::Duration;
use crate::font::FONT;

pub const SCREEN_SIZE: (usize, usize) = (64, 32);
pub const KEY_COUNT: usize = 16;
const PROGRAM_START: usize = 0x200;

#[derive(Debug)]
pub enum Error {
    NoReturnAddr,
    NoSuchRcaCall(u16),
    InvalidInstr([u8; 4]),
    OutOfBounds,
}

#[derive(Copy, Clone, Debug)]
pub struct V(u8);

#[derive(Copy, Clone, Debug)]
pub enum Instr {
    RcaCall(u16),
    ClearScreen,
    Return,
    Jump(u16),
    Call(u16),
    SkipIfEqConst(V, u8),
    SkipIfNotEqConst(V, u8),
    SkipIfEqReg(V, V),
    SkipIfNotEqReg(V, V),
    SetConst(V, u8),
    AddConst(V, u8),
    SetReg(V, V),
    OrReg(V, V),
    AndReg(V, V),
    XorReg(V, V),
    AddReg(V, V),
    SubReg(V, V),
    NegReg(V, V),
    ShrReg(V, V),
    ShlReg(V, V),
    SetIndex(u16),
    AddIndex(V),
    SetIndexFont(V),
    JumpPlusV0(u16),
    RandomAnd(V, u8),
    Draw(V, V, u8),
    Load(V),
    Store(V),
    StoreBcd(V),
    GetKey(V),
    SkipIfKey(V),
    SkipIfNotKey(V),
    GetDelay(V),
    SetDelay(V),
    SetSound(V),
}

pub struct C8 {
    v: [u8; 16],
    i: u16,
    pc: u16,
    stack: Vec<u16>,
    timer: [u8; 2],
    mem: [u8; 4096],
    screen: [bool; SCREEN_SIZE.0 * SCREEN_SIZE.1],
    keys: [bool; KEY_COUNT],
    last_pulse: u64,
    exec_time: u64,
}

impl Default for C8 {
    fn default() -> Self {
        let mut mem = [0; 4096];
        for (i, b) in FONT.iter().enumerate() {
            mem[i] = *b;
        }

        Self {
            v: [0; 16],
            i: 0,
            pc: PROGRAM_START as u16,
            stack: Vec::with_capacity(24),
            timer: [0; 2],
            mem,
            screen: [false; SCREEN_SIZE.0 * SCREEN_SIZE.1],
            keys: [false; KEY_COUNT],
            last_pulse: 0,
            exec_time: 0,
        }
    }
}

impl C8 {
    pub fn display_mem(&self) {
        let row_width = 16;
        for row_addr in (0..4096).step_by(row_width) {
            print!("0x{:04X} |", row_addr);
            for i in 0..row_width {
                print!(" {:02X}", self.mem[row_addr + i]);
            }
            println!("");
        }
    }

    pub fn display_regs(&self) {
        for (i, v) in self.v.iter().enumerate() {
            println!("v{:x} = 0x{:02X}", i, v);
        }
        println!("i = 0x{:04X}", self.i);
        println!("pc = 0x{:04X}", self.pc);
    }

    pub fn load(&mut self, bytes: &[u8]) {
        for (i, b) in bytes.iter().enumerate() {
            self.mem[PROGRAM_START + i] = *b;
        }
    }

    pub fn set_keys(&mut self, keys: [bool; KEY_COUNT]) {
        self.keys = keys;
    }

    pub fn screen(&self) -> &[bool; SCREEN_SIZE.0 * SCREEN_SIZE.1] {
        &self.screen
    }

    fn v(&self, v: V) -> u8 {
        self.v[v.0 as usize]
    }

    fn v_mut(&mut self, v: V) -> &mut u8 {
        &mut self.v[v.0 as usize]
    }

    fn set_flag(&mut self, set: bool) {
        self.v[0xF] = if set { 1 } else { 0 }
    }

    fn draw_sprite(&mut self, x: u8, y: u8, h: u8, pixels: u16) -> bool {
        let mut intersection = false;
        for row in 0..h {
            for col in 0..8 {
                let idx = (y.wrapping_add(row) as usize % SCREEN_SIZE.1) * SCREEN_SIZE.0
                    + (x.wrapping_add(col) as usize % SCREEN_SIZE.0);
                let spr_px = (self.mem[pixels as usize + row as usize] >> (7 - col)) & 1 != 0;
                self.screen[idx] ^= spr_px;
                if spr_px && !self.screen[idx] {
                    intersection = true;
                }
            }
        }
        intersection
    }

    fn fetch(&self, pc: u16) -> Result<[u8; 4], Error> {
        let op = self.mem.get(pc as usize..pc as usize + 2).ok_or(Error::OutOfBounds)?;
        Ok([
            (op[0] >> 4) & 0xF, op[0] & 0xF,
            (op[1] >> 4) & 0xF, op[1] & 0xF,
        ])
    }

    fn decode(&self, opcode: [u8; 4]) -> Result<Instr, Error> {
        Ok(match opcode {
            [0x0, 0x0, 0xE, 0x0] => Instr::ClearScreen,
            [0x0, 0x0, 0xE, 0xE] => Instr::Return,
            [0x0, c, b, a] => Instr::RcaCall(u4s_to_u16(a, b, c, 0)),
            [0x1, c, b, a] => Instr::Jump(u4s_to_u16(a, b, c, 0)),
            [0x2, c, b, a] => Instr::Call(u4s_to_u16(a, b, c, 0)),
            [0x3, x, b, a] => Instr::SkipIfEqConst(V(x), u4s_to_u8(a, b)),
            [0x4, x, b, a] => Instr::SkipIfNotEqConst(V(x), u4s_to_u8(a, b)),
            [0x5, x, y, 0x0] => Instr::SkipIfEqReg(V(x), V(y)),
            [0x6, x, b, a] => Instr::SetConst(V(x), u4s_to_u8(a, b)),
            [0x7, x, b, a] => Instr::AddConst(V(x), u4s_to_u8(a, b)),
            [0x8, x, y, 0x0] => Instr::SetReg(V(x), V(y)),
            [0x8, x, y, 0x1] => Instr::OrReg(V(x), V(y)),
            [0x8, x, y, 0x2] => Instr::AndReg(V(x), V(y)),
            [0x8, x, y, 0x3] => Instr::XorReg(V(x), V(y)),
            [0x8, x, y, 0x4] => Instr::AddReg(V(x), V(y)),
            [0x8, x, y, 0x5] => Instr::SubReg(V(x), V(y)),
            [0x8, x, y, 0x6] => Instr::ShrReg(V(x), V(y)),
            [0x8, x, y, 0x7] => Instr::NegReg(V(x), V(y)),
            [0x8, x, y, 0xE] => Instr::ShlReg(V(x), V(y)),
            [0x9, x, y, 0x0] => Instr::SkipIfNotEqReg(V(x), V(y)),
            [0xA, c, b, a] => Instr::SetIndex(u4s_to_u16(a, b, c, 0)),
            [0xB, c, b, a] => Instr::JumpPlusV0(u4s_to_u16(a, b, c, 0)),
            [0xC, x, b, a] => Instr::RandomAnd(V(x), u4s_to_u8(a, b)),
            [0xD, x, y, h] => Instr::Draw(V(x), V(y), h),
            [0xE, x, 0x9, 0xE] => Instr::SkipIfKey(V(x)),
            [0xE, x, 0xA, 0x1] => Instr::SkipIfNotKey(V(x)),
            [0xF, x, 0x0, 0x7] => Instr::GetDelay(V(x)),
            [0xF, x, 0x0, 0xA] => Instr::GetKey(V(x)),
            [0xF, x, 0x1, 0x5] => Instr::SetDelay(V(x)),
            [0xF, x, 0x1, 0x8] => Instr::SetSound(V(x)),
            [0xF, x, 0x1, 0xE] => Instr::AddIndex(V(x)),
            [0xF, x, 0x2, 0x9] => Instr::SetIndexFont(V(x)),
            [0xF, x, 0x3, 0x3] => Instr::StoreBcd(V(x)),
            [0xF, x, 0x5, 0x5] => Instr::Store(V(x)),
            [0xF, x, 0x6, 0x5] => Instr::Load(V(x)),
            opcode => return Err(Error::InvalidInstr(opcode)),
        })
    }

    fn execute(&mut self, instr: Instr) -> Result<(), Error> {
        let mut step = 2;
        match instr {
            Instr::RcaCall(addr) => return Err(Error::NoSuchRcaCall(addr)),
            Instr::ClearScreen => self.screen = [false; SCREEN_SIZE.0 * SCREEN_SIZE.1],
            Instr::Return => self.pc = self.stack.pop().ok_or(Error::NoReturnAddr)?,
            Instr::Jump(addr) => {
                self.pc = addr;
                step = 0;
            },
            Instr::Call(addr) => {
                self.stack.push(self.pc);
                self.pc = addr;
                step = 0;
            },
            Instr::SkipIfEqConst(x, a) => step = if self.v(x) == a { 4 } else { 2 },
            Instr::SkipIfNotEqConst(x, a) => step = if self.v(x) != a { 4 } else { 2 },
            Instr::SkipIfEqReg(x, y) => step = if self.v(x) == self.v(y) { 4 } else { 2 },
            Instr::SkipIfNotEqReg(x, y) => step = if self.v(x) != self.v(y) { 4 } else { 2 },
            Instr::SetConst(x, a) => *self.v_mut(x) = a,
            Instr::AddConst(x, a) => *self.v_mut(x) = self.v(x).wrapping_add(a),
            Instr::SetReg(x, y) => *self.v_mut(x) = self.v(y),
            Instr::OrReg(x, y) => *self.v_mut(x) |= self.v(y),
            Instr::AndReg(x, y) => *self.v_mut(x) &= self.v(y),
            Instr::XorReg(x, y) => *self.v_mut(x) ^= self.v(y),
            Instr::AddReg(x, y) => {
                let (val, overflow) = self.v(x).overflowing_add(self.v(y));
                *self.v_mut(x) = val;
                self.set_flag(overflow);
            },
            Instr::SubReg(x, y) => {
                let (val, overflow) = self.v(x).overflowing_sub(self.v(y));
                *self.v_mut(x) = val;
                self.set_flag(!overflow);
            },
            Instr::NegReg(x, y) => {
                let (val, overflow) = self.v(y).overflowing_sub(self.v(x));
                *self.v_mut(x) = val;
                self.set_flag(!overflow);
            },
            Instr::ShrReg(x, y) => {
                let v = self.v(y);
                *self.v_mut(x) = v >> 1;
                self.set_flag(v & 1 == 1);
            },
            Instr::ShlReg(x, y) => {
                let v = self.v(y);
                *self.v_mut(x) = v << 1;
                self.set_flag((v >> 7) & 1 == 1);
            },
            Instr::SetIndex(addr) => self.i = addr,
            Instr::AddIndex(x) => {
                let vx = self.v(x) as u16;
                self.i = self.i.wrapping_add(vx);
                self.set_flag((self.i as usize + vx as usize) > 0xFFF);
            },
            Instr::SetIndexFont(x) => self.i = self.v(x) as u16 * 5,
            Instr::JumpPlusV0(addr) => {
                self.pc = addr + self.v[0] as u16;
                step = 0;
            },
            Instr::RandomAnd(x, a) => *self.v_mut(x) = rand::random::<u8>() & a,
            Instr::Draw(x, y, h) => {
                let intersection = self.draw_sprite(self.v(x), self.v(y), h, self.i);
                self.set_flag(intersection);
            },
            Instr::Load(x) => for (i, v) in self.v.iter_mut().take(x.0 as usize + 1).enumerate() {
                *v = self.mem[self.i as usize + i];
            },
            Instr::Store(x) => for (i, v) in self.v.iter().take(x.0 as usize + 1).enumerate() {
                self.mem[self.i as usize + i] = *v;
            },
            Instr::StoreBcd(x) => {
                (0..3).fold(self.v(x), |v, i| {
                    self.mem[self.i as usize + i] = v % 10;
                    v / 10
                });
            },
            Instr::GetKey(x) => match self.keys.iter().enumerate().find(|(_, k)| **k) {
                Some((i, _)) => *self.v_mut(x) = i as u8,
                None => step = 0,
            },
            Instr::SkipIfKey(x) => step = if self.keys[self.v(x) as usize] { 4 } else { 2 },
            Instr::SkipIfNotKey(x) => step = if !self.keys[self.v(x) as usize] { 4 } else { 2 },
            Instr::GetDelay(x) => *self.v_mut(x) = self.timer[0],
            Instr::SetDelay(x) => self.timer[0] = self.v(x),
            Instr::SetSound(x) => self.timer[1] = self.v(x),
        }

        self.pc += step;

        Ok(())
    }

    pub fn tick(&mut self, dur: Duration) -> Result<(), Error> {
        // Update timers
        self.exec_time += dur.as_nanos() as u64;
        if self.exec_time - self.last_pulse > 1000000 / 60 {
            self.timer.iter_mut().for_each(|t| *t = t.saturating_sub(1));
            self.last_pulse = self.exec_time;
        }

        let opcode = self.fetch(self.pc)?;
        let instr = self.decode(opcode)?;
        self.execute(instr)
        //println!("0x{:04X} :: {:X?} ({:X?}) => {:X?}", self.pc, opcode, instr, result);
    }
}

fn u4s_to_u16(a: u8, b: u8, c: u8, d: u8) -> u16 {
    a as u16 | (b as u16) << 4 | (c as u16) << 8 | (d as u16) << 12
}

fn u4s_to_u8(a: u8, b: u8) -> u8 {
    a | (b << 4)
}
