use super::context::TypeContext;
use super::errors::TransformError;
use thalir_core::types::Type;
use tree_sitter::Node;

pub struct TypeResolver;

impl TypeResolver {
    pub fn resolve_type(node: Node, ctx: &dyn TypeContext) -> Result<Type, TransformError> {
        match node.kind() {
            "type_name" => Self::resolve_type_name(node, ctx),
            "elementary_type" => Self::resolve_elementary_type(node, ctx),
            "primitive_type" => Self::resolve_primitive_type(node, ctx),
            "mapping_type" | "mapping" => Self::resolve_mapping_type(node, ctx),
            "array_type" => Self::resolve_array_type(node, ctx),
            "user_defined_type" => Self::resolve_user_defined_type(node, ctx),
            "function" | "function_type" => Ok(Type::Uint(256)),
            _ => {
                if let Some(type_node) = node.child_by_field_name("type") {
                    Self::resolve_type(type_node, ctx)
                } else {
                    Err(TransformError::TypeError(format!(
                        "Unknown type node: {}",
                        node.kind()
                    )))
                }
            }
        }
    }

    fn resolve_type_name(node: Node, ctx: &dyn TypeContext) -> Result<Type, TransformError> {
        let text = ctx.get_node_text(node);

        if text.ends_with("[]") {
            let base_text = &text[..text.len() - 2];

            if let Some(child) = node.child(0) {
                let element_type = Self::resolve_type(child, ctx)?;
                return Ok(Type::Array(Box::new(element_type), None));
            } else {
                let element_type = Self::resolve_type_string(base_text)?;
                return Ok(Type::Array(Box::new(element_type), None));
            }
        }

        if let Some(child) = node.child(0) {
            Self::resolve_type(child, ctx)
        } else {
            Self::resolve_type_string(text)
        }
    }

    fn resolve_elementary_type(node: Node, ctx: &dyn TypeContext) -> Result<Type, TransformError> {
        let type_text = ctx.get_node_text(node);
        Self::resolve_type_string(type_text)
    }

    fn resolve_primitive_type(node: Node, ctx: &dyn TypeContext) -> Result<Type, TransformError> {
        let type_text = ctx.get_node_text(node);
        Self::resolve_type_string(type_text)
    }

    fn resolve_type_string(type_str: &str) -> Result<Type, TransformError> {
        if type_str == "uint" {
            return Ok(Type::Uint(256));
        } else if type_str.starts_with("uint") && type_str != "uint256" {
            if let Some(bits_str) = type_str.strip_prefix("uint") {
                if let Ok(bits) = bits_str.parse::<u16>() {
                    if bits > 0 && bits <= 256 && bits % 8 == 0 {
                        return Ok(Type::Uint(bits));
                    } else {
                        return Err(TransformError::TypeError(format!(
                            "Invalid uint size: {}",
                            bits
                        )));
                    }
                }
            }
        }

        if type_str == "int" {
            return Ok(Type::Int(256));
        } else if type_str.starts_with("int") && type_str != "int256" {
            if let Some(bits_str) = type_str.strip_prefix("int") {
                if let Ok(bits) = bits_str.parse::<u16>() {
                    if bits > 0 && bits <= 256 && bits % 8 == 0 {
                        return Ok(Type::Int(bits));
                    } else {
                        return Err(TransformError::TypeError(format!(
                            "Invalid int size: {}",
                            bits
                        )));
                    }
                }
            }
        }

        match type_str {
            "uint256" => Ok(Type::Uint(256)),
            "int256" => Ok(Type::Int(256)),

            "bool" => Ok(Type::Bool),
            "address" => Ok(Type::Address),
            "address payable" => Ok(Type::Address),
            "bytes32" => Ok(Type::Bytes32),
            "bytes" => Ok(Type::String),
            "string" => Ok(Type::String),

            s if s.starts_with("bytes") && s.len() > 5 => {
                let size_str = &s[5..];
                let size = size_str.parse::<u8>().map_err(|_| {
                    TransformError::TypeError(format!("Invalid bytes size: {}", size_str))
                })?;
                if size >= 1 && size <= 32 {
                    Ok(Type::Bytes(size))
                } else {
                    Err(TransformError::TypeError(format!(
                        "Bytes size must be between 1 and 32, got {}",
                        size
                    )))
                }
            }

            _ => Err(TransformError::TypeError(format!(
                "Unknown type: {}",
                type_str
            ))),
        }
    }

