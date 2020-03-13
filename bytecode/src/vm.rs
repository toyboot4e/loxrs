use crate::chunk::*;
use ::std::ops;

// TODO: maybe handle runtime errors in bytecode VM wihtout `expect`ing

pub type Result = ::std::result::Result<(), VmInterpretError>;

#[derive(Debug)]
pub enum VmInterpretError {
    CompileError,
    RuntimeError,
}

pub struct Vm {
    chunk: ChunkData,
    ip: ChunkCodeIndex,
    stack: Vec<Value>,
}

// stack operations
impl Vm {
    pub fn new() -> Self {
        Self {
            chunk: ChunkData::new(),
            ip: ChunkCodeIndex::default(),
            stack: Vec::with_capacity(256),
        }
    }

    pub fn chunk(&mut self) -> &mut ChunkData {
        &mut self.chunk
    }

    pub fn ip(&self) -> ChunkCodeIndex {
        self.ip
    }

    pub fn stack(&mut self) -> &mut Vec<Value> {
        &mut self.stack
    }

    pub fn print_stack(&self) {
        println!("VM stack: {:?};", &self.stack);
    }
}

// interpret
impl Vm {
    pub unsafe fn run(&mut self) -> Result {
        loop {
            let code = match self.chunk.code().get(self.ip) {
                Some(c) => c.clone(),
                None => break,
            };
            self.ip += 1;

            {
                // TODO: optional trace print
                self.trace_print(code.tag());
            }

            use OpCodeTag::*;
            match code.tag() {
                OpReturn => {
                    let x = self.stack.pop();
                    println!("return: {:?}", x);
                    return Ok(());
                }
                OpConst1 => {
                    let idx = self.chunk.read_u8(self.ip);
                    self.ip += 1;
                    let value = self
                        .chunk
                        .consts()
                        .get(idx as usize)
                        .expect("stack must always be resolved")
                        .clone();
                    self.stack.push(value);
                    // println!("{}, {} => {:?}", "byte1", idx, value);
                }
                OpConst2 => {
                    let idx = self.chunk.read_u16(self.ip);
                    self.ip += 2;
                    let value = self
                        .chunk
                        .consts()
                        .get(idx as usize)
                        .expect("stack must always be resolved")
                        .clone();
                    self.stack.push(value);
                    // println!("{}, {} => {:?}", "byte2", idx, value);
                }
                OpNegate => {
                    let v = -self.stack.pop().expect("interpret OnNegate").clone();
                    self.stack.push(v);
                }
                OPAdd => {
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
        let b = self.stack.pop().expect("binary_op_b");
        let a = self.stack.pop().expect("binary_op_a");
        self.stack.push(oper(a, b));
    }

    fn trace_print(&self, tag: OpCodeTag) {
        // TODO: align
        print!("trace: {:?}; ", tag);
        self.print_stack();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk::OpCodeTag::*;

    #[test]
    fn vm_binary_oper() {
        println!("=== vm_binary_oper()  ===");
        let mut vm = Vm::new();
        {
            // write chunks
            // -((32.2 - 14.2) / 9) i.e.
            // "32.2 14.2 - 9 / -" in Neverse Polish Notation
            let chunk = vm.chunk();

            chunk.store_const(32.2);
            chunk.store_const(14.2);
            chunk.store_const(9.0);

            chunk.push_idx_u8(0u8);
            chunk.push_idx_u8(1u8);
            chunk.push_tag(OpSub);

            chunk.push_idx_u16(2u16);
            chunk.push_tag(OpDiv);

            chunk.push_tag(OpNegate);
            chunk.push_tag(OpReturn);
        }
        unsafe {
            if let Err(why) = vm.run() {
                eprintln!("{:?}", why);
            }
        }
    }
}
