use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::ir::immediates::Offset32;
use cranelift_codegen::ir::types;
use cranelift_codegen::ir::{self as clif_ir, InstBuilder, MemFlags};
use cranelift_frontend::FunctionBuilder;
use std::collections::HashMap;

use crate::{
    block::Terminator,
    instructions::{CallTarget, ContextVariable, Instruction, Size, StorageKey},
    types::Type,
    values::{Constant, Value, VarId},
    IrError, Result,
};
use cranelift_frontend::Variable;

pub fn lower_instruction(
    inst: &Instruction,
    _variables: &HashMap<VarId, Variable>,
    ssa_values: &mut HashMap<Value, clif_ir::Value>,
    builder: &mut FunctionBuilder,
) -> Result<()> {
    match inst {
        Instruction::Add {
            result,
            left,
            right,
            ..
        } => {
            let left = ssa_values.get(left).unwrap();
            let right = ssa_values.get(right).unwrap();
            let res = builder.ins().iadd(*left, *right);
            ssa_values.insert(result.clone(), res);
        }
        Instruction::Sub {
            result,
            left,
            right,
            ..
        } => {
            let left = ssa_values.get(left).unwrap();
            let right = ssa_values.get(right).unwrap();
            let res = builder.ins().isub(*left, *right);
            ssa_values.insert(result.clone(), res);
        }
        Instruction::Mul {
            result,
            left,
            right,
            ..
        } => {
            let left = ssa_values.get(left).unwrap();
            let right = ssa_values.get(right).unwrap();
            let res = builder.ins().imul(*left, *right);
            ssa_values.insert(result.clone(), res);
        }
        Instruction::Div {
            result,
            left,
            right,
            ..
        } => {
            let left = ssa_values.get(left).unwrap();
            let right = ssa_values.get(right).unwrap();
            let res = builder.ins().udiv(*left, *right);
            ssa_values.insert(result.clone(), res);
        }
        Instruction::Mod {
            result,
            left,
            right,
            ..
        } => {
            let left = ssa_values.get(left).unwrap();
            let right = ssa_values.get(right).unwrap();
            let res = builder.ins().urem(*left, *right);
            ssa_values.insert(result.clone(), res);
        }
        Instruction::Eq {
            result,
            left,
            right,
        } => {
            let left = ssa_values.get(left).unwrap();
            let right = ssa_values.get(right).unwrap();
            let res = builder.ins().icmp(IntCC::Equal, *left, *right);
            ssa_values.insert(result.clone(), res);
        }
        Instruction::Ne {
            result,
            left,
            right,
        } => {
            let left = ssa_values.get(left).unwrap();
            let right = ssa_values.get(right).unwrap();
            let res = builder.ins().icmp(IntCC::NotEqual, *left, *right);
            ssa_values.insert(result.clone(), res);
        }
        Instruction::Lt {
            result,
            left,
            right,
        } => {
            let left = ssa_values.get(left).unwrap();
            let right = ssa_values.get(right).unwrap();
            let res = builder.ins().icmp(IntCC::SignedLessThan, *left, *right);
            ssa_values.insert(result.clone(), res);
        }
        Instruction::Gt {
            result,
            left,
            right,
        } => {
            let left = ssa_values.get(left).unwrap();
            let right = ssa_values.get(right).unwrap();
            let res = builder.ins().icmp(IntCC::SignedGreaterThan, *left, *right);
            ssa_values.insert(result.clone(), res);
        }
        Instruction::Le {
            result,
            left,
            right,
        } => {
            let left = ssa_values.get(left).unwrap();
            let right = ssa_values.get(right).unwrap();
            let res = builder
                .ins()
                .icmp(IntCC::SignedLessThanOrEqual, *left, *right);
            ssa_values.insert(result.clone(), res);
        }
        Instruction::Ge {
            result,
            left,
            right,
        } => {
            let left = ssa_values.get(left).unwrap();
            let right = ssa_values.get(right).unwrap();
            let res = builder
                .ins()
                .icmp(IntCC::SignedGreaterThanOrEqual, *left, *right);
            ssa_values.insert(result.clone(), res);
        }
        Instruction::Select {
            result,
            condition,
            then_val,
            else_val,
        } => {
            let cond = ssa_values.get(condition).unwrap();
            let then_v = ssa_values.get(then_val).unwrap();
            let else_v = ssa_values.get(else_val).unwrap();
            let res = builder.ins().select(*cond, *then_v, *else_v);
            ssa_values.insert(result.clone(), res);
        }

        Instruction::And {
            result,
            left,
            right,
        } => {
            let left = ssa_values.get(left).unwrap();
            let right = ssa_values.get(right).unwrap();
            let res = builder.ins().band(*left, *right);
            ssa_values.insert(result.clone(), res);
        }
        Instruction::Or {
            result,
            left,
            right,
        } => {
            let left = ssa_values.get(left).unwrap();
            let right = ssa_values.get(right).unwrap();
            let res = builder.ins().bor(*left, *right);
            ssa_values.insert(result.clone(), res);
        }
        Instruction::Xor {
            result,
            left,
            right,
        } => {
            let left = ssa_values.get(left).unwrap();
            let right = ssa_values.get(right).unwrap();
            let res = builder.ins().bxor(*left, *right);
            ssa_values.insert(result.clone(), res);
        }
        Instruction::Not { result, operand } => {
            let operand = ssa_values.get(operand).unwrap();
            let res = builder.ins().bnot(*operand);
            ssa_values.insert(result.clone(), res);
        }
        Instruction::Shl {
            result,
            value,
            shift,
        } => {
            let value = ssa_values.get(value).unwrap();
            let shift = ssa_values.get(shift).unwrap();
            let res = builder.ins().ishl(*value, *shift);
            ssa_values.insert(result.clone(), res);
        }
        Instruction::Shr {
            result,
            value,
            shift,
        } => {
            let value = ssa_values.get(value).unwrap();
            let shift = ssa_values.get(shift).unwrap();
            let res = builder.ins().ushr(*value, *shift);
            ssa_values.insert(result.clone(), res);
        }
        Instruction::Sar {
            result,
            value,
            shift,
        } => {
            let value = ssa_values.get(value).unwrap();
            let shift = ssa_values.get(shift).unwrap();
            let res = builder.ins().sshr(*value, *shift);
            ssa_values.insert(result.clone(), res);
        }

        Instruction::Pow { result, base, exp } => {
            let base = ssa_values.get(base).unwrap();
            let exp = ssa_values.get(exp).unwrap();
            let res = emit_runtime_call(builder, 0, 0, &[*base, *exp])?;
            ssa_values.insert(result.clone(), res);
        }

        Instruction::CheckedAdd {
            result,
            left,
            right,
            ..
        } => {
            let left = ssa_values.get(left).unwrap();
            let right = ssa_values.get(right).unwrap();
            let (res, overflow) = builder.ins().sadd_overflow(*left, *right);

            let overflow_block = builder.create_block();
            let continue_block = builder.create_block();

            builder
                .ins()
                .brif(overflow, overflow_block, &[], continue_block, &[]);

            builder.switch_to_block(overflow_block);
            builder.ins().trap(clif_ir::TrapCode::INTEGER_OVERFLOW);

            builder.switch_to_block(continue_block);
            ssa_values.insert(result.clone(), res);
        }
        Instruction::CheckedSub {
            result,
            left,
            right,
            ..
        } => {
            let left = ssa_values.get(left).unwrap();
            let right = ssa_values.get(right).unwrap();
            let (res, overflow) = builder.ins().ssub_overflow(*left, *right);

            let overflow_block = builder.create_block();
            let continue_block = builder.create_block();

            builder
                .ins()
                .brif(overflow, overflow_block, &[], continue_block, &[]);

            builder.switch_to_block(overflow_block);
            builder.ins().trap(clif_ir::TrapCode::INTEGER_OVERFLOW);

            builder.switch_to_block(continue_block);
            ssa_values.insert(result.clone(), res);
        }
        Instruction::CheckedMul {
            result,
            left,
            right,
            ..
        } => {
            let left = ssa_values.get(left).unwrap();
            let right = ssa_values.get(right).unwrap();
            let (res, overflow) = builder.ins().smul_overflow(*left, *right);

            let overflow_block = builder.create_block();
            let continue_block = builder.create_block();

            builder
                .ins()
                .brif(overflow, overflow_block, &[], continue_block, &[]);

            builder.switch_to_block(overflow_block);
            builder.ins().trap(clif_ir::TrapCode::INTEGER_OVERFLOW);

            builder.switch_to_block(continue_block);
            ssa_values.insert(result.clone(), res);
        }

        Instruction::Load { result, location } => {
            let addr = get_location_address(location, ssa_values, builder)?;
            let res = builder
                .ins()
                .load(types::I128, MemFlags::trusted(), addr, Offset32::new(0));
            ssa_values.insert(result.clone(), res);
        }
        Instruction::Store { location, value } => {
            let addr = get_location_address(location, ssa_values, builder)?;
            let value = ssa_values.get(value).unwrap();
            builder
                .ins()
                .store(MemFlags::trusted(), *value, addr, Offset32::new(0));
        }
        Instruction::Allocate {
            result,
            ty: _,
            size,
        } => {
            let size_val = match size {
                Size::Static(s) => builder.ins().iconst(types::I64, *s as i64),
                Size::Dynamic(v) => *ssa_values.get(v).unwrap(),
            };
            let res = emit_runtime_call(builder, 0, 1, &[size_val])?;
            ssa_values.insert(result.clone(), res);
        }
        Instruction::Copy { dest, src, size } => {
            let dest_addr = get_location_address(dest, ssa_values, builder)?;
            let src_addr = get_location_address(src, ssa_values, builder)?;
            let size = ssa_values.get(size).unwrap();
            emit_runtime_call_void(builder, 0, 2, &[dest_addr, src_addr, *size])?;
        }

        Instruction::StorageLoad { result, key } => {
            let key_val = get_storage_key_value(key, ssa_values, builder)?;
            let res = emit_runtime_call(builder, 1, 0, &[key_val])?;
            ssa_values.insert(result.clone(), res);
        }
        Instruction::StorageStore { key, value } => {
            let key_val = get_storage_key_value(key, ssa_values, builder)?;
            let value = ssa_values.get(value).unwrap();
            emit_runtime_call_void(builder, 1, 1, &[key_val, *value])?;
        }
        Instruction::StorageDelete { key } => {
            let key_val = get_storage_key_value(key, ssa_values, builder)?;
            emit_runtime_call_void(builder, 1, 2, &[key_val])?;
        }

        Instruction::MappingLoad {
            result,
            mapping,
            key,
        } => {
            let mapping = ssa_values.get(mapping).unwrap();
            let key = ssa_values.get(key).unwrap();
            let res = emit_runtime_call(builder, 2, 0, &[*mapping, *key])?;
            ssa_values.insert(result.clone(), res);
        }
        Instruction::MappingStore {
            mapping,
            key,
            value,
        } => {
            let mapping = ssa_values.get(mapping).unwrap();
            let key = ssa_values.get(key).unwrap();
            let value = ssa_values.get(value).unwrap();
            emit_runtime_call_void(builder, 2, 1, &[*mapping, *key, *value])?;
        }

        Instruction::ArrayLoad {
            result,
            array,
            index,
        } => {
            let array = ssa_values.get(array).unwrap();
            let index = ssa_values.get(index).unwrap();
            let element_size = builder.ins().iconst(types::I64, 32);
            let offset = builder.ins().imul(*index, element_size);
            let addr = builder.ins().iadd(*array, offset);
            let res = builder
                .ins()
                .load(types::I128, MemFlags::trusted(), addr, Offset32::new(0));
            ssa_values.insert(result.clone(), res);
        }
        Instruction::ArrayStore {
            array,
            index,
            value,
        } => {
            let array = ssa_values.get(array).unwrap();
            let index = ssa_values.get(index).unwrap();
            let value = ssa_values.get(value).unwrap();
            let element_size = builder.ins().iconst(types::I64, 32);
            let offset = builder.ins().imul(*index, element_size);
            let addr = builder.ins().iadd(*array, offset);
            builder
                .ins()
                .store(MemFlags::trusted(), *value, addr, Offset32::new(0));
        }
        Instruction::ArrayLength { result, array } => {
            let array = ssa_values.get(array).unwrap();
            let offset = builder.ins().iconst(types::I64, -32);
            let len_addr = builder.ins().iadd(*array, offset);
            let res =
                builder
                    .ins()
                    .load(types::I64, MemFlags::trusted(), len_addr, Offset32::new(0));
            ssa_values.insert(result.clone(), res);
        }
        Instruction::ArrayPush { array, value } => {
            let array = ssa_values.get(array).unwrap();
            let value = ssa_values.get(value).unwrap();
            emit_runtime_call_void(builder, 3, 0, &[*array, *value])?;
        }
        Instruction::ArrayPop { result, array } => {
            let array = ssa_values.get(array).unwrap();
            let res = emit_runtime_call(builder, 3, 1, &[*array])?;
            ssa_values.insert(result.clone(), res);
        }

        Instruction::Call {
            result,
            target,
            args,
            value,
        } => {
            let args_vals: Vec<_> = args
                .iter()
                .map(|arg| *ssa_values.get(arg).unwrap())
                .collect();
            let value_val = value.as_ref().map(|v| *ssa_values.get(v).unwrap());
            let res = emit_call(builder, target, &args_vals, value_val)?;
            ssa_values.insert(result.clone(), res);
        }
        Instruction::DelegateCall {
            result,
            target,
            selector: _,
            args,
        } => {
            let target = ssa_values.get(target).unwrap();
            let args_vals: Vec<_> = args
                .iter()
                .map(|arg| *ssa_values.get(arg).unwrap())
                .collect();
            let mut all_args = vec![*target];
            all_args.extend(args_vals);
            let res = emit_runtime_call(builder, 6, 1, &all_args)?;
            ssa_values.insert(result.clone(), res);
        }
        Instruction::StaticCall {
            result,
            target,
            selector: _,
            args,
        } => {
            let target = ssa_values.get(target).unwrap();
            let args_vals: Vec<_> = args
                .iter()
                .map(|arg| *ssa_values.get(arg).unwrap())
                .collect();
            let mut all_args = vec![*target];
            all_args.extend(args_vals);
            let res = emit_runtime_call(builder, 6, 2, &all_args)?;
            ssa_values.insert(result.clone(), res);
        }

        Instruction::Create {
            result,
            code,
            value,
        } => {
            let code = ssa_values.get(code).unwrap();
            let value = ssa_values.get(value).unwrap();
            let res = emit_runtime_call(builder, 8, 0, &[*code, *value])?;
            ssa_values.insert(result.clone(), res);
        }
        Instruction::Create2 {
            result,
            code,
            salt,
            value,
        } => {
            let code = ssa_values.get(code).unwrap();
            let salt = ssa_values.get(salt).unwrap();
            let value = ssa_values.get(value).unwrap();
            let res = emit_runtime_call(builder, 8, 1, &[*code, *salt, *value])?;
            ssa_values.insert(result.clone(), res);
        }

        Instruction::Selfdestruct { beneficiary } => {
            let beneficiary = ssa_values.get(beneficiary).unwrap();
            emit_runtime_call_void(builder, 9, 0, &[*beneficiary])?;
        }

        Instruction::GetContext { result, var } => {
            let res = emit_get_context(builder, *var)?;
            ssa_values.insert(result.clone(), res);
        }
        Instruction::GetBalance { result, address } => {
            let address = ssa_values.get(address).unwrap();
            let res = emit_runtime_call(builder, 10, 0, &[*address])?;
            ssa_values.insert(result.clone(), res);
        }
        Instruction::GetCode { result, address } => {
            let address = ssa_values.get(address).unwrap();
            let res = emit_runtime_call(builder, 10, 1, &[*address])?;
            ssa_values.insert(result.clone(), res);
        }
        Instruction::GetCodeSize { result, address } => {
            let address = ssa_values.get(address).unwrap();
            let res = emit_runtime_call(builder, 10, 2, &[*address])?;
            ssa_values.insert(result.clone(), res);
        }
        Instruction::GetCodeHash { result, address } => {
            let address = ssa_values.get(address).unwrap();
            let res = emit_runtime_call(builder, 10, 3, &[*address])?;
            ssa_values.insert(result.clone(), res);
        }

        Instruction::Keccak256 { result, data, len } => {
            let data = ssa_values.get(data).unwrap();
            let len = ssa_values.get(len).unwrap();
            let res = emit_runtime_call(builder, 11, 0, &[*data, *len])?;
            ssa_values.insert(result.clone(), res);
        }
        Instruction::Sha256 { result, data, len } => {
            let data = ssa_values.get(data).unwrap();
            let len = ssa_values.get(len).unwrap();
            let res = emit_runtime_call(builder, 11, 1, &[*data, *len])?;
            ssa_values.insert(result.clone(), res);
        }
        Instruction::Ripemd160 { result, data, len } => {
            let data = ssa_values.get(data).unwrap();
            let len = ssa_values.get(len).unwrap();
            let res = emit_runtime_call(builder, 11, 2, &[*data, *len])?;
            ssa_values.insert(result.clone(), res);
        }
        Instruction::EcRecover {
            result,
            hash,
            v,
            r,
            s,
        } => {
            let hash = ssa_values.get(hash).unwrap();
            let v = ssa_values.get(v).unwrap();
            let r = ssa_values.get(r).unwrap();
            let s = ssa_values.get(s).unwrap();
            let res = emit_runtime_call(builder, 11, 3, &[*hash, *v, *r, *s])?;
            ssa_values.insert(result.clone(), res);
        }

        Instruction::EmitEvent {
            event,
            topics,
            data,
        } => {
            let topics_vals: Vec<_> = topics.iter().map(|t| *ssa_values.get(t).unwrap()).collect();
            let data_vals: Vec<_> = data.iter().map(|d| *ssa_values.get(d).unwrap()).collect();
            emit_event(builder, *event, &topics_vals, &data_vals)?;
        }

        Instruction::Cast { result, value, to } => {
            let value = ssa_values.get(value).unwrap();
            let clif_type = convert_type(to)?;
            let res = builder.ins().bitcast(clif_type, MemFlags::new(), *value);
            ssa_values.insert(result.clone(), res);
        }
        Instruction::ZeroExtend { result, value, to } => {
            let value = ssa_values.get(value).unwrap();
            let clif_type = convert_type(to)?;
            let res = builder.ins().uextend(clif_type, *value);
            ssa_values.insert(result.clone(), res);
        }
        Instruction::SignExtend { result, value, to } => {
            let value = ssa_values.get(value).unwrap();
            let clif_type = convert_type(to)?;
            let res = builder.ins().sextend(clif_type, *value);
            ssa_values.insert(result.clone(), res);
        }
        Instruction::Truncate { result, value, to } => {
            let value = ssa_values.get(value).unwrap();
            let clif_type = convert_type(to)?;
            let res = builder.ins().ireduce(clif_type, *value);
            ssa_values.insert(result.clone(), res);
        }

        Instruction::Assert {
            condition,
            message: _,
        } => {
            let cond = ssa_values.get(condition).unwrap();
            let trap_block = builder.create_block();
            let continue_block = builder.create_block();

            builder
                .ins()
                .brif(*cond, continue_block, &[], trap_block, &[]);

            builder.switch_to_block(trap_block);
            builder.ins().trap(clif_ir::TrapCode::unwrap_user(1));

            builder.switch_to_block(continue_block);
        }
        Instruction::Require {
            condition,
            message: _,
        } => {
            let cond = ssa_values.get(condition).unwrap();
            let trap_block = builder.create_block();
            let continue_block = builder.create_block();

            builder
                .ins()
                .brif(*cond, continue_block, &[], trap_block, &[]);

            builder.switch_to_block(trap_block);
            builder.ins().trap(clif_ir::TrapCode::unwrap_user(2));

            builder.switch_to_block(continue_block);
        }

        Instruction::CheckedDiv {
            result,
            left,
            right,
            ty,
        } => {
            let left = ssa_values.get(left).unwrap();
            let right = ssa_values.get(right).unwrap();

            let zero = builder.ins().iconst(convert_type(ty)?, 0);
            let is_zero = builder.ins().icmp(IntCC::Equal, *right, zero);
            builder
                .ins()
                .trapnz(is_zero, clif_ir::TrapCode::unwrap_user(10));

            let res = builder.ins().udiv(*left, *right);
            ssa_values.insert(result.clone(), res);
        }

        Instruction::Jump { target: _, args: _ } => {}
        Instruction::Branch {
            condition: _,
            then_block: _,
            else_block: _,
            then_args: _,
            else_args: _,
        } => {}
        Instruction::Return { value: _ } => {}
        Instruction::Revert { message: _ } => {
            builder.ins().trap(clif_ir::TrapCode::unwrap_user(0));
        }

        Instruction::MemoryAlloc { result, size } => {
            let size = ssa_values.get(size).unwrap();
            let res = emit_runtime_call(builder, 20, 1, &[*size])?;
            ssa_values.insert(result.clone(), res);
        }
        Instruction::MemoryCopy { dest, src, size } => {
            let dest = ssa_values.get(dest).unwrap();
            let src = ssa_values.get(src).unwrap();
            let size = ssa_values.get(size).unwrap();
            emit_runtime_call(builder, 21, 3, &[*dest, *src, *size])?;
        }
        Instruction::MemorySize { result } => {
            let res = emit_runtime_call(builder, 22, 0, &[])?;
            ssa_values.insert(result.clone(), res);
        }

        Instruction::Assign { result, value } => {
            let value = ssa_values.get(value).unwrap();
            ssa_values.insert(result.clone(), *value);
        }
        Instruction::Phi { result, values } => {
            if let Some((_, first_val)) = values.first() {
                let val = ssa_values.get(first_val).unwrap();
                ssa_values.insert(result.clone(), *val);
            }
        }
    }
    Ok(())
}

