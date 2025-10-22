use crate::{
    block::BasicBlock,
    contract::Contract,
    function::Function,
    instructions::Instruction,
    types::Type,
    values::{Constant, Value},
};
use std::fmt::Write;

pub fn format_contract(contract: &Contract) -> String {
    let mut output = String::new();

    writeln!(&mut output, "; Contract: {}", contract.name).unwrap();
    writeln!(&mut output, "; Version: {}", contract.metadata.version).unwrap();

    if !contract.storage_layout.slots.is_empty() {
        writeln!(&mut output, "; Storage Layout:").unwrap();
        for slot in &contract.storage_layout.slots {
            writeln!(
                &mut output,
                ";   {}: {} at slot {}",
                slot.name, slot.var_type, slot.slot
            )
            .unwrap();
        }
    }

    writeln!(&mut output).unwrap();

    for (_name, function) in &contract.functions {
        write!(&mut output, "{}", format_function(function)).unwrap();
        writeln!(&mut output).unwrap();
    }

    output
}

pub fn format_function(function: &Function) -> String {
    let mut output = String::new();

    write!(&mut output, "function %{}", function.signature.name).unwrap();

    write!(&mut output, "(").unwrap();
    for (i, param) in function.signature.params.iter().enumerate() {
        if i > 0 {
            write!(&mut output, ", ").unwrap();
        }
        write!(&mut output, "{}", format_type(&param.param_type)).unwrap();
    }
    write!(&mut output, ")").unwrap();

    if !function.signature.returns.is_empty() {
        write!(&mut output, " -> ").unwrap();
        for (i, ret) in function.signature.returns.iter().enumerate() {
            if i > 0 {
                write!(&mut output, ", ").unwrap();
            }
            write!(&mut output, "{}", format_type(ret)).unwrap();
        }
    }

    writeln!(&mut output, " {{").unwrap();

    if function.visibility != crate::function::Visibility::Private {
        writeln!(&mut output, "    ; Visibility: {:?}", function.visibility).unwrap();
    }
    if function.mutability != crate::function::Mutability::NonPayable {
        writeln!(&mut output, "    ; Mutability: {:?}", function.mutability).unwrap();
    }

    for (_block_id, block) in &function.body.blocks {
        write!(&mut output, "{}", format_block(block)).unwrap();
    }

    writeln!(&mut output, "}}").unwrap();

    output
}

fn format_block(block: &BasicBlock) -> String {
    let mut output = String::new();

    write!(&mut output, "\n{}:", block.id).unwrap();
    if !block.params.is_empty() {
        write!(&mut output, "(").unwrap();
        for (i, param) in block.params.iter().enumerate() {
            if i > 0 {
                write!(&mut output, ", ").unwrap();
            }
            write!(
                &mut output,
                "{}: {}",
                param.name,
                format_type(&param.param_type)
            )
            .unwrap();
        }
        write!(&mut output, ")").unwrap();
    }
    writeln!(&mut output).unwrap();

    for inst in &block.instructions {
        writeln!(&mut output, "    {}", format_instruction(inst)).unwrap();
    }

    writeln!(&mut output, "    {}", format_terminator(&block.terminator)).unwrap();

    output
}

