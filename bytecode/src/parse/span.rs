use std::fmt;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub struct BytePos(pub usize);

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct ByteSpan {
    pub lo: BytePos,
    pub hi: BytePos,
}

impl Default for ByteSpan {
    fn default() -> Self {
        Self {
            lo: BytePos(0),
            hi: BytePos(0),
        }
    }
}

impl fmt::Display for ByteSpan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.lo.0, self.hi.0)?;
        Ok(())
    }
}

impl From<[usize; 2]> for ByteSpan {
    fn from(x: [usize; 2]) -> Self {
        Self::new(BytePos(x[0]), BytePos(x[1]))
    }
}

impl ByteSpan {
    pub fn new(lo: BytePos, hi: BytePos) -> Self {
        Self { lo, hi }
    }

    pub fn len(&self) -> usize {
        self.hi.0 - self.lo.0
    }
}

/// Human friendly source position representation
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct SrcPos {
    /// One-based line number
    ln: usize,
    /// One-based column number
    col: usize,
}

impl SrcPos {
    pub fn initial() -> Self {
        Self::new(1, 1)
    }

    pub fn new(ln: usize, col: usize) -> Self {
        Self { ln, col }
    }

    pub fn ln(&self) -> usize {
        self.ln
    }

    pub fn col(&self) -> usize {
        self.col
    }
}

impl SrcPos {
    pub fn inc_ln(&mut self) {
        self.ln += 1;
        self.col = 1;
    }

    pub fn inc_col(&mut self) {
        self.col += 1;
    }
}

impl fmt::Display for SrcPos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(ln:{} col{})", self.ln, self.col)
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct SrcSpan {
    pub lo: SrcPos,
    pub hi: SrcPos,
}