pub fn lower_terminator(
    term: &Terminator,
    ssa_values: &HashMap<Value, clif_ir::Value>,
    builder: &mut FunctionBuilder,
    block_map: &std::collections::HashMap<crate::block::BlockId, clif_ir::Block>,
) -> Result<()> {
    match term {
        Terminator::Jump(block_id, ..) => {
            let block = block_map.get(block_id).unwrap();
            builder.ins().jump(*block, &[]);
        }
        Terminator::Branch {
            condition,
            then_block,
            else_block,
            ..
        } => {
            let cond = ssa_values.get(condition).unwrap();
            let then_dest = block_map.get(then_block).unwrap();
            let else_dest = block_map.get(else_block).unwrap();
            builder.ins().brif(*cond, *then_dest, &[], *else_dest, &[]);
        }
        Terminator::Return(value) => {
            let return_value = value.as_ref().and_then(|v| ssa_values.get(v));
            if let Some(val) = return_value {
                builder.ins().return_(&[*val]);
            } else {
                builder.ins().return_(&[]);
            }
        }
        Terminator::Switch {
            value,
            cases,
            default,
        } => {
            let value = ssa_values.get(value).unwrap();
            let default_block = block_map.get(default).unwrap();

            let mut next_block = None;
            for (case_val, case_block_id) in cases.iter().rev() {
                let case_block = block_map.get(case_block_id).unwrap();

                let case_value = match case_val.as_constant() {
                    Some(Constant::Uint(val, _)) => {
                        let bytes = val.to_bytes_le();
                        let mut result = 0u128;
                        for (i, &byte) in bytes.iter().enumerate().take(16) {
                            result |= (byte as u128) << (i * 8);
                        }
                        result as i64
                    }
                    Some(Constant::Int(val, _)) => {
                        use num_traits::cast::ToPrimitive;
                        val.to_i64().unwrap_or(0)
                    }
                    _ => 0,
                };

                let case_const = if case_value as i64 == case_value {
                    builder.ins().iconst(types::I64, case_value)
                } else {
                    builder.ins().iconst(types::I64, case_value)
                };
                let case_const = builder.ins().sextend(types::I128, case_const);
                let cmp = builder.ins().icmp(IntCC::Equal, *value, case_const);

                if let Some(nb) = next_block {
                    builder.ins().brif(cmp, *case_block, &[], nb, &[]);
                } else {
                    builder
                        .ins()
                        .brif(cmp, *case_block, &[], *default_block, &[]);
                }
                next_block = Some(*case_block);
            }

            if next_block.is_none() {
                builder.ins().jump(*default_block, &[]);
            }
        }
        Terminator::Revert(_msg) => {
            builder.ins().trap(clif_ir::TrapCode::unwrap_user(3));
        }
        Terminator::Panic(_msg) => {
            builder.ins().trap(clif_ir::TrapCode::unwrap_user(4));
        }
        Terminator::Invalid => {
            builder.ins().trap(clif_ir::TrapCode::unwrap_user(5));
        }
    }
    Ok(())
}

