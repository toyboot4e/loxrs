use std::{io::prelude::*, mem::transmute};

/// Operation code to the virtual machine
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub enum OpCode {
    OpReturn,

    /// Followed by a byte index
    OpConst8,
    /// Followed by a two bytes index
    OpConst16,

    OpNegate,
    OpAdd,
    OpSub,
    OpMul,
    OpDiv,
}

impl Into<u8> for OpCode {
    fn into(self) -> u8 {
        self as u8
    }
}

/// Constant value (it's f64 for now)
pub type Value = f64;

/// Chunk of instructions ([`OpCode`]s)
#[derive(Debug, Clone)]
pub struct ChunkData {
    /// Upcated bytes
    bytes: Vec<u8>,
    /// Constant values stored
    consts: Vec<Value>,
}

impl ChunkData {
    pub fn new() -> Self {
        Self {
            bytes: Vec::new(),
            consts: Vec::new(),
        }
    }
}

/// Accessors
impl ChunkData {
    #[inline(always)]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    #[inline(always)]
    pub fn consts(&mut self) -> &Vec<Value> {
        &self.consts
    }

    #[inline(always)]
    pub fn push_const(&mut self, value: Value) {
        self.consts.push(value)
    }

    #[inline(always)]
    pub fn read_opcode(&self, ix: usize) -> OpCode {
        unsafe { transmute(self.bytes[ix]) }
    }

    #[inline(always)]
    pub fn read_u8(&self, ix: usize) -> u8 {
        self.bytes[ix]
    }

    #[inline(always)]
    pub fn read_u16(&self, ix: usize) -> u16 {
        ((self.bytes[ix] as u16) << 8) | (self.bytes[ix + 1] as u16)
    }
}

/// Write
impl ChunkData {
    #[inline(always)]
    pub fn push_code(&mut self, code: OpCode) {
        self.bytes.push(code as u8);
    }

    #[inline(always)]
    pub fn push_ix_u8(&mut self, x: u8) {
        self.bytes.push(OpCode::OpConst8 as u8);
        self.bytes.push(x);
    }

    #[inline(always)]
    pub fn push_ix_u16(&mut self, x: u16) {
        self.bytes.push(OpCode::OpConst16 as u8);
        // higher 8 bits
        self.bytes.push((x >> 8) as u8);
        // lower 8 bits
        self.bytes.push(x as u8);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn test_memory_sizes() {
        assert_eq!(1, size_of::<OpCode>());
    }
}

// --------------------------------------------------------------------------------
// debug & tests

/// Extends `ChunkData` i.e. `Vec<OpCode>`
pub trait DebugPrint {
    fn debug_print(&self, title: &str);
}

impl DebugPrint for ChunkData {
    /// Disassembles `ChunkData`
    fn debug_print(&self, title: &str) {
        let out = std::io::stdout();
        let out = &mut out.lock();

        writeln!(out, "== {} ==", title).unwrap();
        use OpCode::*;

        // TODO: consider using StdoutLock
        let mut iter = self.bytes.iter().enumerate();
        while let Some((offset, &byte)) = iter.next() {
            let code: OpCode = unsafe { transmute(byte) };
            match code {
                OpConst8 => {
                    let ix = self.read_u8(offset + 1);
                    writeln!(
                        out,
                        "1 byte: idx =  {}, value = {:?}",
                        ix,
                        self.consts.get(ix as usize)
                    )
                    .unwrap();
                    iter.next();
                }

                OpConst16 => {
                    let ix = self.read_u16(offset + 1);
                    writeln!(
                        out,
                        "2 bytes: idx = {}, value = {:?}",
                        ix,
                        self.consts.get(ix as usize)
                    )
                    .unwrap();

                    iter.next();
                    iter.next();
                }

                OpNegate | OpAdd | OpSub | OpMul | OpDiv | OpReturn => {
                    writeln!(out, "{:?}", code).unwrap()
                }
            }
        }

        out.flush().unwrap();
    }
}
