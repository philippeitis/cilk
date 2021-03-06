use crate::ir::{basic_block::*, function::*, module::*, opcode::*, types::*};

pub struct IRLivenessAnalyzer<'a> {
    module: &'a Module,
}

impl<'a> IRLivenessAnalyzer<'a> {
    pub fn new(module: &'a Module) -> Self {
        Self { module }
    }

    pub fn analyze(&mut self) {
        for (_, f) in &self.module.functions {
            self.set_def(f);
            self.visit(&f);
        }
    }

    pub fn set_def(&mut self, f: &Function) {
        for (_, bb) in &f.basic_blocks.arena {
            let def = &mut bb.liveness.borrow_mut().def;

            for inst_val in &*bb.iseq.borrow() {
                let inst_id = inst_val.get_inst_id().unwrap();
                let inst = &f.inst_table[inst_id];

                if inst.opcode == Opcode::Call && f.get_return_type() != Type::Void {
                    def.insert(inst_id);
                    continue;
                }

                if inst.opcode.returns_value() {
                    def.insert(inst_id);
                }
            }
        }
    }

    pub fn visit(&mut self, f: &Function) {
        for (bb_id, bb) in &f.basic_blocks.arena {
            for inst_val in &*bb.iseq.borrow() {
                let inst = &f.inst_table[inst_val.get_inst_id().unwrap()];
                self.visit_operands(f, bb_id, &inst.operands, inst.opcode == Opcode::Phi);
            }
        }
    }

    pub fn visit_operands(
        &mut self,
        f: &Function,
        cur_bb: BasicBlockId,
        operands: &[Operand],
        is_phi: bool,
    ) {
        for i in 0..operands.len() {
            match &operands[i] {
                Operand::BasicBlock(_)
                | Operand::Type(_)
                | Operand::ICmpKind(_)
                | Operand::FCmpKind(_) => {}
                Operand::Value(v) if is_phi => some_then!(
                    id,
                    v.get_inst_id(),
                    self.propagate(f, cur_bb, id, Some(*operands[i + 1].as_basic_block()))
                ),
                Operand::Value(v) => {
                    some_then!(id, v.get_inst_id(), self.propagate(f, cur_bb, id, None));
                }
            }
        }
    }

    pub fn propagate(
        &mut self,
        f: &Function,
        bb_id: BasicBlockId,
        inst_id: InstructionId,
        phi_incoming: Option<BasicBlockId>,
    ) {
        let bb = &f.basic_blocks.arena[bb_id];

        {
            let mut bb_liveness = bb.liveness.borrow_mut();

            if bb_liveness.def.contains(&inst_id) {
                return;
            }

            if !bb_liveness.live_in.insert(inst_id) {
                // live_in already had the value inst_id
                return;
            }
        }

        if let Some(phi_incoming) = phi_incoming {
            self.propagate_if_necessary(f, phi_incoming, inst_id);
        } else {
            for pred_id in &bb.pred {
                self.propagate_if_necessary(f, *pred_id, inst_id);
            }
        }
    }

    fn propagate_if_necessary(
        &mut self,
        f: &Function,
        pred_id: BasicBlockId,
        inst_id: InstructionId,
    ) {
        let pred = &f.basic_blocks.arena[pred_id];
        if pred.liveness.borrow_mut().live_out.insert(inst_id) {
            // live_out didn't have the value inst_id
            self.propagate(f, pred_id, inst_id, None);
        }
    }
}
