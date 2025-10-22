use super::{BlockBuilder, IRContext, IRRegistry};
use crate::{
    block::BlockId,
    function::{Function, FunctionSignature, Mutability, Parameter, Visibility},
    types::Type,
    values::{ParamId, Value},
    Result,
};

pub struct FunctionBuilder<'a> {
    contract_name: String,
    function: Function,
    context: &'a mut IRContext,
    registry: &'a mut IRRegistry,
    current_block: Option<BlockId>,
    created_blocks: std::collections::HashSet<BlockId>,
}

impl<'a> FunctionBuilder<'a> {
    pub fn new(
        contract_name: String,
        name: String,
        context: &'a mut IRContext,
        registry: &'a mut IRRegistry,
    ) -> Self {
        let signature = FunctionSignature {
            name: name.clone(),
            params: Vec::new(),
            returns: Vec::new(),
            is_payable: false,
        };

        let function = Function::new(signature);

        Self {
            contract_name,
            function,
            context,
            registry,
            current_block: None,
            created_blocks: std::collections::HashSet::new(),
        }
    }

    pub fn param(&mut self, name: &str, ty: Type) -> &mut Self {
        self.function
            .signature
            .params
            .push(Parameter::new(name, ty));
        self
    }

    pub fn get_params(&self) -> &Vec<Parameter> {
        &self.function.signature.params
    }

    pub fn returns(&mut self, ty: Type) -> &mut Self {
        self.function.signature.returns = vec![ty];
        self
    }

    pub fn returns_multiple(&mut self, types: Vec<Type>) -> &mut Self {
        self.function.signature.returns = types;
        self
    }

    pub fn visibility(&mut self, vis: Visibility) -> &mut Self {
        self.function.visibility = vis;
        self
    }

    pub fn mutability(&mut self, mut_: Mutability) -> &mut Self {
        self.function.mutability = mut_;
        self
    }

    pub fn modifier(&mut self, _name: &str) -> &mut Self {
        self.function.modifiers.push(crate::contract::ModifierRef {
            id: crate::contract::ModifierId(0),
            arguments: Vec::new(),
        });
        self
    }

    pub fn create_block_id(&mut self) -> BlockId {
        let block_id = self.function.body.create_block();
        self.created_blocks.insert(block_id);
        block_id
    }

    pub fn switch_to_block(&mut self, block_id: BlockId) -> Result<BlockBuilder<'_>> {
        if !self.created_blocks.contains(&block_id) {
            return Err(crate::IrError::BuilderError(format!(
                "Block {} does not exist in function",
                block_id
            )));
        }

        self.current_block = Some(block_id);
        self.context.set_current_block(block_id);

        let qualified_func = format!("{}::{}", self.contract_name, self.function.signature.name);

        Ok(BlockBuilder::new(
            block_id,
            qualified_func,
            self.context,
            self.registry,
        ))
    }

    pub fn current_block(&self) -> Option<BlockId> {
        self.current_block
    }

    pub fn entry_block(&mut self) -> BlockBuilder<'_> {
        let block_id = self.function.body.entry_block;
        self.current_block = Some(block_id);
        self.context.set_current_block(block_id);
        self.created_blocks.insert(block_id);

        let qualified_func = format!("{}::{}", self.contract_name, self.function.signature.name);

        BlockBuilder::new(block_id, qualified_func, self.context, self.registry)
    }

    pub fn new_block(&mut self, _name: &str) -> BlockBuilder<'_> {
        self.block(_name)
    }

    pub fn block(&mut self, _name: &str) -> BlockBuilder<'_> {
        let block_id = self.function.body.create_block();
        self.current_block = Some(block_id);
        self.context.set_current_block(block_id);
        self.created_blocks.insert(block_id);

        let qualified_func = format!("{}::{}", self.contract_name, self.function.signature.name);

        BlockBuilder::new(block_id, qualified_func, self.context, self.registry)
    }

    pub fn block_with_id(&mut self, block_id: BlockId) -> BlockBuilder<'_> {
        self.current_block = Some(block_id);
        self.context.set_current_block(block_id);
        self.created_blocks.insert(block_id);

        let qualified_func = format!("{}::{}", self.contract_name, self.function.signature.name);

        BlockBuilder::new(block_id, qualified_func, self.context, self.registry)
    }

    pub fn local(&mut self, name: &str, ty: Type) -> Value {
        let var_id = self.context.ssa().get_or_create_var(name);
        self.function
            .body
            .locals
            .push(crate::function::LocalVariable {
                id: crate::function::LocalId(self.function.body.locals.len() as u32),
                name: name.to_string(),
                var_type: ty,
                location: crate::function::DataLocation::Memory,
            });
        Value::Variable(var_id)
    }

    pub fn get_param(&self, index: usize) -> Value {
        Value::Param(ParamId(index as u32))
    }

    pub fn current_function(&self) -> &Function {
        &self.function
    }

    pub fn build(self) -> Result<Function> {
        let qualified_name = format!("{}::{}", self.contract_name, self.function.signature.name);
        self.registry
            .add_function(self.contract_name.clone(), self.function)?;

        self.registry
            .get_function(&qualified_name)
            .ok_or_else(|| {
                crate::IrError::BuilderError(format!(
                    "Function {} not found after registration",
                    qualified_name
                ))
            })
            .map(|f| f.clone())
    }
}