fn convert_type(ty: &Type) -> Result<types::Type> {
    match ty {
        Type::Bool => Ok(types::I8),
        Type::Uint(8) => Ok(types::I8),
        Type::Uint(16) => Ok(types::I16),
        Type::Uint(32) => Ok(types::I32),
        Type::Uint(64) => Ok(types::I64),
        Type::Uint(128) => Ok(types::I128),
        Type::Uint(256) => Ok(types::I128),
        Type::Int(8) => Ok(types::I8),
        Type::Int(16) => Ok(types::I16),
        Type::Int(32) => Ok(types::I32),
        Type::Int(64) => Ok(types::I64),
        Type::Int(128) => Ok(types::I128),
        Type::Int(256) => Ok(types::I128),
        Type::Address => Ok(types::I128),
        Type::Bytes4 => Ok(types::I32),
        Type::Bytes20 => Ok(types::I128),
        Type::Bytes32 => Ok(types::I128),
        Type::Bytes(n) if *n <= 32 => Ok(types::I128),
        _ => Err(IrError::TypeError(format!(
            "Cannot convert type {:?} to Cranelift",
            ty
        ))),
    }
}

fn get_location_address(
    location: &crate::values::Location,
    ssa_values: &HashMap<Value, clif_ir::Value>,
    builder: &mut FunctionBuilder,
) -> Result<clif_ir::Value> {
    use crate::values::Location;
    match location {
        Location::Memory { base, offset } => {
            let base_val = ssa_values
                .get(base)
                .ok_or_else(|| IrError::InvalidInstruction("Base value not found".into()))?;
            let offset_val = ssa_values
                .get(offset)
                .ok_or_else(|| IrError::InvalidInstruction("Offset value not found".into()))?;
            Ok(builder.ins().iadd(*base_val, *offset_val))
        }
        Location::Storage { .. }
        | Location::Stack { .. }
        | Location::Calldata { .. }
        | Location::ReturnData { .. } => Err(IrError::InvalidInstruction(
            "Unsupported location type in memory operation".into(),
        )),
    }
}

