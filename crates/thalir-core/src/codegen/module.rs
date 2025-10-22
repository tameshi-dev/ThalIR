use cranelift::prelude::EntityRef;
use cranelift_codegen::ir::Function;
use cranelift_codegen::isa;
use cranelift_codegen::settings;
use cranelift_codegen::Context;
use cranelift_frontend::Variable;
use cranelift_module::Module;
use cranelift_object::{ObjectBuilder, ObjectModule};
use std::collections::HashMap;

use crate::{
    codegen::context::CodegenContext,
    codegen::lowering::{lower_instruction, lower_terminator},
    contract::Contract,
    values::VarId,
    IrError, Result,
};

pub struct ModuleBuilder {
    module: ObjectModule,
}

impl ModuleBuilder {
    pub fn new() -> Result<Self> {
        let flags_builder = settings::builder();
        let isa_builder = isa::lookup_by_name("x86_64-unknown-unknown-elf")
            .map_err(|e| IrError::CraneliftError(format!("Failed to lookup ISA: {}", e)))?;

        let flags = settings::Flags::new(flags_builder);
        let isa = isa_builder
            .finish(flags)
            .map_err(|e| IrError::CraneliftError(format!("Failed to create ISA: {}", e)))?;

        let object_builder = ObjectBuilder::new(
            isa.clone(),
            "contract",
            cranelift_module::default_libcall_names(),
        )
        .unwrap();
        let module = ObjectModule::new(object_builder);

        Ok(Self { module })
    }

    pub fn compile_contract(mut self, contract: &Contract) -> Result<Vec<u8>> {
        let mut func_ids = HashMap::new();

        for (name, function) in &contract.functions {
            let mut sig = self.module.make_signature();
            for param in &function.signature.params {
                sig.params.push(cranelift_codegen::ir::AbiParam::new(
                    param.param_type.to_cranelift().unwrap(),
                ));
            }
            for ret in &function.signature.returns {
                sig.returns.push(cranelift_codegen::ir::AbiParam::new(
                    ret.to_cranelift().unwrap(),
                ));
            }

            let func_id = self
                .module
                .declare_function(name, cranelift_module::Linkage::Export, &sig)
                .map_err(|e| {
                    IrError::CraneliftError(format!("Failed to declare function: {}", e))
                })?;
            func_ids.insert(name.clone(), func_id);
        }

        for (name, function) in &contract.functions {
            let func_id = func_ids.get(name).unwrap();
            let mut clif_func = Function::new();
            clif_func.signature = self
                .module
                .declarations()
                .get_function_decl(*func_id)
                .signature
                .clone();

            let mut ctx = CodegenContext::new(&mut clif_func);
            let mut func_builder = ctx.func_builder();

            let mut block_map = HashMap::new();
            let mut variables = HashMap::new();
            let mut ssa_values = HashMap::new();

            for (block_id, _) in &function.body.blocks {
                let clif_block = func_builder.create_block();
                block_map.insert(*block_id, clif_block);
            }

            let entry_clif_block = block_map.get(&function.body.entry_block).unwrap();
            func_builder.append_block_params_for_function_params(*entry_clif_block);
            func_builder.switch_to_block(*entry_clif_block);

            for (i, param) in function.signature.params.iter().enumerate() {
                let var = Variable::new(i);
                func_builder.declare_var(var, param.param_type.to_cranelift().unwrap());
                let val = func_builder.block_params(*entry_clif_block)[i];
                func_builder.def_var(var, val);
                variables.insert(VarId(i as u32), var);
                ssa_values.insert(
                    crate::values::Value::Param(crate::values::ParamId(i as u32)),
                    val,
                );
            }

            for (block_id, block) in &function.body.blocks {
                let clif_block = *block_map.get(block_id).unwrap();
                func_builder.switch_to_block(clif_block);

                for inst in &block.instructions {
                    lower_instruction(inst, &variables, &mut ssa_values, &mut func_builder)?;
                }

                if !matches!(block.terminator, crate::block::Terminator::Invalid) {
                    lower_terminator(
                        &block.terminator,
                        &ssa_values,
                        &mut func_builder,
                        &block_map,
                    )?;
                }
            }

            func_builder.seal_all_blocks();
            func_builder.finalize();

            let mut context = Context::for_function(clif_func);
            self.module
                .define_function(*func_id, &mut context)
                .map_err(|e| {
                    IrError::CraneliftError(format!("Failed to define function: {}", e))
                })?;
        }

        let product = self.module.finish();
        let obj_bytes = product
            .emit()
            .map_err(|e| IrError::CraneliftError(format!("Failed to emit object: {}", e)))?;

        Ok(obj_bytes)
    }
}
