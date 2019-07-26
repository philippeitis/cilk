use super::{basic_block::*, function::*, instr::*, module::*};
use crate::ir::types::*;
// use super::{convert::*, node::*};
// use id_arena::*;

pub struct LivenessAnalysis<'a> {
    pub module: &'a MachineModule, // TODO: Will be used to get type
}

impl<'a> LivenessAnalysis<'a> {
    pub fn new(module: &'a MachineModule) -> Self {
        Self { module }
    }

    pub fn analyze_module(&mut self) {
        for (_, func) in &self.module.functions {
            self.analyze_function(func);
        }
    }

    pub fn analyze_function(&mut self, cur_func: &MachineFunction) {
        self.number_vreg(cur_func);
        self.set_def(cur_func);
        self.visit(cur_func);
    }

    fn number_vreg(&mut self, cur_func: &MachineFunction) {
        let mut vreg = 1;
        for (_, bb) in &cur_func.basic_blocks {
            for instr_id in &bb.iseq {
                cur_func.instr_arena[*instr_id].set_vreg(vreg);
                vreg += 1;
            }
        }
    }

    fn set_def(&mut self, cur_func: &MachineFunction) {
        for (_, bb) in &cur_func.basic_blocks {
            for instr_id in &bb.iseq {
                self.set_def_instr(cur_func, bb, *instr_id);
            }
        }
    }

    fn set_def_instr(
        &mut self,
        cur_func: &MachineFunction,
        bb: &MachineBasicBlock,
        instr_id: MachineInstrId,
    ) {
        let instr = &cur_func.instr_arena[instr_id];

        if let MachineOpcode::Add
        | MachineOpcode::Sub
        | MachineOpcode::Seteq
        | MachineOpcode::Setle
        | MachineOpcode::Load = instr.opcode
        {
            bb.liveness.borrow_mut().def.insert(instr_id);
        }

        if instr.opcode == MachineOpcode::Call && instr.ty.as_ref().unwrap() != &Type::Void {
            bb.liveness.borrow_mut().def.insert(instr_id);
        }
    }

    fn visit(&mut self, cur_func: &MachineFunction) {
        for (bb_id, bb) in &cur_func.basic_blocks {
            for instr_id in &bb.iseq {
                self.visit_instr(cur_func, bb_id, *instr_id);
            }
        }
    }

    fn visit_instr(
        &mut self,
        cur_func: &MachineFunction,
        bb: MachineBasicBlockId,
        instr_id: MachineInstrId,
    ) {
        let instr = &cur_func.instr_arena[instr_id];
        for operand in &instr.oprand {
            match_then!(
                MachineOprand::Instr(id),
                operand,
                self.propagate(cur_func, bb, *id)
            );
        }
    }

    fn propagate(
        &self,
        cur_func: &MachineFunction,
        bb: MachineBasicBlockId,
        instr_id: MachineInstrId,
    ) {
        let bb = &cur_func.basic_blocks[bb];

        {
            let mut bb_liveness = bb.liveness.borrow_mut();

            if bb_liveness.def.contains(&instr_id) {
                return;
            }

            if !bb_liveness.live_in.insert(instr_id) {
                // live_in already had the value instr_id
                return;
            }
        }

        for pred_id in &bb.pred {
            let pred = &cur_func.basic_blocks[*pred_id];
            if pred.liveness.borrow_mut().live_out.insert(instr_id) {
                // live_out didn't have the value instr_id
                self.propagate(cur_func, *pred_id, instr_id);
            }
        }
    }
}