fn get_storage_key_value(
    key: &StorageKey,
    ssa_values: &HashMap<Value, clif_ir::Value>,
    builder: &mut FunctionBuilder,
) -> Result<clif_ir::Value> {
    match key {
        StorageKey::Slot(slot) => {
            let bytes = slot.to_bytes_le();
            let mut result = 0u128;
            for (i, &byte) in bytes.iter().enumerate().take(16) {
                result |= (byte as u128) << (i * 8);
            }
            let slot_val = result as i64;
            let val = builder.ins().iconst(types::I64, slot_val);
            Ok(builder.ins().uextend(types::I128, val))
        }
        StorageKey::Dynamic(val) => ssa_values
            .get(val)
            .copied()
            .ok_or_else(|| IrError::InvalidInstruction("Dynamic key value not found".into())),
        StorageKey::Computed(val) => ssa_values
            .get(val)
            .copied()
            .ok_or_else(|| IrError::InvalidInstruction("Computed key value not found".into())),
        StorageKey::MappingKey { base, key } => {
            let bytes = base.to_bytes_le();
            let mut result = 0u128;
            for (i, &byte) in bytes.iter().enumerate().take(16) {
                result |= (byte as u128) << (i * 8);
            }
            let base_val = result as i64;
            let base_const_64 = builder.ins().iconst(types::I64, base_val);
            let base_const = builder.ins().uextend(types::I128, base_const_64);
            let key_val = ssa_values
                .get(key)
                .ok_or_else(|| IrError::InvalidInstruction("Mapping key not found".into()))?;

            emit_runtime_call(builder, 11, 0, &[*key_val, base_const])
        }
        StorageKey::ArrayElement { base, index } => {
            let bytes = base.to_bytes_le();
            let mut result = 0u128;
            for (i, &byte) in bytes.iter().enumerate().take(16) {
                result |= (byte as u128) << (i * 8);
            }
            let base_val = result as i64;
            let base_const_64 = builder.ins().iconst(types::I64, base_val);
            let base_const = builder.ins().uextend(types::I128, base_const_64);
            let index_val = ssa_values
                .get(index)
                .ok_or_else(|| IrError::InvalidInstruction("Array index not found".into()))?;
            Ok(builder.ins().iadd(base_const, *index_val))
        }
    }
}

