use crate::chunk::*;
use ::anyhow::{anyhow, Context, Error, Result};
use ::std::ops;

#[derive(Debug)]
pub enum VmError {
    CompileError,
    RuntimeError,
}

pub struct Vm {
    /// Chunk of bytecodes
    chunk: ChunkData,
    /// Instruction "pointer" (actual raw pointer is better for efficiency)
    ip: ChunkCodeIndex,
    /// Space for calculation
    stack: Vec<Value>,
}

impl Vm {
    pub fn new() -> Self {
        Self {
            chunk: ChunkData::new(),
            ip: ChunkCodeIndex::default(),
            stack: Vec::with_capacity(256),
        }
    }

    pub fn chunk_mut(&mut self) -> &mut ChunkData {
        &mut self.chunk
    }

    pub fn ip(&self) -> ChunkCodeIndex {
        self.ip
    }

    pub fn stack(&mut self) -> &Vec<Value> {
        &self.stack
    }
}

/// Print
impl Vm {
    pub fn print_stack(&self) {
        println!("VM stack: {:?};", &self.stack);
    }
}

/// Run
impl Vm {
    pub fn run(&mut self) -> Result<()> {
        let chunk_len = self.chunk.bytes().len();
        while self.ip < chunk_len {
            let byte = self.chunk.read_u8(self.ip);

            self.ip += 1;

            {
                // TODO: optional trace print
                // self.trace_print(code.tag());
            }

            let code: OpCode = unsafe { std::mem::transmute(byte) };
            use OpCode::*;
            match code {
                OpReturn => {
                    let x = self.stack.pop();
                    {
                        // TODO: optional trace print
                        println!("return: {:?}", x);
                    }
                    return Ok(());
                }

                OpConst8 => {
                    let idx = self.chunk.read_u8(self.ip);
                    self.ip += 1;
                    let value = *self
                        .chunk
                        .consts()
                        .get(idx as ChunkConstIndex)
                        .ok_or(anyhow!("missing index after OpConst8"))?;
                    self.stack.push(value);
                    // println!("{}, {} => {:?}", "byte1", idx, value);
                }

                OpConst16 => {
                    let idx = self.chunk.read_u16(self.ip);
                    self.ip += 2;
                    let value = *self
                        .chunk
                        .consts()
                        .get(idx as ChunkConstIndex)
                        .ok_or(anyhow!("missing index after OpConst16"))?;
                    self.stack.push(value);
                    // println!("{}, {} => {:?}", "byte2", idx, value);
                }

                OpNegate => {
                    let v = -self.stack.pop().expect("interpret OnNegate").clone();
                    self.stack.push(v);
                }
                OpAdd => {
                    self.binary_op(ops::Add::add);
                }
                OpSub => {
                    self.binary_op(ops::Sub::sub);
                }
                OpMul => {
                    self.binary_op(ops::Mul::mul);
                }
                OpDiv => {
                    self.binary_op(ops::Div::div);
                }
            }
        }

        Ok(())
    }

    #[inline]
    fn binary_op(&mut self, oper: impl Fn(Value, Value) -> Value) {
        let b = self.stack.pop().expect("binary_op b");
        let a = self.stack.pop().expect("binary_op a");
        self.stack.push(oper(a, b));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk::OpCode::*;

    /// Tests `-((32.2 - 14.2) / 9)` turns into 2
    ///
    /// In prefix notation, the expression is:
    /// 32.2 14.2 - 9 / -    = 2
    #[test]
    fn vm_binary_oper() {
        println!("=== vm_binary_oper()  ===");
        let mut vm = Vm::new();
        {
            let chunk = vm.chunk_mut();

            chunk.store_const(32.2);
            chunk.store_const(14.2);
            chunk.store_const(9.0);

            chunk.push_idx_u8(0u8);
            chunk.push_idx_u8(1u8);
            chunk.push_code(OpSub);

            chunk.push_idx_u16(2u16);
            chunk.push_code(OpDiv);

            chunk.push_code(OpNegate);

            chunk.push_code(OpReturn);
        }

        if let Err(why) = vm.run() {
            eprintln!("{:?}", why);
        } else {
            assert_eq!(Some(&2f64), vm.stack().last());
        }
    }
}
