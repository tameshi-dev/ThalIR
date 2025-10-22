use super::errors::TransformError;
use super::{context::TransformationContext, type_resolver::TypeResolver};
use anyhow::Result;
use num_bigint::BigUint;
use thalir_core::{
    builder::{BlockBuilder, InstBuilderExt},
    types::Type,
    values::Value,
};
use tree_sitter::Node;

pub struct ExpressionTransformer;

impl ExpressionTransformer {
    pub fn new() -> Self {
        Self
    }

    pub fn transform_expression(
        &mut self,
        node: Node,
        ctx: &mut TransformationContext,
        block: &mut BlockBuilder,
    ) -> Result<Value> {
        match node.kind() {
            "identifier" => self.transform_identifier(node, ctx, block),
            "number_literal" => self.transform_number_literal(node, ctx, block),
            "string_literal" => self.transform_string_literal(node, ctx, block),
            "boolean_literal" => self.transform_boolean_literal(node, ctx, block),
            "binary_expression" => self.transform_binary_expression(node, ctx, block),
            "unary_expression" => self.transform_unary_expression(node, ctx, block),
            "assignment_expression" => self.transform_assignment(node, ctx, block),
            "call_expression" => self.transform_call_expression(node, ctx, block),
            "member_expression" => self.transform_member_expression(node, ctx, block),
            "index_expression" => self.transform_index_expression(node, ctx, block),
            "parenthesized_expression" => {
                if let Some(inner) = node.child(1) {
                    self.transform_expression(inner, ctx, block)
                } else {
                    Err(anyhow::anyhow!("Empty parenthesized expression"))
                }
            }
            _ => {
                ctx.add_error(TransformError::UnsupportedFeature(format!(
                    "Expression type: {}",
                    node.kind()
                )));

                Ok(block.constant_uint(0, 256))
            }
        }
    }

    fn transform_identifier(
        &mut self,
        node: Node,
        ctx: &mut TransformationContext,
        block: &mut BlockBuilder,
    ) -> Result<Value> {
        let name = ctx.get_node_text(node);

        if let Some(symbol) = ctx.lookup_symbol(name) {
            if symbol.is_state_var {
                let slot = symbol.slot.unwrap_or(0);
                Ok(block.storage_load(BigUint::from(slot)))
            } else {
                Ok(symbol.value.clone())
            }
        } else {
            match name {
                "msg" => Ok(block.new_temp()),
                "block" => Ok(block.new_temp()),
                "tx" => Ok(block.new_temp()),
                _ => {
                    ctx.add_error(TransformError::SymbolNotFound(name.to_string()));
                    Ok(block.constant_uint(0, 256))
                }
            }
        }
    }

    fn transform_number_literal(
        &mut self,
        node: Node,
        ctx: &TransformationContext,
        block: &mut BlockBuilder,
    ) -> Result<Value> {
        let text = ctx.get_node_text(node);

        let clean_text = text.replace('_', "");

        if clean_text.starts_with("0x") || clean_text.starts_with("0X") {
            let hex_str = &clean_text[2..];
            let value = u64::from_str_radix(hex_str, 16).unwrap_or(0);
            Ok(block.constant_uint(value, 256))
        } else {
            let value = clean_text.parse::<u64>().unwrap_or(0);
            Ok(block.constant_uint(value, 256))
        }
    }

    fn transform_string_literal(
        &mut self,
        node: Node,
        ctx: &TransformationContext,
        block: &mut BlockBuilder,
    ) -> Result<Value> {
        let text = ctx.get_node_text(node);

        let content = if text.starts_with('"') && text.ends_with('"') {
            &text[1..text.len() - 1]
        } else if text.starts_with('\'') && text.ends_with('\'') {
            &text[1..text.len() - 1]
        } else {
            text
        };

        let hash = content
            .bytes()
            .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
        Ok(block.constant_uint(hash, 256))
    }

    fn transform_boolean_literal(
        &mut self,
        node: Node,
        ctx: &TransformationContext,
        block: &mut BlockBuilder,
    ) -> Result<Value> {
        let text = ctx.get_node_text(node);
        let value = text == "true";
        Ok(block.constant_bool(value))
    }