fn emit_runtime_call(
    builder: &mut FunctionBuilder,
    namespace: u32,
    index: u32,
    args: &[clif_ir::Value],
) -> Result<clif_ir::Value> {
    let sig = builder
        .func
        .import_signature(cranelift_codegen::ir::Signature {
            params: args
                .iter()
                .map(|_| cranelift_codegen::ir::AbiParam::new(types::I128))
                .collect(),
            returns: vec![cranelift_codegen::ir::AbiParam::new(types::I128)],
            call_conv: cranelift_codegen::isa::CallConv::SystemV,
        });

    let user_ref = builder
        .func
        .declare_imported_user_function(clif_ir::UserExternalName { namespace, index });

    let func_ref = builder
        .func
        .import_function(cranelift_codegen::ir::ExtFuncData {
            name: cranelift_codegen::ir::ExternalName::user(user_ref),
            signature: sig,
            colocated: false,
        });

    let call = builder.ins().call(func_ref, args);
    Ok(builder.inst_results(call)[0])
}

fn emit_runtime_call_void(
    builder: &mut FunctionBuilder,
    namespace: u32,
    index: u32,
    args: &[clif_ir::Value],
) -> Result<()> {
    let sig = builder
        .func
        .import_signature(cranelift_codegen::ir::Signature {
            params: args
                .iter()
                .map(|_| cranelift_codegen::ir::AbiParam::new(types::I128))
                .collect(),
            returns: vec![],
            call_conv: cranelift_codegen::isa::CallConv::SystemV,
        });

    let user_ref = builder
        .func
        .declare_imported_user_function(clif_ir::UserExternalName { namespace, index });

    let func_ref = builder
        .func
        .import_function(cranelift_codegen::ir::ExtFuncData {
            name: cranelift_codegen::ir::ExternalName::user(user_ref),
            signature: sig,
            colocated: false,
        });

    builder.ins().call(func_ref, args);
    Ok(())
}

