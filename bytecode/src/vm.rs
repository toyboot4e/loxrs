use ::std::ops;

use ::anyhow::*;

use crate::chunk::*;

#[derive(Debug)]
pub enum VmError {
    CompileError,
    RuntimeError,
}

/// Loxrs virtual machine
pub struct Vm {
    chunk: ChunkData,
    ix: usize,
    stack: Vec<Value>,
}

impl Vm {
    pub fn new() -> Self {
        Self {
            chunk: ChunkData::new(),
            ix: 0,
            stack: Vec::with_capacity(256),
        }
    }

    pub fn clear_stack(&mut self) {
        self.stack.clear();
    }

    pub fn chunk_mut(&mut self) -> &mut ChunkData {
        &mut self.chunk
    }

    pub fn ix(&self) -> usize {
        self.ix
    }

    pub fn stack(&mut self) -> &Vec<Value> {
        &self.stack
    }
}

/// Run
impl Vm {
    pub fn run(&mut self) -> Result<()> {
        let chunk_len = self.chunk.bytes().len();
        while self.ix < chunk_len {
            // consume the next instruction
            let byte = self.chunk.read_u8(self.ix);
            self.ix += 1;

            {
                // TODO: optional trace print
                // self.trace_print(code.tag());
            }

            let code: OpCode = unsafe { std::mem::transmute(byte) };
            use OpCode::*;
            match code {
                OpReturn => {
                    // FIXME: the return value has to be poped by the caller
                    return Ok(());
                }

                OpConst8 => {
                    let ix = self.chunk.read_u8(self.ix);
                    self.ix += 1;
                    let value = *self
                        .chunk
                        .consts()
                        .get(ix as usize)
                        .ok_or(anyhow!("missing index after OpConst8"))?;
                    self.stack.push(value);
                    // println!("{}, {} => {:?}", "byte1", ix, value);
                }

                OpConst16 => {
                    let ix = self.chunk.read_u16(self.ix);
                    self.ix += 2;
                    let ip = self.ix;
                    let value = *self
                        .chunk
                        .consts()
                        .get(ix as usize)
                        .ok_or_else(|| anyhow!("missing index after OpConst16 at {}", ip))?;
                    self.stack.push(value);
                    // println!("{}, {} => {:?}", "byte2", ix, value);
                }

                OpNegate => {
                    let v = -self.stack.pop().expect("error when interpretting OnNegate");
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

    /// Tests `-((64.0 - 32.0) / 16.0)` results in `2.0`
    #[test]
    fn vm_binary_oper() {
        println!("=== vm_binary_oper()  ===");
        let mut vm = Vm::new();
        {
            let chunk = vm.chunk_mut();

            // use 2^x considering the accuracy of floating values
            chunk.push_const(64.0);
            chunk.push_const(32.0);
            chunk.push_const(16.0);

            chunk.push_ix_u8(0); // 64.0
            chunk.push_ix_u8(1); // 32.0
            chunk.push_code(OpSub); // -

            chunk.push_ix_u16(2); // 16.0
            chunk.push_code(OpDiv); // /

            chunk.push_code(OpNegate); // -

            chunk.push_code(OpReturn);
        }

        match vm.run() {
            Err(why) => panic!("{:?}", why),
            Ok(()) => assert_eq!(Some(&-2.0), vm.stack().last()),
        }
    }
}