    fn transform_binary_expression(
        &mut self,
        node: Node,
        ctx: &mut TransformationContext,
        block: &mut BlockBuilder,
    ) -> Result<Value> {
        let left_node = node
            .child_by_field_name("left")
            .ok_or_else(|| anyhow::anyhow!("Missing left operand"))?;
        let right_node = node
            .child_by_field_name("right")
            .ok_or_else(|| anyhow::anyhow!("Missing right operand"))?;
        let op_node = node
            .child_by_field_name("operator")
            .ok_or_else(|| anyhow::anyhow!("Missing operator"))?;

        let left = self.transform_expression(left_node, ctx, block)?;
        let right = self.transform_expression(right_node, ctx, block)?;
        let operator = ctx.get_node_text(op_node);

        let ty = TypeResolver::infer_expression_type(left_node, ctx)?;

        match operator {
            "+" => Ok(block.add(left, right, ty)),
            "-" => Ok(block.sub(left, right, ty)),
            "*" => Ok(block.mul(left, right, ty)),
            "/" => Ok(block.div(left, right, ty)),
            "%" => Ok(block.mod_(left, right, ty)),

            "==" => Ok(block.eq(left, right)),
            "!=" => Ok(block.ne(left, right)),
            "<" => Ok(block.lt(left, right)),
            ">" => Ok(block.gt(left, right)),
            "<=" => Ok(block.le(left, right)),
            ">=" => Ok(block.ge(left, right)),

            "&&" => Ok(block.and(left, right)),
            "||" => Ok(block.or(left, right)),

            "&" => Ok(block.and(left, right)),
            "|" => Ok(block.or(left, right)),
            "^" => {
                let not_left = block.not(left.clone());
                let not_right = block.not(right.clone());
                let left_and_not_right = block.and(left, not_right);
                let not_left_and_right = block.and(not_left, right);
                Ok(block.or(left_and_not_right, not_left_and_right))
            }
            "<<" => Ok(block.shl(left, right)),
            ">>" => Ok(block.shr(left, right)),

            _ => {
                ctx.add_error(TransformError::UnsupportedFeature(format!(
                    "Binary operator: {}",
                    operator
                )));
                Ok(left)
            }
        }
    }

    fn transform_unary_expression(
        &mut self,
        node: Node,
        ctx: &mut TransformationContext,
        block: &mut BlockBuilder,
    ) -> Result<Value> {
        let operand_node = node
            .child_by_field_name("operand")
            .or_else(|| node.child(1))
            .ok_or_else(|| anyhow::anyhow!("Missing operand"))?;
        let op_node = node
            .child_by_field_name("operator")
            .or_else(|| node.child(0))
            .ok_or_else(|| anyhow::anyhow!("Missing operator"))?;

        let operator = ctx.get_node_text(op_node);

        match operator {
            "!" => {
                let operand = self.transform_expression(operand_node, ctx, block)?;
                Ok(block.not(operand))
            }
            "-" => {
                let operand = self.transform_expression(operand_node, ctx, block)?;
                let ty = TypeResolver::infer_expression_type(operand_node, ctx)?;
                let zero = block.constant_uint(0, 256);
                Ok(block.sub(zero, operand, ty))
            }
            "++" | "--" => self.transform_update_expression(node, ctx, block),
            _ => {
                ctx.add_error(TransformError::UnsupportedFeature(format!(
                    "Unary operator: {}",
                    operator
                )));
                let operand = self.transform_expression(operand_node, ctx, block)?;
                Ok(operand)
            }
        }
    }

    fn transform_update_expression(
        &mut self,
        node: Node,
        ctx: &mut TransformationContext,
        block: &mut BlockBuilder,
    ) -> Result<Value> {
        let operand_node = node
            .child_by_field_name("operand")
            .or_else(|| {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "identifier" {
                        return Some(child);
                    }
                }
                None
            })
            .ok_or_else(|| anyhow::anyhow!("Missing operand for update expression"))?;

        let name = ctx.get_node_text(operand_node);
        let symbol = ctx
            .lookup_symbol(name)
            .ok_or_else(|| anyhow::anyhow!("Unknown variable: {}", name))?;

        let current_value = if symbol.is_state_var {
            let slot = symbol.slot.unwrap_or(0);
            block.storage_load(BigUint::from(slot))
        } else {
            symbol.value.clone()
        };