fn emit_call(
    builder: &mut FunctionBuilder,
    target: &CallTarget,
    args: &[clif_ir::Value],
    value: Option<clif_ir::Value>,
) -> Result<clif_ir::Value> {
    match target {
        CallTarget::Internal(name) => {
            let sig = builder
                .func
                .import_signature(cranelift_codegen::ir::Signature {
                    params: args
                        .iter()
                        .map(|_| cranelift_codegen::ir::AbiParam::new(types::I128))
                        .collect(),
                    returns: vec![cranelift_codegen::ir::AbiParam::new(types::I128)],
                    call_conv: cranelift_codegen::isa::CallConv::SystemV,
                });

            let user_ref = builder
                .func
                .declare_imported_user_function(clif_ir::UserExternalName {
                    namespace: 4,
                    index: name.len() as u32,
                });

            let func_ref = builder
                .func
                .import_function(cranelift_codegen::ir::ExtFuncData {
                    name: cranelift_codegen::ir::ExternalName::user(user_ref),
                    signature: sig,
                    colocated: true,
                });

            let call = builder.ins().call(func_ref, args);
            Ok(builder.inst_results(call)[0])
        }
        CallTarget::External(addr) => {
            let addr_val = match addr {
                Value::Constant(c) => match c {
                    Constant::Address(bytes) => {
                        let mut val = 0u128;
                        for (i, &byte) in bytes.iter().enumerate().take(16) {
                            val |= (byte as u128) << (i * 8);
                        }
                        let addr_64 = builder.ins().iconst(types::I64, val as i64);
                        builder.ins().uextend(types::I128, addr_64)
                    }
                    _ => {
                        let zero = builder.ins().iconst(types::I64, 0);
                        builder.ins().uextend(types::I128, zero)
                    }
                },
                _ => {
                    let zero = builder.ins().iconst(types::I64, 0);
                    builder.ins().uextend(types::I128, zero)
                }
            };
            let mut all_args = vec![addr_val];
            if let Some(val) = value {
                all_args.push(val);
            }
            all_args.extend_from_slice(args);
            emit_runtime_call(builder, 6, 0, &all_args)
        }
        CallTarget::Library(name) => emit_runtime_call(builder, 5, name.len() as u32, args),
        CallTarget::Builtin(_) => emit_runtime_call(builder, 7, 0, args),
    }
}

