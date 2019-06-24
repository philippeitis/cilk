use super::{function::*, value::*};
use id_arena::*;

pub type BasicBlockId = Id<BasicBlock>;

#[derive(Clone, Debug)]
pub struct BasicBlock {
    pub iseq: Vec<Value>,
}

impl BasicBlock {
    pub fn new() -> Self {
        Self { iseq: vec![] }
    }

    pub fn to_string(&self, f: &Function) -> String {
        self.iseq.iter().fold("".to_string(), |s, instr| {
            format!("{}{}\n", s, instr.to_string(f, true))
        })
    }
}
