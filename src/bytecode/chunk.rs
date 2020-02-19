use ::std::fmt;
use ::std::io::prelude::*;

// TODO: test length (memory layout)

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

// TODO: is it byte length?
#[derive(Debug, Clone, Copy)]
pub enum OpCodeTag {
    OpReturn = 0,
    /// Followed by constant index
    OpConstant1Byte = 1,
    OpConstant2Byte = 2,
}

impl fmt::Display for OpCodeTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use OpCodeTag::*;
        write!(
            f,
            "{}",
            match self {
                OpReturn => "OP_return",
                OpConstant1Byte => "OP_constant_1byte",
                OpConstant2Byte => "OP_constant_2byte",
            }
        )
    }
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

    fn push_u8(&mut self, x: u8) {
        self.code().push(OpCode {
            tag: OpCodeTag::OpConstant1Byte,
        });
        self.code().push(OpCode { const_idx: x });
    }

    fn push_u16(&mut self, x: u16) {
        self.code().push(OpCode {
            tag: OpCodeTag::OpConstant2Byte,
        });
        self.code().push(OpCode {
            const_idx: (x >> 8) as u8,
        });
        self.code().push(OpCode { const_idx: x as u8 });
    }
}

/// Chunk of instructions / `Vec` of `OpCode`.
///
/// It's different from the original implementation in the book:
///
/// * FIXME: Capacity being initialized with zero and becomes one after pushing a first element.
/// * No automatic shrinking
pub struct ChunkData {
    code: Vec<OpCode>,
    // tracks: Vec<ChunkTrackItem>,
}
pub type CodeIndex = usize;

impl ChunkData {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            // tracks: Vec::new(),
        }
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
    unsafe fn read_u8(&self, offset: CodeIndex) -> u8;
    unsafe fn read_u16(&self, offset: CodeIndex) -> u16;
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
                    writeln!(out, "1 byte: {}", self.read_u8(offset + 1)).unwrap();
                    iter.next();
                }
                OpCodeTag::OpConstant2Byte => {
                    writeln!(out, "2 bytes: {}", self.read_u16(offset + 1)).unwrap();
                    iter.next();
                    iter.next();
                }
                _ => writeln!(out, "{}", code.tag).unwrap(),
            }
        }

        out.flush().unwrap();
    }

    unsafe fn read_u8(&self, offset: CodeIndex) -> u8 {
        self.code[offset].const_idx
    }

    unsafe fn read_u16(&self, offset: CodeIndex) -> u16 {
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
        chunk.push_u8(42u8);
        chunk.push_u16(600u16);
        unsafe {
            chunk.debug_print("tested chunk");
        }
    }
}
