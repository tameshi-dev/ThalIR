use super::{IRContext, IRRegistry};
use crate::{
    block::{BlockId, Terminator},
    function::{
        DataLocation, Function, FunctionSignature, LocalId, LocalVariable, Mutability, Parameter,
        Visibility,
    },
    instructions::{ContextVariable, Instruction, StorageKey},
    types::Type,
    values::{Constant, ParamId, SourceLocation, Value},
    IrError, Result,
};
use std::collections::HashSet;

#[allow(dead_code)]

pub struct FunctionBuilderCursor<'a> {
    contract_name: String,
    function: Function,
    context: &'a mut IRContext,
    registry: &'a mut IRRegistry,
    created_blocks: HashSet<BlockId>,
    current_source_location: Option<SourceLocation>,
}

impl<'a> FunctionBuilderCursor<'a> {
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
            created_blocks: HashSet::new(),
            current_source_location: None,
        }
    }

    pub fn set_source_location(&mut self, location: SourceLocation) {
        self.current_source_location = Some(location);
    }

    pub fn clear_source_location(&mut self) {
        self.current_source_location = None;
    }

    pub fn init_cursor(&mut self) {}

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

    pub fn create_block(&mut self) -> BlockId {
        let block_id = self.function.body.create_block();
        self.created_blocks.insert(block_id);
        block_id
    }

    pub fn entry_block(&self) -> BlockId {
        self.function.body.entry_block
    }

    pub fn switch_to_block(&mut self, block_id: BlockId) -> Result<()> {
        if !self.created_blocks.contains(&block_id) && block_id != self.function.body.entry_block {
            return Err(IrError::BuilderError(format!(
                "Block {:?} does not exist",
                block_id
            )));
        }

        self.context.set_current_block(block_id);
        Ok(())
    }

    pub fn current_block(&self) -> Option<BlockId> {
        self.context.current_block()
    }

    pub fn is_terminated(&self) -> bool {
        if let Some(block_id) = self.current_block() {
            self.function
                .body
                .blocks
                .get(&block_id)
                .map(|b| b.is_terminated())
                .unwrap_or(false)
        } else {
            false
        }
    }

    pub fn ins(&mut self) -> Result<FunctionInstBuilder<'_>> {
        let block_id = self.current_block().ok_or_else(|| {
            IrError::BuilderError("No current block - call switch_to_block first".into())
        })?;

        Ok(FunctionInstBuilder {
            block_id,
            function: &mut self.function,
            context: self.context,
            source_location: self.current_source_location.clone(),
        })
    }

    pub fn local(&mut self, name: &str, ty: Type) -> Value {
        let var_id = self.context.ssa().get_or_create_var(name);
        self.function.body.locals.push(LocalVariable {
            id: LocalId(self.function.body.locals.len() as u32),
            name: name.to_string(),
            var_type: ty,
            location: DataLocation::Memory,
        });
        Value::Variable(var_id)
    }

    pub fn get_param(&self, index: usize) -> Value {
        Value::Param(ParamId(index as u32))
    }

    pub fn build(self) -> Result<Function> {
        for block_id in &self.created_blocks {
            if let Some(block) = self.function.body.blocks.get(block_id) {
                if !block.is_terminated() {
                    return Err(IrError::BuilderError(format!(
                        "Block {:?} is not terminated",
                        block_id
                    )));
                }
            }
        }

        self.registry
            .add_function(self.contract_name.clone(), self.function.clone())?;
        Ok(self.function)
    }
}

pub struct FunctionInstBuilder<'a> {
    block_id: BlockId,
    function: &'a mut Function,
    #[allow(dead_code)]
    context: &'a mut IRContext,
    source_location: Option<SourceLocation>,
}

impl<'a> FunctionInstBuilder<'a> {
    pub fn constant_bool(&mut self, value: bool) -> Value {
        Value::Constant(Constant::Bool(value))
    }

    pub fn constant_uint(&mut self, value: u64, bits: u16) -> Value {
        use num_bigint::BigUint;
        Value::Constant(Constant::Uint(BigUint::from(value), bits))
    }

    pub fn constant_int(&mut self, value: i64, bits: u16) -> Value {
        use num_bigint::BigInt;
        Value::Constant(Constant::Int(BigInt::from(value), bits))
    }

    pub fn add(&mut self, left: Value, right: Value, ty: Type) -> Value {
        let result = self.next_value();
        let inst = Instruction::Add {
            result: result.clone(),
            left,
            right,
            ty,
        };
        self.insert_inst(inst);
        result
    }

    pub fn sub(&mut self, left: Value, right: Value, ty: Type) -> Value {
        let result = self.next_value();
        let inst = Instruction::Sub {
            result: result.clone(),
            left,
            right,
            ty,
        };
        self.insert_inst(inst);
        result
    }

