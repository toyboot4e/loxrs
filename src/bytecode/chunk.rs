use ::std::io::prelude::*;

// TODO: can I remove `unsafe` around union?

/// Operation code for the bytecode interpreter
///
/// Refer to the [reference](https://doc.rust-lang.org/reference/items/unions.html) about unions in
/// Rust
// #[derive(Debug, Clone)]
#[repr(C)]
pub union OpCode {
    tag: OpCodeTag,
    const_idx: u8,
}

impl OpCode {
    pub unsafe fn tag(&self) -> OpCodeTag {
        self.tag
    }

    // TODO: make it safe
    pub unsafe fn clone(&self) -> Self {
        std::mem::transmute::<u8, OpCode>(self.const_idx.clone())
    }
}

// TODO: is it byte length?
#[derive(Debug, Clone, Copy)]
pub enum OpCodeTag {
    OpReturn,
    /// Followed by an index
    OpConstant1Byte,
    /// Followed by an index
    OpConstant2Byte,
    OpNegate,
    OPAdd,
    OpSub,
    OpMul,
    OpDiv,
}

impl OpCodeTag {
    fn as_byte(&self) {
        print!("{:b}", unsafe {
            std::mem::transmute::<OpCodeTag, u8>(self.clone())
        });
    }
}

pub trait Chunk {
    fn code(&mut self) -> &mut Vec<OpCode>;

    fn push_tag(&mut self, tag: OpCodeTag) {
        self.code().push(OpCode { tag: tag });
    }

    fn push_idx_u8(&mut self, x: u8) {
        self.code().push(OpCode {
            tag: OpCodeTag::OpConstant1Byte,
        });
        self.code().push(OpCode { const_idx: x });
    }

    fn push_idx_u16(&mut self, x: u16) {
        self.code().push(OpCode {
            tag: OpCodeTag::OpConstant2Byte,
        });
        self.code().push(OpCode {
            const_idx: (x >> 8) as u8,
        });
        self.code().push(OpCode { const_idx: x as u8 });
    }
}

pub type Value = f64;

/// Chunk of instructions / `Vec` of `OpCode`.
///
/// It's different from the original implementation in the book:
///
/// * FIXME: Capacity being initialized with zero and becomes one after pushing a first element.
/// * No automatic shrinking
pub struct ChunkData {
    code: Vec<OpCode>,
    consts: Vec<Value>,
    // tracks: Vec<ChunkTrackItem>,
}
pub type ChunkCodeIndex = usize;

impl ChunkData {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            consts: Vec::new(),
            // tracks: Vec::new(),
        }
    }

    pub fn consts(&mut self) -> &mut Vec<Value> {
        &mut self.consts
    }

    pub fn push_const(&mut self, value: Value) {
        self.consts.push(value)
    }
}

impl Chunk for ChunkData {
    fn code(&mut self) -> &mut Vec<OpCode> {
        &mut self.code
    }
}

pub struct ChunkTrackItem {
    line: u32,
}

// *************************
// ***** debug & tests *****
// *************************

/// Extends `ChunkData` i.e. `Vec<OpCode>`
pub trait DebugPrintUnsafe {
    unsafe fn debug_print(&self, title: &str);
    // internal utilities
    unsafe fn read_u8(&self, offset: ChunkCodeIndex) -> u8;
    unsafe fn read_u16(&self, offset: ChunkCodeIndex) -> u16;
}

impl DebugPrintUnsafe for ChunkData {
    /// Disassembles `ChunkData`
    unsafe fn debug_print(&self, title: &str) {
        let out = std::io::stdout();
        let out = &mut out.lock();

        writeln!(out, "== {} ==", title).unwrap();

        // TODO: consider using StdoutLock
        let mut iter = self.code.iter().enumerate();
        while let Some((offset, code)) = iter.next() {
            match code.tag {
                OpCodeTag::OpConstant1Byte => {
                    let idx = self.read_u8(offset + 1);
                    writeln!(
                        out,
                        "1 byte: idx =  {}, value = {:?}",
                        idx,
                        self.consts.get(idx as usize)
                    )
                    .unwrap();
                    iter.next();
                }
                OpCodeTag::OpConstant2Byte => {
                    let idx = self.read_u16(offset + 1);
                    writeln!(
                        out,
                        "2 bytes: idx = {}, value = {:?}",
                        idx,
                        self.consts.get(idx as usize)
                    )
                    .unwrap();
                    iter.next();
                    iter.next();
                }
                _ => writeln!(out, "{:?}", code.tag).unwrap(),
            }
        }

        out.flush().unwrap();
    }

    unsafe fn read_u8(&self, offset: ChunkCodeIndex) -> u8 {
        self.code[offset].const_idx
    }

    unsafe fn read_u16(&self, offset: ChunkCodeIndex) -> u16 {
        ((self.code[offset].const_idx as u16) << 8) | self.code[offset + 1].const_idx as u16
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::std::mem;

    #[test]
    fn test_memory_sizes() {
        // each `OpCode` must have size of 1 byte
        assert_eq!(1, mem::size_of::<OpCode>());
        assert_eq!(1, mem::size_of::<OpCodeTag>());
    }

    // cargo test -- --no-capture
    #[test]
    fn debug_print_chunk_data() {
        use OpCodeTag::*;
        let mut chunk = ChunkData::new();
        chunk.push_tag(OpReturn);
        chunk.push_tag(OpReturn);
        chunk.push_idx_u8(42u8);
        chunk.push_idx_u16(600u16);
        chunk.consts.push(4124.45);
        chunk.push_idx_u8(0);
        unsafe {
            chunk.debug_print("tested chunk");
        }
    }
}
