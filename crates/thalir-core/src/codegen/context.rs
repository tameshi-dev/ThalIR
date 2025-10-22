use cranelift_codegen::ir::{self as clif_ir};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};

use crate::types::Type;

pub struct CodegenContext<'a> {
    pub func: &'a mut clif_ir::Function,
    pub builder_context: FunctionBuilderContext,
}

impl<'a> CodegenContext<'a> {
    pub fn new(func: &'a mut clif_ir::Function) -> Self {
        Self {
            func,
            builder_context: FunctionBuilderContext::new(),
        }
    }

    pub fn func_builder(&mut self) -> FunctionBuilder<'_> {
        FunctionBuilder::new(self.func, &mut self.builder_context)
    }

    pub fn get_clif_type(&self, ty: &Type) -> clif_ir::Type {
        match ty {
            Type::Bool => clif_ir::types::I8,
            Type::Uint(8) | Type::Int(8) => clif_ir::types::I8,
            Type::Uint(16) | Type::Int(16) => clif_ir::types::I16,
            Type::Uint(32) | Type::Int(32) => clif_ir::types::I32,
            Type::Uint(64) | Type::Int(64) => clif_ir::types::I64,
            Type::Uint(256) | Type::Int(256) | Type::Address => clif_ir::types::I128,

            _ => clif_ir::types::I64,
        }
    }
}