fn emit_get_context(builder: &mut FunctionBuilder, var: ContextVariable) -> Result<clif_ir::Value> {
    let ctx_offset = match var {
        ContextVariable::MsgSender => 0,
        ContextVariable::MsgValue => 20,
        ContextVariable::MsgData => 52,
        ContextVariable::MsgSig => 84,
        ContextVariable::BlockNumber => 116,
        ContextVariable::BlockTimestamp => 148,
        ContextVariable::BlockDifficulty => 180,
        ContextVariable::BlockGasLimit => 212,
        ContextVariable::BlockCoinbase => 244,
        ContextVariable::ChainId => 264,
        ContextVariable::BlockBaseFee => 296,
        ContextVariable::TxOrigin => 328,
        ContextVariable::TxGasPrice => 348,
        ContextVariable::GasLeft => 380,
        ContextVariable::ThisAddress => 412,
        ContextVariable::ThisBalance => 432,
    };

    let ctx_ptr = builder.block_params(builder.current_block().unwrap())[0];
    Ok(builder.ins().load(
        types::I128,
        MemFlags::trusted(),
        ctx_ptr,
        Offset32::new(ctx_offset),
    ))
}

fn emit_event(
    builder: &mut FunctionBuilder,
    event_id: crate::contract::EventId,
    topics: &[clif_ir::Value],
    data: &[clif_ir::Value],
) -> Result<()> {
    let event_val = builder.ins().iconst(types::I32, event_id.0 as i64);
    let topic_count = builder.ins().iconst(types::I32, topics.len() as i64);

    // Not yet implemented: event data packing requires ABI encoding helpers
    let data_ptr = builder.ins().iconst(types::I64, 0);
    let data_len = builder.ins().iconst(types::I64, data.len() as i64);

    let mut args = vec![event_val, topic_count];
    args.extend_from_slice(&topics[..topics.len().min(4)]);
    while args.len() < 6 {
        let zero = builder.ins().iconst(types::I64, 0);
        args.push(builder.ins().uextend(types::I128, zero));
    }
    args.push(data_ptr);
    args.push(data_len);

    let args_i128: Vec<_> = args
        .iter()
        .map(|&v| {
            if builder.func.dfg.value_type(v) != types::I128 {
                builder.ins().uextend(types::I128, v)
            } else {
                v
            }
        })
        .collect();

    emit_runtime_call_void(builder, 12, 0, &args_i128)?;
    Ok(())
}
