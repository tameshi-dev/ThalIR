use anyhow::Result;
use std::collections::HashMap;
use thalir_core::{
    analysis::PassManager,
    block::{BasicBlock, Terminator},
    contract::Contract,
    function::{Function, Mutability, Visibility},
    instructions::{Instruction, StorageKey},
    types::Type,
    values::{Constant, Value},
    ObfuscationConfig, ObfuscationMapping, ObfuscationPass,
};

fn format_bytes(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>()
}

pub struct ThalIREmitter {
    pub(crate) contracts: Vec<Contract>,
}

pub struct SSAContext {
    next_value: u32,
    value_map: HashMap<Value, u32>,
}

impl SSAContext {
    pub fn new() -> Self {
        Self {
            next_value: 0,
            value_map: HashMap::new(),
        }
    }

    pub fn reset(&mut self) {
        self.next_value = 0;
        self.value_map.clear();
    }

    pub fn get_or_allocate(&mut self, value: &Value) -> u32 {
        if let Some(&v) = self.value_map.get(value) {
            v
        } else {
            let v = self.next_value;
            self.value_map.insert(value.clone(), v);
            self.next_value += 1;
            v
        }
    }

    pub fn allocate_new(&mut self) -> u32 {
        let v = self.next_value;
        self.next_value += 1;
        v
    }

    pub fn allocate_temp(&mut self, value: Value) -> u32 {
        self.get_or_allocate(&value)
    }
}

impl ThalIREmitter {
    pub fn new(contracts: Vec<Contract>) -> Self {
        Self { contracts }
    }

    pub fn with_obfuscation(
        mut contracts: Vec<Contract>,
        obf_config: ObfuscationConfig,
    ) -> Result<(Self, Option<ObfuscationMapping>)> {
        let mut manager = PassManager::new();
        manager.register_pass(ObfuscationPass::new(obf_config.clone()));

        for contract in &mut contracts {
            manager.run_all(contract)?;
        }

        let mapping = if obf_config.retain_mapping {
            manager
                .get_pass::<ObfuscationPass>()
                .map(|pass| pass.export_mapping())
        } else {
            None
        };

        Ok((Self::new(contracts), mapping))
    }

    pub fn emit_to_string(&self, with_types: bool) -> String {
        let mut output = String::new();

        for contract in &self.contracts {
            self.print_contract(&mut output, contract, with_types);
        }

        output
    }

    fn print_contract(&self, output: &mut String, contract: &Contract, with_types: bool) {
        output.push_str(&format!("contract {} {{\n", contract.name));

        if !contract.storage_layout.slots.is_empty() {
            output.push_str("\n  // Storage Layout\n");
            for var in &contract.storage_layout.slots {
                output.push_str(&format!(
                    "  slot {} = {}: {}\n",
                    var.slot,
                    var.name,
                    self.format_type(&var.var_type)
                ));
            }
        }

        let mut ssa = SSAContext::new();
        for (name, function) in &contract.functions {
            output.push_str("\n");
            self.print_function(output, name, function, &mut ssa, with_types);
        }

        output.push_str("}\n");
    }