    pub fn lt(&mut self, left: Value, right: Value) -> Value {
        let result = self.next_value();
        let inst = Instruction::Lt {
            result: result.clone(),
            left,
            right,
        };
        self.insert_inst(inst);
        result
    }

    pub fn eq(&mut self, left: Value, right: Value) -> Value {
        let result = self.next_value();
        let inst = Instruction::Eq {
            result: result.clone(),
            left,
            right,
        };
        self.insert_inst(inst);
        result
    }

    pub fn gt(&mut self, left: Value, right: Value) -> Value {
        let result = self.next_value();
        let inst = Instruction::Gt {
            result: result.clone(),
            left,
            right,
        };
        self.insert_inst(inst);
        result
    }

    pub fn mul(&mut self, left: Value, right: Value, ty: Type) -> Value {
        let result = self.next_value();
        let inst = Instruction::Mul {
            result: result.clone(),
            left,
            right,
            ty,
        };
        self.insert_inst(inst);
        result
    }

    pub fn div(&mut self, left: Value, right: Value, ty: Type) -> Value {
        let result = self.next_value();
        let inst = Instruction::Div {
            result: result.clone(),
            left,
            right,
            ty,
        };
        self.insert_inst(inst);
        result
    }

    pub fn not(&mut self, value: Value) -> Value {
        let result = self.next_value();
        let inst = Instruction::Not {
            result: result.clone(),
            operand: value,
        };
        self.insert_inst(inst);
        result
    }

    pub fn and(&mut self, left: Value, right: Value) -> Value {
        let result = self.next_value();
        let inst = Instruction::And {
            result: result.clone(),
            left,
            right,
        };
        self.insert_inst(inst);
        result
    }

    pub fn or(&mut self, left: Value, right: Value) -> Value {
        let result = self.next_value();
        let inst = Instruction::Or {
            result: result.clone(),
            left,
            right,
        };
        self.insert_inst(inst);
        result
    }

    pub fn sload(&mut self, key: Value) -> Value {
        let result = self.next_value();
        let inst = Instruction::StorageLoad {
            result: result.clone(),
            key: StorageKey::Dynamic(key),
        };
        self.insert_inst(inst);
        result
    }

    pub fn sstore(&mut self, key: Value, value: Value) -> Result<()> {
        let inst = Instruction::StorageStore {
            key: StorageKey::Dynamic(key),
            value,
        };
        self.insert_inst(inst);
        Ok(())
    }

    pub fn branch(
        &mut self,
        condition: Value,
        then_block: BlockId,
        else_block: BlockId,
    ) -> Result<()> {
        let term = Terminator::Branch {
            condition,
            then_block,
            then_args: Vec::new(),
            else_block,
            else_args: Vec::new(),
        };
        self.set_terminator(term)
    }

    pub fn jump(&mut self, target: BlockId) -> Result<()> {
        let term = Terminator::Jump(target, Vec::new());
        self.set_terminator(term)
    }

    pub fn return_value(&mut self, value: Value) -> Result<()> {
        let term = Terminator::Return(Some(value));
        self.set_terminator(term)
    }

    pub fn return_void(&mut self) -> Result<()> {
        let term = Terminator::Return(None);
        self.set_terminator(term)
    }

    pub fn msg_sender(&mut self) -> Value {
        let result = self.next_value();
        let inst = Instruction::GetContext {
            result: result.clone(),
            var: ContextVariable::MsgSender,
        };
        self.insert_inst(inst);
        result
    }

    pub fn msg_value(&mut self) -> Value {
        let result = self.next_value();
        let inst = Instruction::GetContext {
            result: result.clone(),
            var: ContextVariable::MsgValue,
        };
        self.insert_inst(inst);
        result
    }

    fn insert_inst(&mut self, inst: Instruction) {
        if let Some(block) = self.function.body.blocks.get_mut(&self.block_id) {
            if let Some(ref location) = self.source_location {
                let index = block.instructions.len();
                block
                    .metadata
                    .instruction_locations
                    .insert(index, location.clone());
            }
            block.instructions.push(inst);
        }
    }

    fn set_terminator(&mut self, term: Terminator) -> Result<()> {
        if let Some(block) = self.function.body.blocks.get_mut(&self.block_id) {
            if !matches!(block.terminator, Terminator::Invalid) {
                return Err(IrError::BuilderError("Block already terminated".into()));
            }
            block.terminator = term;
            Ok(())
        } else {
            Err(IrError::BuilderError("Block not found".into()))
        }
    }

    fn next_value(&mut self) -> Value {
        use crate::values::TempId;
        use std::sync::atomic::{AtomicU32, Ordering};

        static COUNTER: AtomicU32 = AtomicU32::new(1000);
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        Value::Temp(TempId(id))
    }
}
