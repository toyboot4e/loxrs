use std::io::prelude::*;

#[derive(Debug, Clone, Copy)]
pub enum OpCode {
    OpReturn,

    /// Followed by a byte index
    OpConst8,
    /// Followed by a two byte index
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

pub type Value = f64;

/// Chunk of instructions (`OpCode`s)
pub struct ChunkData {
    /// Read as `OpCode` or index
    bytes: Vec<u8>,
    consts: Vec<Value>,
    // tracks: Vec<ChunkTrackItem>,
}

pub type ChunkCodeIndex = usize;
pub type ChunkConstIndex = usize;

pub struct ChunkTrackItem {
    line: u32,
}

impl ChunkData {
    pub fn new() -> Self {
        Self {
            bytes: Vec::new(),
            consts: Vec::new(),
            // tracks: Vec::new(),
        }
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn consts(&mut self) -> &Vec<Value> {
        &self.consts
    }

    pub fn store_const(&mut self, value: Value) {
        self.consts.push(value)
    }
}

/// Read
impl ChunkData {
    pub fn read_u8(&self, offset: ChunkCodeIndex) -> u8 {
        self.bytes[offset]
    }

    pub fn read_u16(&self, offset: ChunkCodeIndex) -> u16 {
        ((self.bytes[offset] as u16) << 8) | (self.bytes[offset + 1] as u16)
    }
}

/// Write
impl ChunkData {
    #[inline(always)]
    pub fn push_code(&mut self, tag: OpCode) {
        self.bytes.push(tag as u8);
    }

    #[inline(always)]
    pub fn push_idx_u8(&mut self, x: u8) {
        self.bytes.push(OpCode::OpConst8 as u8);
        self.bytes.push(x);
    }

    #[inline(always)]
    pub fn push_idx_u16(&mut self, x: u16) {
        self.bytes.push(OpCode::OpConst16 as u8);
        self.bytes.push(x as u8);
        self.bytes.push((x >> 8) as u8);
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
            let code: OpCode = unsafe { std::mem::transmute(byte) };
            match code {
                OpConst8 => {
                    let idx = self.read_u8(offset + 1);
                    writeln!(
                        out,
                        "1 byte: idx =  {}, value = {:?}",
                        idx,
                        self.consts.get(idx as ChunkConstIndex)
                    )
                    .unwrap();
                    iter.next();
                }

                OpConst16 => {
                    let idx = self.read_u16(offset + 1);
                    writeln!(
                        out,
                        "2 bytes: idx = {}, value = {:?}",
                        idx,
                        self.consts.get(idx as ChunkConstIndex)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;

    #[test]
    fn test_memory_sizes() {
        assert_eq!(1, mem::size_of::<OpCode>());
    }

    /// Not automatic test; check it with your eye
    #[test]
    fn debug_print_chunk_data() {
        use OpCode::*;

        let mut chunk = ChunkData::new();
        chunk.push_code(OpReturn);
        chunk.push_code(OpReturn);
        chunk.push_idx_u8(42u8);
        chunk.push_idx_u16(600u16);
        chunk.consts.push(4124.45);
        chunk.push_idx_u8(0);

        chunk.debug_print("test chunk");
    }
}