    fn print_function(
        &self,
        output: &mut String,
        name: &str,
        function: &Function,
        ssa: &mut SSAContext,
        _with_types: bool,
    ) {
        ssa.reset();

        let param_vnums: Vec<u32> = (0..function.signature.params.len())
            .map(|_| ssa.allocate_new())
            .collect();

        let param_types: Vec<String> = function
            .signature
            .params
            .iter()
            .map(|p| self.format_type(&p.param_type))
            .collect();

        let return_type = if !function.signature.returns.is_empty() {
            format!(
                " -> {}",
                function
                    .signature
                    .returns
                    .iter()
                    .map(|t| self.format_type(t))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        } else {
            String::new()
        };

        let visibility = match function.visibility {
            Visibility::Public => "public",
            Visibility::External => "external",
            Visibility::Internal => "internal",
            Visibility::Private => "private",
        };

        let mutability = match function.mutability {
            Mutability::Pure => "pure",
            Mutability::View => "view",
            Mutability::Payable => "payable",
            Mutability::NonPayable => "",
        };

        output.push_str(&format!(
            "  function %{}({}){} {} {} {{\n",
            name,
            param_types.join(", "),
            return_type,
            visibility,
            mutability
        ));

        if let Some(entry_block) = function.body.blocks.get(&function.body.entry_block) {
            output.push_str(&format!("  block{}(", entry_block.id.0));
            for (i, param) in function.signature.params.iter().enumerate() {
                if i > 0 {
                    output.push_str(", ");
                }
                output.push_str(&format!(
                    "v{}: {}",
                    param_vnums[i],
                    self.format_type(&param.param_type)
                ));
            }
            output.push_str("):\n");

            self.print_block_body(output, entry_block, ssa, &param_vnums);

            for (block_id, block) in &function.body.blocks {
                if block_id != &function.body.entry_block {
                    output.push_str(&format!("\n  block{}:\n", block.id.0));
                    self.print_block_body(output, block, ssa, &param_vnums);
                }
            }
        }

        output.push_str("  }\n");
    }

    fn print_block_body(
        &self,
        output: &mut String,
        block: &BasicBlock,
        ssa: &mut SSAContext,
        param_vnums: &[u32],
    ) {
        for inst in &block.instructions {
            let inst_str = self.format_instruction(inst, ssa, param_vnums);
            output.push_str(&format!("    {}\n", inst_str));
        }

        match &block.terminator {
            Terminator::Return(None) => {
                output.push_str("    return\n");
            }
            Terminator::Return(Some(val)) => {
                let v = self.format_value(val, ssa, param_vnums);
                output.push_str(&format!("    return {}\n", v));
            }
            Terminator::Jump(target, args) => {
                if args.is_empty() {
                    output.push_str(&format!("    jump block{}\n", target.0));
                } else {
                    let args_str: Vec<String> = args
                        .iter()
                        .map(|v| self.format_value(v, ssa, param_vnums))
                        .collect();
                    output.push_str(&format!(
                        "    jump block{}({})\n",
                        target.0,
                        args_str.join(", ")
                    ));
                }
            }
            Terminator::Branch {
                condition,
                then_block,
                else_block,
                then_args,
                else_args,
            } => {
                let cond = self.format_value(condition, ssa, param_vnums);
                if then_args.is_empty() && else_args.is_empty() {
                    output.push_str(&format!(
                        "    brz {}, block{}, block{}\n",
                        cond, else_block.0, then_block.0
                    ));
                } else {
                    output.push_str(&format!(
                        "    brz {}, block{}, block{}\n",
                        cond, else_block.0, then_block.0
                    ));
                }
            }
            _ => {}
        }
    }

    pub fn format_instruction(
        &self,
        inst: &Instruction,
        ssa: &mut SSAContext,
        param_vnums: &[u32],
    ) -> String {
        match inst {
            Instruction::Add {
                result,
                left,
                right,
                ty,
            } => {
                let result_v = ssa.allocate_temp(result.clone());
                let left_v = self.format_value(left, ssa, param_vnums);
                let right_v = self.format_value(right, ssa, param_vnums);
                format!(
                    "v{} = iadd.{} {}, {}",
                    result_v,
                    self.type_suffix(ty),
                    left_v,
                    right_v
                )
            }
            Instruction::Sub {
                result,
                left,
                right,
                ty,
            } => {
                let result_v = ssa.allocate_temp(result.clone());
                let left_v = self.format_value(left, ssa, param_vnums);
                let right_v = self.format_value(right, ssa, param_vnums);
                format!(
                    "v{} = isub.{} {}, {}",
                    result_v,
                    self.type_suffix(ty),
                    left_v,
                    right_v
                )
            }
            Instruction::Mul {
                result,
                left,
                right,
                ty,
            } => {
                let result_v = ssa.allocate_temp(result.clone());
                let left_v = self.format_value(left, ssa, param_vnums);
                let right_v = self.format_value(right, ssa, param_vnums);
                format!(
                    "v{} = imul.{} {}, {}",
                    result_v,
                    self.type_suffix(ty),
                    left_v,
                    right_v
                )
            }
            Instruction::Div {
                result,
                left,
                right,
                ty,
            } => {
                let result_v = ssa.allocate_temp(result.clone());
                let left_v = self.format_value(left, ssa, param_vnums);
                let right_v = self.format_value(right, ssa, param_vnums);
                format!(
                    "v{} = udiv.{} {}, {}",
                    result_v,
                    self.type_suffix(ty),
                    left_v,
                    right_v
                )
            }
            Instruction::Mod {
                result,
                left,
                right,
                ty,
            } => {
                let result_v = ssa.allocate_temp(result.clone());
                let left_v = self.format_value(left, ssa, param_vnums);
                let right_v = self.format_value(right, ssa, param_vnums);
                format!(
                    "v{} = urem.{} {}, {}",
                    result_v,
                    self.type_suffix(ty),
                    left_v,
                    right_v
                )
            }
            Instruction::StorageStore { key, value } => {
                let key_v = self.format_storage_key(key, ssa);
                let val_v = self.format_value(value, ssa, param_vnums);
                format!("sstore {}, {}", key_v, val_v)
            }
            Instruction::StorageLoad { result: _, key } => {
                let result_v = ssa.allocate_new();
                let key_v = self.format_storage_key(key, ssa);
                format!("v{} = sload {}", result_v, key_v)
            }
            Instruction::Eq {
                result,
                left,
                right,
            } => {
                let result_v = ssa.allocate_temp(result.clone());
                let left_v = self.format_value(left, ssa, param_vnums);
                let right_v = self.format_value(right, ssa, param_vnums);
                format!("v{} = icmp eq {}, {}", result_v, left_v, right_v)
            }
            Instruction::Ne {
                result,
                left,
                right,
            } => {
                let result_v = ssa.allocate_temp(result.clone());
                let left_v = self.format_value(left, ssa, param_vnums);
                let right_v = self.format_value(right, ssa, param_vnums);
                format!("v{} = icmp ne {}, {}", result_v, left_v, right_v)
            }
            Instruction::Lt {
                result,
                left,
                right,
            } => {
                let result_v = ssa.allocate_temp(result.clone());
                let left_v = self.format_value(left, ssa, param_vnums);
                let right_v = self.format_value(right, ssa, param_vnums);
                format!("v{} = icmp ult {}, {}", result_v, left_v, right_v)
            }
            Instruction::Le {
                result,
                left,
                right,
            } => {
                let result_v = ssa.allocate_temp(result.clone());
                let left_v = self.format_value(left, ssa, param_vnums);
                let right_v = self.format_value(right, ssa, param_vnums);
                format!("v{} = icmp ule {}, {}", result_v, left_v, right_v)
            }
            Instruction::Gt {
                result,
                left,
                right,
            } => {
                let result_v = ssa.allocate_temp(result.clone());
                let left_v = self.format_value(left, ssa, param_vnums);
                let right_v = self.format_value(right, ssa, param_vnums);
                format!("v{} = icmp ugt {}, {}", result_v, left_v, right_v)
            }
            Instruction::Ge {
                result,
                left,
                right,
            } => {
                let result_v = ssa.allocate_temp(result.clone());
                let left_v = self.format_value(left, ssa, param_vnums);
                let right_v = self.format_value(right, ssa, param_vnums);
                format!("v{} = icmp uge {}, {}", result_v, left_v, right_v)
            }
            Instruction::Select {
                result,
                condition,
                then_val,
                else_val,
            } => {
                let result_v = ssa.allocate_temp(result.clone());
                let cond_v = self.format_value(condition, ssa, param_vnums);
                let then_v = self.format_value(then_val, ssa, param_vnums);
                let else_v = self.format_value(else_val, ssa, param_vnums);
                format!("v{} = select {}, {}, {}", result_v, cond_v, then_v, else_v)
            }
            Instruction::Call {
                result,
                target,
                args,
                value: _,
            } => {
                let result_v = ssa.allocate_temp(result.clone());
                let args_str: Vec<String> = args
                    .iter()
                    .map(|v| self.format_value(v, ssa, param_vnums))
                    .collect();
                match target {
                    thalir_core::instructions::CallTarget::Internal(name) => {
                        format!("v{} = call %{}({})", result_v, name, args_str.join(", "))
                    }
                    thalir_core::instructions::CallTarget::External(addr) => {
                        let addr_str = self.format_value(addr, ssa, param_vnums);
                        format!(
                            "v{} = call_ext {}({})",
                            result_v,
                            addr_str,
                            args_str.join(", ")
                        )
                    }
                    thalir_core::instructions::CallTarget::Library(name) => {
                        format!(
                            "v{} = call_lib %{}({})",
                            result_v,
                            name,
                            args_str.join(", ")
                        )
                    }
                    thalir_core::instructions::CallTarget::Builtin(builtin) => {
                        format!(
                            "v{} = call_builtin {:?}({})",
                            result_v,
                            builtin,
                            args_str.join(", ")
                        )
                    }
                }
            }
            Instruction::GetContext { result, var } => {
                let result_v = ssa.allocate_temp(result.clone());
                let var_name = match var {
                    thalir_core::instructions::ContextVariable::MsgSender => "msg.sender",
                    thalir_core::instructions::ContextVariable::MsgValue => "msg.value",
                    thalir_core::instructions::ContextVariable::MsgData => "msg.data",
                    thalir_core::instructions::ContextVariable::MsgSig => "msg.sig",
                    thalir_core::instructions::ContextVariable::BlockNumber => "block.number",
                    thalir_core::instructions::ContextVariable::BlockTimestamp => "block.timestamp",
                    thalir_core::instructions::ContextVariable::BlockDifficulty => {
                        "block.difficulty"
                    }
                    thalir_core::instructions::ContextVariable::BlockGasLimit => "block.gaslimit",
                    thalir_core::instructions::ContextVariable::BlockCoinbase => "block.coinbase",
                    thalir_core::instructions::ContextVariable::ChainId => "chain.id",
                    _ => "unknown",
                };
                format!("v{} = get_context {}", result_v, var_name)
            }
            Instruction::Assert { condition, message } => {
                let cond = self.format_value(condition, ssa, param_vnums);
                format!("assert {}, \"{}\"", cond, message)
            }
            Instruction::Require { condition, message } => {
                let cond = self.format_value(condition, ssa, param_vnums);
                format!("require {}, \"{}\"", cond, message)
            }
            Instruction::Revert { message } => {
                format!("revert \"{}\"", message)
            }
            Instruction::EmitEvent {
                event,
                topics,
                data,
            } => {
                let topics_str: Vec<String> = topics
                    .iter()
                    .map(|v| self.format_value(v, ssa, param_vnums))
                    .collect();
                let data_str: Vec<String> = data
                    .iter()
                    .map(|v| self.format_value(v, ssa, param_vnums))
                    .collect();
                if topics.is_empty() && data.is_empty() {
                    format!("emit event_{}", event.0)
                } else if topics.is_empty() {
                    format!("emit event_{}({})", event.0, data_str.join(", "))
                } else {
                    format!(
                        "emit event_{}[{}]({})",
                        event.0,
                        topics_str.join(", "),
                        data_str.join(", ")
                    )
                }
            }
            Instruction::MappingLoad {
                result,
                mapping,
                key,
            } => {
                let result_v = ssa.allocate_temp(result.clone());
                let mapping_v = self.format_value(mapping, ssa, param_vnums);
                let key_v = self.format_value(key, ssa, param_vnums);
                format!("v{} = mapping_load {}, {}", result_v, mapping_v, key_v)
            }
            Instruction::MappingStore {
                mapping,
                key,
                value,
            } => {
                let mapping_v = self.format_value(mapping, ssa, param_vnums);
                let key_v = self.format_value(key, ssa, param_vnums);
                let value_v = self.format_value(value, ssa, param_vnums);
                format!("mapping_store {}, {}, {}", mapping_v, key_v, value_v)
            }
            Instruction::ArrayLoad {
                result,
                array,
                index,
            } => {
                let result_v = ssa.allocate_temp(result.clone());
                let array_v = self.format_value(array, ssa, param_vnums);
                let index_v = self.format_value(index, ssa, param_vnums);
                format!("v{} = array_load {}, {}", result_v, array_v, index_v)
            }
            Instruction::ArrayStore {
                array,
                index,
                value,
            } => {
                let array_v = self.format_value(array, ssa, param_vnums);
                let index_v = self.format_value(index, ssa, param_vnums);
                let value_v = self.format_value(value, ssa, param_vnums);
                format!("array_store {}, {}, {}", array_v, index_v, value_v)
            }
            Instruction::ArrayLength { result, array } => {
                let result_v = ssa.allocate_temp(result.clone());
                let array_v = self.format_value(array, ssa, param_vnums);
                format!("v{} = array_length {}", result_v, array_v)
            }
            Instruction::ArrayPush { array, value } => {
                let array_v = self.format_value(array, ssa, param_vnums);
                let value_v = self.format_value(value, ssa, param_vnums);
                format!("array_push {}, {}", array_v, value_v)
            }
            Instruction::ArrayPop { result, array } => {
                let result_v = ssa.allocate_temp(result.clone());
                let array_v = self.format_value(array, ssa, param_vnums);
                format!("v{} = array_pop {}", result_v, array_v)
            }
            Instruction::And {
                result,
                left,
                right,
            } => {
                let result_v = ssa.allocate_temp(result.clone());
                let left_v = self.format_value(left, ssa, param_vnums);
                let right_v = self.format_value(right, ssa, param_vnums);
                format!("v{} = band {}, {}", result_v, left_v, right_v)
            }
            Instruction::Or {
                result,
                left,
                right,
            } => {
                let result_v = ssa.allocate_temp(result.clone());
                let left_v = self.format_value(left, ssa, param_vnums);
                let right_v = self.format_value(right, ssa, param_vnums);
                format!("v{} = bor {}, {}", result_v, left_v, right_v)
            }
            Instruction::Xor {
                result,
                left,
                right,
            } => {
                let result_v = ssa.allocate_temp(result.clone());
                let left_v = self.format_value(left, ssa, param_vnums);
                let right_v = self.format_value(right, ssa, param_vnums);
                format!("v{} = bxor {}, {}", result_v, left_v, right_v)
            }
            Instruction::Shl {
                result,
                value,
                shift,
            } => {
                let result_v = ssa.allocate_temp(result.clone());
                let value_v = self.format_value(value, ssa, param_vnums);
                let shift_v = self.format_value(shift, ssa, param_vnums);
                format!("v{} = ishl {}, {}", result_v, value_v, shift_v)
            }
            Instruction::Shr {
                result,
                value,
                shift,
            } => {
                let result_v = ssa.allocate_temp(result.clone());
                let value_v = self.format_value(value, ssa, param_vnums);
                let shift_v = self.format_value(shift, ssa, param_vnums);
                format!("v{} = ushr {}, {}", result_v, value_v, shift_v)
            }
            _ => format!("{:?}", inst),
        }
    }

    pub fn format_value(&self, value: &Value, ssa: &mut SSAContext, param_vnums: &[u32]) -> String {
        match value {
            Value::Param(id) => {
                if (id.0 as usize) < param_vnums.len() {
                    format!("v{}", param_vnums[id.0 as usize])
                } else {
                    format!("v{}", ssa.get_or_allocate(value))
                }
            }
            Value::Temp(_) => {
                format!("v{}", ssa.get_or_allocate(value))
            }
            Value::Variable(_) => {
                format!("v{}", ssa.get_or_allocate(value))
            }
            Value::BlockParam(_) => {
                format!("v{}", ssa.get_or_allocate(value))
            }
            Value::StorageRef(_) => {
                format!("sref_{}", ssa.get_or_allocate(value))
            }
            Value::MemoryRef(_) => {
                format!("mref_{}", ssa.get_or_allocate(value))
            }
            Value::Global(_) => {
                format!("global_{}", ssa.get_or_allocate(value))
            }
            Value::Register(_id) => {
                format!("reg_{}", ssa.get_or_allocate(value))
            }
            Value::Undefined => "undefined".to_string(),
            Value::Constant(c) => self.format_constant(c),
        }
    }

    fn format_constant(&self, c: &Constant) -> String {
        match c {
            Constant::Uint(val, bits) => format!("iconst.i{} {}", bits, val),
            Constant::Int(val, bits) => format!("iconst.i{} {}", bits, val),
            Constant::Bool(b) => format!("iconst.i1 {}", if *b { 1 } else { 0 }),
            Constant::Address(addr) => format!("iconst.i160 0x{}", format_bytes(addr)),
            Constant::Bytes(bytes) => format!("bconst 0x{}", format_bytes(bytes)),
            Constant::String(s) => format!("sconst \"{}\"", s),
            Constant::Null => "null".to_string(),
        }
    }

    fn format_storage_key(&self, key: &StorageKey, ssa: &mut SSAContext) -> String {
        match key {
            StorageKey::Slot(slot) => format!("iconst.i256 {}", slot),
            StorageKey::MappingKey { base, key } => {
                format!("mapping({}, {})", base, self.format_value(key, ssa, &[]))
            }
            StorageKey::ArrayElement { base, index } => {
                format!("array({}, {})", base, self.format_value(index, ssa, &[]))
            }
            StorageKey::Dynamic(val) => format!("dynamic({})", self.format_value(val, ssa, &[])),
            StorageKey::Computed(val) => format!("computed({})", self.format_value(val, ssa, &[])),
        }
    }

    pub fn format_type(&self, ty: &Type) -> String {
        match ty {
            Type::Uint(bits) => format!("i{}", bits),
            Type::Int(bits) => format!("i{}", bits),
            Type::Bool => "i1".to_string(),
            Type::Address => "i160".to_string(),
            Type::Bytes(size) => format!("bytes{}", size),
            Type::String => "string".to_string(),
            Type::Array(element, size) => {
                if let Some(s) = size {
                    format!("[{}; {}]", self.format_type(element), s)
                } else {
                    format!("[{}]", self.format_type(element))
                }
            }
            Type::Mapping(key, value) => {
                format!(
                    "mapping({} => {})",
                    self.format_type(key),
                    self.format_type(value)
                )
            }
            Type::Struct(id) => format!("struct_{:?}", id),
            Type::Enum(id) => format!("enum_{:?}", id),
            Type::Contract(id) => format!("contract_{:?}", id),
            Type::Function(_) => "function".to_string(),
            Type::StoragePointer(inner) => format!("storage_ptr<{}>", self.format_type(inner)),
            Type::MemoryPointer(inner) => format!("memory_ptr<{}>", self.format_type(inner)),
            Type::CalldataPointer(inner) => format!("calldata_ptr<{}>", self.format_type(inner)),
            Type::Bytes4 => "bytes4".to_string(),
            Type::Bytes20 => "bytes20".to_string(),
            Type::Bytes32 => "bytes32".to_string(),
            Type::ClifType(_) => "clif_type".to_string(),
        }
    }

    fn type_suffix(&self, ty: &Type) -> String {
        match ty {
            Type::Uint(bits) | Type::Int(bits) => format!("i{}", bits),
            Type::Bool => "i1".to_string(),
            Type::Address => "i160".to_string(),
            _ => "i256".to_string(),
        }
    }
}