fn format_instruction(inst: &Instruction) -> String {
    use crate::instructions::CallTarget;
    use crate::instructions::ContextVariable;

    match inst {
        Instruction::Add {
            result,
            left,
            right,
            ty,
        } => {
            format!(
                "{} = iadd.{} {}, {}",
                format_value(result),
                format_type_short(ty),
                format_value(left),
                format_value(right)
            )
        }
        Instruction::Sub {
            result,
            left,
            right,
            ty,
        } => {
            format!(
                "{} = isub.{} {}, {}",
                format_value(result),
                format_type_short(ty),
                format_value(left),
                format_value(right)
            )
        }
        Instruction::Mul {
            result,
            left,
            right,
            ty,
        } => {
            format!(
                "{} = imul.{} {}, {}",
                format_value(result),
                format_type_short(ty),
                format_value(left),
                format_value(right)
            )
        }
        Instruction::Div {
            result,
            left,
            right,
            ty,
        } => {
            format!(
                "{} = idiv.{} {}, {}",
                format_value(result),
                format_type_short(ty),
                format_value(left),
                format_value(right)
            )
        }

        Instruction::Eq {
            result,
            left,
            right,
        } => {
            format!(
                "{} = icmp eq {}, {}",
                format_value(result),
                format_value(left),
                format_value(right)
            )
        }
        Instruction::Ne {
            result,
            left,
            right,
        } => {
            format!(
                "{} = icmp ne {}, {}",
                format_value(result),
                format_value(left),
                format_value(right)
            )
        }
        Instruction::Lt {
            result,
            left,
            right,
        } => {
            format!(
                "{} = icmp ult {}, {}",
                format_value(result),
                format_value(left),
                format_value(right)
            )
        }
        Instruction::Gt {
            result,
            left,
            right,
        } => {
            format!(
                "{} = icmp ugt {}, {}",
                format_value(result),
                format_value(left),
                format_value(right)
            )
        }
        Instruction::Le {
            result,
            left,
            right,
        } => {
            format!(
                "{} = icmp ule {}, {}",
                format_value(result),
                format_value(left),
                format_value(right)
            )
        }
        Instruction::Ge {
            result,
            left,
            right,
        } => {
            format!(
                "{} = icmp uge {}, {}",
                format_value(result),
                format_value(left),
                format_value(right)
            )
        }

        Instruction::StorageLoad { result, key } => {
            format!(
                "{} = storage_load {}",
                format_value(result),
                format_storage_key(key)
            )
        }
        Instruction::StorageStore { key, value } => {
            format!(
                "storage_store {}, {}",
                format_storage_key(key),
                format_value(value)
            )
        }

        Instruction::MappingLoad {
            result,
            mapping,
            key,
        } => {
            format!(
                "{} = mapping_load {}, {}",
                format_value(result),
                format_value(mapping),
                format_value(key)
            )
        }
        Instruction::MappingStore {
            mapping,
            key,
            value,
        } => {
            format!(
                "mapping_store {}, {}, {}",
                format_value(mapping),
                format_value(key),
                format_value(value)
            )
        }

        Instruction::GetContext { result, var } => {
            let var_str = match var {
                ContextVariable::MsgSender => "msg.sender",
                ContextVariable::MsgValue => "msg.value",
                ContextVariable::MsgData => "msg.data",
                ContextVariable::MsgSig => "msg.sig",
                ContextVariable::BlockNumber => "block.number",
                ContextVariable::BlockTimestamp => "block.timestamp",
                ContextVariable::BlockDifficulty => "block.difficulty",
                ContextVariable::BlockGasLimit => "block.gaslimit",
                ContextVariable::BlockCoinbase => "block.coinbase",
                ContextVariable::ChainId => "block.chainid",
                ContextVariable::BlockBaseFee => "block.basefee",
                ContextVariable::GasLeft => "gasleft",
                ContextVariable::TxOrigin => "tx.origin",
                ContextVariable::TxGasPrice => "tx.gasprice",
                ContextVariable::ThisAddress => "address(this)",
                ContextVariable::ThisBalance => "address(this).balance",
            };
            format!("{} = get_context {}", format_value(result), var_str)
        }

        Instruction::Call {
            result,
            target,
            args,
            value,
        } => {
            let target_str = match target {
                CallTarget::External(addr) => format!("{}(", format_value(addr)),
                CallTarget::Internal(name) => format!("{}(", name),
                CallTarget::Library(name) => format!("lib.{}(", name),
                CallTarget::Builtin(_) => format!("builtin("),
            };

            let args_str = args.iter().map(format_value).collect::<Vec<_>>().join(", ");
            let value_str = value
                .as_ref()
                .map(|v| format!(", value: {}", format_value(v)))
                .unwrap_or_default();

            if matches!(target, CallTarget::External(_)) {
                format!(
                    "{} = call_ext {}{}){}",
                    format_value(result),
                    target_str,
                    args_str,
                    value_str
                )
            } else {
                format!(
                    "{} = call {}{}){}",
                    format_value(result),
                    target_str,
                    args_str,
                    value_str
                )
            }
        }

        Instruction::Require { condition, message } => {
            format!("require {}, \"{}\"", format_value(condition), message)
        }
        Instruction::Assert { condition, message } => {
            format!("assert {}, \"{}\"", format_value(condition), message)
        }
        Instruction::Revert { message } => {
            format!("revert \"{}\"", message)
        }

        _ => format!("{:?}", inst),
    }
}

fn format_terminator(term: &crate::block::Terminator) -> String {
    use crate::block::Terminator;

    match term {
        Terminator::Jump(block, args) => {
            if args.is_empty() {
                format!("jump {}", block)
            } else {
                format!(
                    "jump {}({})",
                    block,
                    args.iter().map(format_value).collect::<Vec<_>>().join(", ")
                )
            }
        }
        Terminator::Branch {
            condition,
            then_block,
            then_args,
            else_block,
            else_args,
        } => {
            format!(
                "brif {}, {}({}), {}({})",
                format_value(condition),
                then_block,
                then_args
                    .iter()
                    .map(format_value)
                    .collect::<Vec<_>>()
                    .join(", "),
                else_block,
                else_args
                    .iter()
                    .map(format_value)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
        Terminator::Return(None) => "return".to_string(),
        Terminator::Return(Some(val)) => format!("return {}", format_value(val)),
        Terminator::Revert(msg) => format!("revert \"{}\"", msg),
        _ => format!("{:?}", term),
    }
}

fn format_value(val: &Value) -> String {
    match val {
        Value::Variable(id) => id.to_string(),
        Value::Temp(id) => id.to_string(),
        Value::Param(id) => id.to_string(),
        Value::Constant(c) => format_constant(c),
        _ => format!("{:?}", val),
    }
}

fn format_constant(c: &Constant) -> String {
    match c {
        Constant::Bool(b) => b.to_string(),
        Constant::Uint(val, bits) => format!("{}u{}", val, bits),
        Constant::Int(val, bits) => format!("{}i{}", val, bits),
        Constant::Address(addr) => format!("0x{}", hex::encode(addr)),
        _ => format!("{}", c),
    }
}

fn format_type(ty: &Type) -> String {
    match ty {
        Type::Bool => "i8".to_string(),
        Type::Uint(256) | Type::Int(256) => "i256".to_string(),
        Type::Uint(bits) => format!("u{}", bits),
        Type::Int(bits) => format!("i{}", bits),
        Type::Address => "i160".to_string(),
        _ => format!("{}", ty),
    }
}

fn format_type_short(ty: &Type) -> String {
    match ty {
        Type::Uint(256) => "u256".to_string(),
        Type::Int(256) => "i256".to_string(),
        Type::Uint(bits) => format!("u{}", bits),
        Type::Int(bits) => format!("i{}", bits),
        _ => format_type(ty),
    }
}

fn format_storage_key(key: &crate::instructions::StorageKey) -> String {
    match key {
        crate::instructions::StorageKey::Slot(n) => format!("slot_{}", n),
        crate::instructions::StorageKey::Computed(v) => format!("computed[{}]", format_value(v)),
        _ => format!("{:?}", key),
    }
}

mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }
}