    fn resolve_mapping_type(node: Node, ctx: &dyn TypeContext) -> Result<Type, TransformError> {
        let key_type = node
            .child_by_field_name("key")
            .or_else(|| node.child_by_field_name("key_type"));
        let value_type = node
            .child_by_field_name("value")
            .or_else(|| node.child_by_field_name("value_type"));

        let (key_node, value_node) = if key_type.is_some() && value_type.is_some() {
            (key_type.unwrap(), value_type.unwrap())
        } else {
            let mut types = Vec::new();
            let mut cursor = node.walk();

            if let Some(parent) = node.parent() {
                let mut parent_cursor = parent.walk();
                let mut found_mapping = false;
                for sibling in parent.children(&mut parent_cursor) {
                    if sibling == node {
                        found_mapping = true;
                    } else if found_mapping {
                        if sibling.kind() == "primitive_type"
                            || sibling.kind() == "user_defined_type"
                        {
                            types.push(sibling);
                        } else if sibling.kind() == "type_name" {
                            types.push(sibling);
                        }
                    }
                }
            }

            if types.is_empty() {
                for child in node.children(&mut cursor) {
                    if child.kind() == "primitive_type"
                        || child.kind() == "user_defined_type"
                        || child.kind() == "elementary_type"
                        || child.kind() == "type_name"
                    {
                        types.push(child);
                    }
                }
            }

            if types.len() >= 2 {
                (types[0], types[1])
            } else {
                return Err(TransformError::MissingField {
                    field: "key and value types".to_string(),
                    node_type: format!("mapping (found {} type nodes)", types.len()),
                });
            }
        };

        let key = Self::resolve_type(key_node, ctx)?;
        let value = Self::resolve_type(value_node, ctx)?;

        Ok(Type::Mapping(Box::new(key), Box::new(value)))
    }

    fn resolve_array_type(node: Node, ctx: &dyn TypeContext) -> Result<Type, TransformError> {
        let element_type = node
            .child_by_field_name("element")
            .or_else(|| node.child(0))
            .ok_or_else(|| TransformError::MissingField {
                field: "element".to_string(),
                node_type: "array_type".to_string(),
            })?;

        let element = Self::resolve_type(element_type, ctx)?;

        if let Some(size_node) = node.child_by_field_name("size") {
            let size_text = ctx.get_node_text(size_node);
            if let Ok(size) = size_text.parse::<u32>() {
                Ok(Type::Array(Box::new(element), Some(size as usize)))
            } else {
                Ok(Type::Array(Box::new(element), None))
            }
        } else {
            Ok(Type::Array(Box::new(element), None))
        }
    }

    fn resolve_user_defined_type(
        node: Node,
        ctx: &dyn TypeContext,
    ) -> Result<Type, TransformError> {
        let type_name = ctx.get_node_text(node);

        Ok(Type::String)
    }

    pub fn infer_expression_type(
        node: Node,
        ctx: &dyn TypeContext,
    ) -> Result<Type, TransformError> {
        match node.kind() {
            "number_literal" => {
                let text = ctx.get_node_text(node);

                Ok(Type::Uint(256))
            }
            "string_literal" => Ok(Type::String),
            "boolean_literal" => Ok(Type::Bool),
            "identifier" => {
                let name = ctx.get_node_text(node);
                ctx.lookup_symbol(name)
                    .map(|sym| sym.ty.clone())
                    .ok_or_else(|| TransformError::SymbolNotFound(name.to_string()))
            }
            "binary_expression" => {
                let operator = node
                    .child_by_field_name("operator")
                    .map(|n| ctx.get_node_text(n))
                    .unwrap_or("");

                match operator {
                    "==" | "!=" | "<" | ">" | "<=" | ">=" | "&&" | "||" => Ok(Type::Bool),
                    _ => {
                        if let Some(left) = node.child_by_field_name("left") {
                            Self::infer_expression_type(left, ctx)
                        } else {
                            Ok(Type::Uint(256))
                        }
                    }
                }
            }
            "unary_expression" => {
                let operator = node
                    .child_by_field_name("operator")
                    .map(|n| ctx.get_node_text(n))
                    .unwrap_or("");

                match operator {
                    "!" => Ok(Type::Bool),
                    _ => {
                        if let Some(operand) = node.child_by_field_name("operand") {
                            Self::infer_expression_type(operand, ctx)
                        } else {
                            Ok(Type::Uint(256))
                        }
                    }
                }
            }
            _ => Ok(Type::Uint(256)),
        }
    }
}
