pub use crate::codegen::common::dag::node::*;

#[derive(Debug, Clone, PartialEq)]
pub enum MemNodeKind {
    // RegImm,
    RegFi,
    Reg,
    // FiReg,
    // ImmReg,
    Address,
}
