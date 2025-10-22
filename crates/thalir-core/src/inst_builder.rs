use crate::{
    block::BlockId,
    cursor::FuncCursor,
    instructions::Instruction,
    types::Type,
    values::{Constant, Value},
    Result,
};

pub struct InstBuilder<'b, 'a: 'b> {
    cursor: &'b mut FuncCursor<'a>,
}

impl<'b, 'a> InstBuilder<'b, 'a> {
    pub fn new(cursor: &'b mut FuncCursor<'a>) -> Self {
        Self { cursor }
    }

    pub fn add(self, left: Value, right: Value, ty: Type) -> Result<Value> {
        let result = self.next_value();
        let inst = Instruction::Add {
            result: result.clone(),
            left,
            right,
            ty,
        };
        self.cursor.insert_inst(inst)?;
        Ok(result)
    }

    pub fn sub(self, left: Value, right: Value, ty: Type) -> Result<Value> {
        let result = self.next_value();
        let inst = Instruction::Sub {
            result: result.clone(),
            left,
            right,
            ty,
        };
        self.cursor.insert_inst(inst)?;
        Ok(result)
    }

    pub fn mul(self, left: Value, right: Value, ty: Type) -> Result<Value> {
        let result = self.next_value();
        let inst = Instruction::Mul {
            result: result.clone(),
            left,
            right,
            ty,
        };
        self.cursor.insert_inst(inst)?;
        Ok(result)
    }

    pub fn div(self, left: Value, right: Value, ty: Type) -> Result<Value> {
        let result = self.next_value();
        let inst = Instruction::Div {
            result: result.clone(),
            left,
            right,
            ty,
        };
        self.cursor.insert_inst(inst)?;
        Ok(result)
    }

    pub fn eq(self, left: Value, right: Value) -> Result<Value> {
        let result = self.next_value();
        let inst = Instruction::Eq {
            result: result.clone(),
            left,
            right,
        };
        self.cursor.insert_inst(inst)?;
        Ok(result)
    }

    pub fn ne(self, left: Value, right: Value) -> Result<Value> {
        let result = self.next_value();
        let inst = Instruction::Ne {
            result: result.clone(),
            left,
            right,
        };
        self.cursor.insert_inst(inst)?;
        Ok(result)
    }

    pub fn lt(self, left: Value, right: Value) -> Result<Value> {
        let result = self.next_value();
        let inst = Instruction::Lt {
            result: result.clone(),
            left,
            right,
        };
        self.cursor.insert_inst(inst)?;
        Ok(result)
    }

    pub fn gt(self, left: Value, right: Value) -> Result<Value> {
        let result = self.next_value();
        let inst = Instruction::Gt {
            result: result.clone(),
            left,
            right,
        };
        self.cursor.insert_inst(inst)?;
        Ok(result)
    }

    pub fn le(self, left: Value, right: Value) -> Result<Value> {
        let result = self.next_value();
        let inst = Instruction::Le {
            result: result.clone(),
            left,
            right,
        };
        self.cursor.insert_inst(inst)?;
        Ok(result)
    }

    pub fn ge(self, left: Value, right: Value) -> Result<Value> {
        let result = self.next_value();
        let inst = Instruction::Ge {
            result: result.clone(),
            left,
            right,
        };
        self.cursor.insert_inst(inst)?;
        Ok(result)
    }

    pub fn constant_bool(self, value: bool) -> Value {
        Value::Constant(Constant::Bool(value))
    }

    pub fn constant_uint(self, value: u64, bits: u16) -> Value {
        use num_bigint::BigUint;
        Value::Constant(Constant::Uint(BigUint::from(value), bits))
    }

    pub fn constant_int(self, value: i64, bits: u16) -> Value {
        use num_bigint::BigInt;
        Value::Constant(Constant::Int(BigInt::from(value), bits))
    }

    pub fn constant_address(self, addr: [u8; 20]) -> Value {
        Value::Constant(Constant::Address(addr))
    }

    pub fn jump(self, target: BlockId, args: Vec<Value>) -> Result<()> {
        use crate::block::Terminator;
        self.cursor.set_terminator(Terminator::Jump(target, args))
    }

    pub fn branch(
        self,
        condition: Value,
        then_block: BlockId,
        then_args: Vec<Value>,
        else_block: BlockId,
        else_args: Vec<Value>,
    ) -> Result<()> {
        use crate::block::Terminator;
        self.cursor.set_terminator(Terminator::Branch {
            condition,
            then_block,
            then_args,
            else_block,
            else_args,
        })
    }

    pub fn return_value(self, value: Option<Value>) -> Result<()> {
        use crate::block::Terminator;
        self.cursor.set_terminator(Terminator::Return(value))
    }

    pub fn return_void(self) -> Result<()> {
        self.return_value(None)
    }

    pub fn revert(self, reason: String) -> Result<()> {
        use crate::block::Terminator;
        self.cursor.set_terminator(Terminator::Revert(reason))
    }

    pub fn msg_sender(self) -> Result<Value> {
        use crate::instructions::ContextVariable;
        let result = self.next_value();
        let inst = Instruction::GetContext {
            result: result.clone(),
            var: ContextVariable::MsgSender,
        };
        self.cursor.insert_inst(inst)?;
        Ok(result)
    }

    pub fn msg_value(self) -> Result<Value> {
        use crate::instructions::ContextVariable;
        let result = self.next_value();
        let inst = Instruction::GetContext {
            result: result.clone(),
            var: ContextVariable::MsgValue,
        };
        self.cursor.insert_inst(inst)?;
        Ok(result)
    }

    pub fn storage_load(self, key: Value) -> Result<Value> {
        use crate::instructions::StorageKey;
        let result = self.next_value();
        let inst = Instruction::StorageLoad {
            result: result.clone(),
            key: StorageKey::Dynamic(key),
        };
        self.cursor.insert_inst(inst)?;
        Ok(result)
    }

    pub fn storage_store(self, key: Value, value: Value) -> Result<()> {
        use crate::instructions::StorageKey;
        let inst = Instruction::StorageStore {
            key: StorageKey::Dynamic(key),
            value,
        };
        self.cursor.insert_inst(inst)
    }

    fn next_value(&self) -> Value {
        use crate::values::TempId;
        use std::sync::atomic::{AtomicU32, Ordering};

        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        Value::Temp(TempId(id))
    }
}
