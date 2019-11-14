use std::fmt;

#[derive(Copy, Clone, Debug)]
pub struct V(pub u8);

impl From<u8> for V {
    fn from(v: u8) -> Self {
        Self(v)
    }
}

impl fmt::Display for V {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "v{:x}", self.0)
    }
}

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

impl fmt::Display for Instr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Instr::RcaCall(addr) => write!(f, "rca {:#X}", addr),
            Instr::ClearScreen => write!(f, "clear"),
            Instr::Return => write!(f, "ret"),
            Instr::Jump(addr) => write!(f, "jmp {:#X}", addr),
            Instr::Call(addr) => write!(f, "call {:#X}", addr),
            Instr::SkipIfEqConst(x, a) => write!(f, "skipeq {} {}", x, a),
            Instr::SkipIfNotEqConst(x, a) => write!(f, "skipneq {} {}", x, a),
            Instr::SkipIfEqReg(x, y) => write!(f, "skipeq {} {}", x, y),
            Instr::SkipIfNotEqReg(x, y) => write!(f, "skipneq {} {}", x, y),
            Instr::SetConst(x, a) => write!(f, "set {} {}", x, a),
            Instr::AddConst(x, a) => write!(f, "add {} {}", x, a),
            Instr::SetReg(x, y) => write!(f, "set {} {}", x, y),
            Instr::OrReg(x, y) => write!(f, "or {} {}", x, y),
            Instr::AndReg(x, y) => write!(f, "and {} {}", x, y),
            Instr::XorReg(x, y) => write!(f, "xor {} {}", x, y),
            Instr::AddReg(x, y) => write!(f, "add {} {}", x, y),
            Instr::SubReg(x, y) => write!(f, "sub {} {}", x, y),
            Instr::NegReg(x, y) => write!(f, "neg {} {}", x, y),
            Instr::ShrReg(x, y) => write!(f, "shr {} {}", x, y),
            Instr::ShlReg(x, y) => write!(f, "shl {} {}", x, y),
            Instr::SetIndex(addr) => write!(f, "seti {:#X}", addr),
            Instr::AddIndex(x) => write!(f, "addi {}", x),
            Instr::SetIndexFont(x) => write!(f, "setifont {}", x),
            Instr::JumpPlusV0(addr) => write!(f, "jumpaddv0 {}", addr),
            Instr::RandomAnd(x, a) => write!(f, "rand {} {:#X}", x, a),
            Instr::Draw(x, y, h) => write!(f, "draw {} {} {}", x, y, h),
            Instr::Load(x) => write!(f, "ld {}", x),
            Instr::Store(x) => write!(f, "str {}", x),
            Instr::StoreBcd(x) => write!(f, "strbcd {}", x),
            Instr::GetKey(x) => write!(f, "ldkey {}", x),
            Instr::SkipIfKey(x) => write!(f, "skipkeyeq {}", x),
            Instr::SkipIfNotKey(x) => write!(f, "skipkeyneq {}", x),
            Instr::GetDelay(x) => write!(f, "lddelay {}", x),
            Instr::SetDelay(x) => write!(f, "strdelay {}", x),
            Instr::SetSound(x) => write!(f, "strsound {}", x),
        }
    }
}
