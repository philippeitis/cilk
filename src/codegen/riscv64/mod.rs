pub mod asm;
pub mod dag;
pub mod exec;
pub mod frame_object;
pub mod machine;

use crate::{
    codegen::common::{
        dag::{combine, convert},
        machine::{branch_folding, module::MachineModule, phi_elimination},
    },
    ir,
    ir::module::Module,
    ir::types::*,
    traits::pass::ModulePassManager,
};

impl TypeSize for Type {
    fn size_in_byte(&self, tys: &Types) -> usize {
        match self {
            Type::Int1 => 1,
            Type::Int8 => 1,
            Type::Int32 => 4,
            Type::Int64 => 8,
            Type::F64 => 8,
            Type::Array(id) => tys.base.borrow().non_primitive_types[*id]
                .as_array()
                .size_in_byte(tys),
            Type::Struct(id) => tys.base.borrow().non_primitive_types[*id]
                .as_struct()
                .size_in_byte(tys),
            Type::Pointer(_) => 8,
            Type::Function(_) => unimplemented!(),
            Type::Void => 0,
        }
    }

    fn size_in_bits(&self, tys: &Types) -> usize {
        self.size_in_byte(tys) * 8
    }

    fn align_in_byte(&self, tys: &Types) -> usize {
        match self {
            Type::Int1 => 1,
            Type::Int8 => 1,
            Type::Int32 => 4,
            Type::Int64 => 8,
            Type::F64 => 8,
            Type::Array(id) => tys.base.borrow().non_primitive_types[*id]
                .as_array()
                .align_in_byte(tys),
            Type::Struct(id) => tys.base.borrow().non_primitive_types[*id]
                .as_struct()
                .align_in_byte(tys),
            Type::Pointer(_) => 8,
            Type::Function(_) => unimplemented!(),
            Type::Void => 0,
        }
    }
}

const MAX_ALIGN: usize = 16;

impl TypeSize for ArrayType {
    fn size_in_byte(&self, tys: &Types) -> usize {
        self.elem_ty.size_in_byte(tys) * self.len
    }

    fn size_in_bits(&self, tys: &Types) -> usize {
        self.size_in_byte(tys) * 8
    }

    fn align_in_byte(&self, tys: &Types) -> usize {
        let size = self.size_in_byte(tys);
        let align = self.elem_ty.align_in_byte(tys);
        if size > MAX_ALIGN {
            MAX_ALIGN
        } else {
            align
        }
    }
}

impl TypeSize for StructType {
    fn size_in_byte(&self, _tys: &Types) -> usize {
        self.size()
    }

    fn size_in_bits(&self, tys: &Types) -> usize {
        self.size_in_byte(tys) * 8
    }

    fn align_in_byte(&self, _tys: &Types) -> usize {
        self.align()
    }
}

pub fn standard_conversion_into_machine_module(module: &mut Module) -> MachineModule {
    ir::branch_folding::BranchFolding::new().run_on_module(module);
    ir::merge_ret::MergeReturns::new().run_on_module(module);

    let mut dag_module = convert::ConvertToDAGModule::new(module).run();

    let mut pass_mgr = ModulePassManager::new();
    pass_mgr.add_pass(combine::Combine::new());
    pass_mgr.add_pass(dag::legalize::Legalize::new());
    pass_mgr.add_pass(dag::isel::MISelector::new());
    pass_mgr.run_on_module(&mut dag_module);
    // println!("{:?}", dag_module);

    let mut machine_module = dag::mc_convert::convert_module(dag_module);

    let mut pass_mgr = ModulePassManager::new();
    pass_mgr.add_pass(phi_elimination::PhiElimination::new());
    // pass_mgr.add_pass(machine::two_addr::TwoAddressConverter::new());
    pass_mgr.add_pass(machine::regalloc::RegisterAllocator::new());
    pass_mgr.add_pass(branch_folding::BranchFolding::new());
    pass_mgr.add_pass(machine::validate_frame_index::ValidateFrameIndex::new());
    pass_mgr.add_pass(machine::pro_epi_inserter::PrologueEpilogueInserter::new());
    pass_mgr.add_pass(machine::replace_copy::ReplaceCopyWithProperMInst::new());
    // pass_mgr.add_pass(machine::replace_data::ReplaceConstFPWithMemoryRef::new());
    pass_mgr.run_on_module(&mut machine_module);

    machine_module
}
