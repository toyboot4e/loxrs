use ::std::fmt;

/// Operation code for the bytecode interpreter
pub enum OpCode {
    Return,
}

impl fmt::Display for OpCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                OpCode::Return => "return",
            }
        )
    }
}

/// Chunk of instructions / `Vec` of `OpCode`.
///
/// It's different from the original implementation in the book:
/// * Capacity being initialized with zero and becomes one after pushing a first element.
/// * No automatic shrinking
pub type ChunkData = Vec<OpCode>;

pub trait DebugPrint {
    fn debug_print(&self, name: &str);
}

impl DebugPrint for ChunkData {
    fn debug_print(&self, title: &str) {
        println!("== {} ==", title);
        for code in self.iter() {
            println!("{}", code);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // cargo test -- --nocapture
    #[test]
    fn test_chunk_debug_print() {
        let mut chunk = ChunkData::new();
        chunk.push(OpCode::Return);
        chunk.push(OpCode::Return);
        chunk.debug_print("tested chunk");
    }
}
