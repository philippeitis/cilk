use super::{function::DAGFunction, module::DAGModule, node::*};
use crate::ir::types::*;
use crate::util::allocator::*;
use rustc_hash::FxHashMap;

pub struct MISelector {
    selected: FxHashMap<Raw<DAGNode>, Raw<DAGNode>>,
}

impl MISelector {
    pub fn new() -> Self {
        Self {
            selected: FxHashMap::default(),
        }
    }

    pub fn run_on_module(&mut self, module: &mut DAGModule) {
        for (_, func) in &mut module.functions {
            self.run_on_function(func)
        }
    }

    fn run_on_function(&mut self, func: &mut DAGFunction) {
        for bb_id in &func.dag_basic_blocks {
            let bb = &func.dag_basic_block_arena[*bb_id];
            self.run_on_node(&mut func.dag_heap, bb.entry.unwrap());
        }
    }

    fn run_on_node(
        &mut self,
        heap: &mut RawAllocator<DAGNode>,
        mut node: Raw<DAGNode>,
    ) -> Raw<DAGNode> {
        macro_rules! is_const {
            ($node:expr) => {
                $node.is_constant()
            };
            ($node:expr, $ty:expr) => {
                is_const!($node) && $node.ty == $ty
            };
        }
        macro_rules! is_fi {
            ($node:expr) => {
                $node.is_frame_index()
            };
            ($node:expr, $ty:expr) => {
                is_fi!($node) && $node.ty == $ty
            };
        }
        macro_rules! is_maybe_reg {
            ($node:expr) => {
                $node.is_maybe_register()
            };
            ($node:expr, $ty:expr) => {
                is_maybe_reg!($node) && $node.ty == $ty
            };
        }

        if !node.is_operation() {
            return node;
        }

        if let Some(node) = self.selected.get(&node) {
            return *node;
        }

        // TODO: following code will be auto-generated by macro
        let mut selected = match node.kind {
            NodeKind::IR(IRNodeKind::Add) => {
                let kind = if is_maybe_reg!(node.operand[0], Type::Int32) {
                    if is_maybe_reg!(node.operand[1], Type::Int32) {
                        // (Add $a:GR32 $b:GR32) => (ADDrr32 $a $b)
                        NodeKind::MI(MINodeKind::ADDrr32)
                    } else if is_const!(node.operand[1], Type::Int32) {
                        // (Add $a:GR32 $b:const.i32) => (ADDri32 $a $b)
                        NodeKind::MI(MINodeKind::ADDri32)
                    } else {
                        NodeKind::IR(IRNodeKind::Add)
                    }
                } else if is_maybe_reg!(node.operand[0])
                    && matches!(node.operand[0].ty,Type::Int64|Type::Pointer(_))
                {
                    if is_const!(node.operand[1], Type::Int32) {
                        NodeKind::MI(MINodeKind::ADDr64i32)
                    } else {
                        NodeKind::IR(IRNodeKind::Add)
                    }
                } else {
                    NodeKind::IR(IRNodeKind::Add)
                };

                let op0 = self.run_on_node(heap, node.operand[0]);
                let op1 = self.run_on_node(heap, node.operand[1]);
                heap.alloc(DAGNode::new(kind, vec![op0, op1], node.ty.clone()))
            }
            NodeKind::IR(IRNodeKind::Sub) => {
                let kind = if is_maybe_reg!(node.operand[0], Type::Int32) {
                    if is_maybe_reg!(node.operand[1], Type::Int32) {
                        // (Sub $a:GR32 $b:GR32) => (SUBrr32 $a $b)
                        NodeKind::MI(MINodeKind::SUBrr32)
                    } else if is_const!(node.operand[1], Type::Int32) {
                        // (Sub $a:GR32 $b:const.i32) => (SUBri32 $a $b)
                        NodeKind::MI(MINodeKind::SUBri32)
                    } else {
                        NodeKind::IR(IRNodeKind::Sub)
                    }
                } else {
                    NodeKind::IR(IRNodeKind::Sub)
                };

                let op0 = self.run_on_node(heap, node.operand[0]);
                let op1 = self.run_on_node(heap, node.operand[1]);
                heap.alloc(DAGNode::new(kind, vec![op0, op1], node.ty.clone()))
            }
            NodeKind::IR(IRNodeKind::Mul) => {
                let kind = if is_maybe_reg!(node.operand[0], Type::Int32) {
                    if is_maybe_reg!(node.operand[1], Type::Int32) {
                        // (Mul $a:GR32 $b:GR32) => (IMULrr32 $a $b)
                        NodeKind::MI(MINodeKind::IMULrr32)
                    } else if is_const!(node.operand[1], Type::Int32) {
                        // (Mul $a:GR32 $b:const.i32) => (IMULrri32 $a $b)
                        NodeKind::MI(MINodeKind::IMULrri32)
                    } else {
                        NodeKind::IR(IRNodeKind::Mul)
                    }
                } else if is_maybe_reg!(node.operand[0], Type::Int64) {
                    if is_const!(node.operand[1], Type::Int32) {
                        // (Mul $a:GR64 $b:const.i32) => (IMULrr64i32 $a $b)
                        NodeKind::MI(MINodeKind::IMULrr64i32)
                    } else {
                        NodeKind::IR(IRNodeKind::Mul)
                    }
                } else {
                    NodeKind::IR(IRNodeKind::Mul)
                };

                let op0 = self.run_on_node(heap, node.operand[0]);
                let op1 = self.run_on_node(heap, node.operand[1]);
                heap.alloc(DAGNode::new(kind, vec![op0, op1], node.ty.clone()))
            }
            NodeKind::IR(IRNodeKind::Load) => {
                let kind = if node.operand[0].is_frame_index() {
                    match node.operand[0].ty {
                        Type::Int32 => NodeKind::MI(MINodeKind::MOVrm32),
                        Type::Pointer(_) | Type::Int64 => NodeKind::MI(MINodeKind::MOVrm64),
                        _ => unimplemented!(),
                    }
                } else if node.operand[0].is_maybe_register()
                    && matches!(node.operand[0].ty, Type::Pointer(_))
                {
                    NodeKind::MI(MINodeKind::MOVrp32)
                } else {
                    unimplemented!()
                };
                let op0 = self.run_on_node(heap, node.operand[0]);
                heap.alloc(DAGNode::new(kind, vec![op0], node.ty.clone()))
            }
            NodeKind::IR(IRNodeKind::Store) => {
                let kind = if is_fi!(node.operand[0], Type::Int32)
                    && is_maybe_reg!(node.operand[1], Type::Int32)
                {
                    NodeKind::MI(MINodeKind::MOVmr32)
                } else if is_fi!(node.operand[0], Type::Int32)
                    && is_const!(node.operand[1], Type::Int32)
                {
                    NodeKind::MI(MINodeKind::MOVmi32)
                } else if node.operand[0].is_maybe_register()
                    && matches!(node.operand[0].ty, Type::Pointer(_))
                    && is_const!(node.operand[1], Type::Int32)
                {
                    NodeKind::MI(MINodeKind::MOVpi32)
                } else if node.operand[0].is_maybe_register()
                    && matches!(node.operand[0].ty, Type::Pointer(_))
                    && is_maybe_reg!(node.operand[1], Type::Int32)
                {
                    NodeKind::MI(MINodeKind::MOVpr32)
                } else {
                    unimplemented!()
                };
                let op0 = self.run_on_node(heap, node.operand[0]);
                let op1 = self.run_on_node(heap, node.operand[1]);
                heap.alloc(DAGNode::new(kind, vec![op0, op1], node.ty.clone()))
            }
            _ => {
                node.operand = node
                    .operand
                    .iter()
                    .map(|op| self.run_on_node(heap, *op))
                    .collect();
                node
            }
        };

        self.selected.insert(node, selected);

        if let Some(next) = node.next {
            selected.next = Some(self.run_on_node(heap, next));
        }

        selected
    }
}