        let one = block.constant_uint(1, 256);
        let ty = symbol.ty.clone();

        let is_increment = ctx.source[node.byte_range()].contains("++");

        let new_value = if is_increment {
            block.add(current_value.clone(), one, ty.clone())
        } else {
            block.sub(current_value.clone(), one, ty.clone())
        };

        if symbol.is_state_var {
            let slot = symbol.slot.unwrap_or(0);
            block.storage_store(BigUint::from(slot), new_value.clone());
        } else {
            if let Some(mut_symbol) = ctx.scope_stack.last_mut().and_then(|s| s.lookup_mut(name)) {
                mut_symbol.value = new_value.clone();
            }
        }

        Ok(new_value)
    }

    fn transform_assignment(
        &mut self,
        node: Node,
        ctx: &mut TransformationContext,
        block: &mut BlockBuilder,
    ) -> Result<Value> {
        let left_node = node
            .child_by_field_name("left")
            .ok_or_else(|| anyhow::anyhow!("Missing assignment target"))?;
        let right_node = node
            .child_by_field_name("right")
            .ok_or_else(|| anyhow::anyhow!("Missing assignment value"))?;

        let value = self.transform_expression(right_node, ctx, block)?;

        match left_node.kind() {
            "identifier" => {
                let name = ctx.get_node_text(left_node);

                if let Some(symbol) = ctx.lookup_symbol(name) {
                    if symbol.is_state_var {
                        let slot = symbol.slot.unwrap_or(0);
                        block.storage_store(BigUint::from(slot), value.clone());
                    } else {
                        if let Some(mut_symbol) =
                            ctx.scope_stack.last_mut().and_then(|s| s.lookup_mut(name))
                        {
                            mut_symbol.value = value.clone();
                        }
                    }
                } else {
                    ctx.add_error(TransformError::SymbolNotFound(name.to_string()));
                }
            }
            "member_expression" => {
                // Not yet implemented: struct member assignments require type system integration
            }
            "index_expression" => {
                /*
                Array and mapping indexed assignments need storage slot computation.
                Currently handled via higher-level array_store/mapping_store operations
                in the structural transformer.
                */
            }
            _ => {
                ctx.add_error(TransformError::UnsupportedFeature(format!(
                    "Assignment target: {}",
                    left_node.kind()
                )));
            }
        }

        Ok(value)
    }

    fn transform_call_expression(
        &mut self,
        node: Node,
        ctx: &mut TransformationContext,
        block: &mut BlockBuilder,
    ) -> Result<Value> {
        let function_node = node
            .child_by_field_name("function")
            .ok_or_else(|| anyhow::anyhow!("Missing function in call"))?;

        let mut args = Vec::new();
        if let Some(args_node) = node.child_by_field_name("arguments") {
            let mut cursor = args_node.walk();
            for child in args_node.children(&mut cursor) {
                if child.kind() != "," && child.kind() != "(" && child.kind() != ")" {
                    args.push(self.transform_expression(child, ctx, block)?);
                }
            }
        }

        match function_node.kind() {
            "identifier" => {
                let name = ctx.get_node_text(function_node);

                match name {
                    "require" => {
                        if !args.is_empty() {
                            let condition = args[0].clone();
                            /*
                            String literal extraction from AST nodes requires traversing
                            the tree-sitter node to find string_literal children and
                            extracting their text content. Using generic message until
                            string extraction helper is implemented.
                            */
                            let message = "Requirement failed";
                            block.require(condition, message);
                        }
                        Ok(block.new_temp())
                    }
                    "assert" => {
                        if !args.is_empty() {
                            let condition = args[0].clone();
                            block.assert(condition, "Assertion failed");
                        }
                        Ok(block.new_temp())
                    }
                    "revert" => {
                        // Not yet implemented: revert should emit proper terminator instruction
                        let _message = "Revert";
                        Ok(block.new_temp())
                    }
                    _ => {
                        let mangled_name = self.resolve_mangled_function_name(name, &args, block);
                        Ok(block.call_internal(&mangled_name, args))
                    }
                }
            }
            "member_expression" => {
                // Not yet implemented: external/library calls require call encoding infrastructure
                Ok(block.new_temp())
            }
            _ => {
                ctx.add_error(TransformError::UnsupportedFeature(format!(
                    "Call type: {}",
                    function_node.kind()
                )));
                Ok(block.new_temp())
            }
        }
    }

    fn transform_member_expression(
        &mut self,
        node: Node,
        ctx: &mut TransformationContext,
        block: &mut BlockBuilder,
    ) -> Result<Value> {
        let object_node = node
            .child_by_field_name("object")
            .ok_or_else(|| anyhow::anyhow!("Missing object in member expression"))?;
        let property_node = node
            .child_by_field_name("property")
            .ok_or_else(|| anyhow::anyhow!("Missing property in member expression"))?;

        let object_name = ctx.get_node_text(object_node);
        let property_name = ctx.get_node_text(property_node);

        match object_name {
            "msg" => match property_name {
                "sender" => Ok(block.msg_sender()),
                "value" => Ok(block.msg_value()),
                "data" => Ok(block.msg_data()),
                _ => Ok(block.new_temp()),
            },
            "block" => match property_name {
                "timestamp" => Ok(block.block_timestamp()),
                "number" => Ok(block.block_number()),
                "coinbase" => Ok(block.block_coinbase()),
                _ => Ok(block.new_temp()),
            },
            "tx" => match property_name {
                "origin" => Ok(block.tx_origin()),
                "gasprice" => Ok(block.tx_gasprice()),
                _ => Ok(block.new_temp()),
            },
            _ => {
                /*
                Struct member access requires type information to compute field offsets.
                The type resolver must provide struct definitions with field layouts
                before we can generate proper load instructions with computed offsets.
                */
                let _object = self.transform_expression(object_node, ctx, block)?;
                Ok(block.new_temp())
            }
        }
    }

    fn transform_index_expression(
        &mut self,
        node: Node,
        ctx: &mut TransformationContext,
        block: &mut BlockBuilder,
    ) -> Result<Value> {
        let object_node = node
            .child_by_field_name("object")
            .ok_or_else(|| anyhow::anyhow!("Missing object in index expression"))?;
        let index_node = node
            .child_by_field_name("index")
            .ok_or_else(|| anyhow::anyhow!("Missing index in index expression"))?;

        let object = self.transform_expression(object_node, ctx, block)?;
        let index = self.transform_expression(index_node, ctx, block)?;

        if let Ok(ty) = TypeResolver::infer_expression_type(object_node, ctx) {
            match ty {
                Type::Array(_, _) => {
                    // Not yet implemented: array indexing requires layout computation
                    Ok(block.new_temp())
                }
                Type::Mapping(_, _) => Ok(block.mapping_load(object, index)),
                _ => {
                    ctx.add_error(TransformError::TypeError(format!(
                        "Cannot index type: {:?}",
                        ty
                    )));
                    Ok(block.new_temp())
                }
            }
        } else {
            Ok(block.new_temp())
        }
    }

    fn resolve_mangled_function_name(
        &self,
        base_name: &str,
        _args: &[Value],
        _block: &BlockBuilder,
    ) -> String {
        base_name.to_string()
    }

    fn type_to_mangle_string(ty: &Type) -> String {
        match ty {
            Type::Uint(bits) => format!("i{}", bits),
            Type::Int(bits) => format!("s{}", bits),
            Type::Address => "i160".to_string(),
            Type::Bool => "i1".to_string(),
            Type::Bytes(size) => format!("bytes{}", size),
            Type::String => "string".to_string(),
            Type::Bytes4 => "bytes4".to_string(),
            Type::Bytes20 => "bytes20".to_string(),
            Type::Bytes32 => "bytes32".to_string(),
            Type::Array(elem, None) => format!("arr_{}", Self::type_to_mangle_string(elem)),
            Type::Array(elem, Some(len)) => {
                format!("arr{}_{}", len, Self::type_to_mangle_string(elem))
            }
            Type::Mapping(key, value) => format!(
                "map_{}_{}",
                Self::type_to_mangle_string(key),
                Self::type_to_mangle_string(value)
            ),
            Type::Struct(id) => format!("struct_{:?}", id),
            Type::Enum(id) => format!("enum_{:?}", id),
            Type::Function(_) => "function".to_string(),
            Type::Contract(_) => "contract".to_string(),
            _ => "unknown".to_string(),
        }
    }
}
