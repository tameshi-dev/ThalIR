use std::collections::HashMap;
use thalir_core::{
    block::Terminator,
    function::{Mutability, Visibility},
    instructions::{CallTarget, StorageKey},
    types::Type,
    values::{Constant, Value, ValueId},
};

pub struct SSAContext {
    pub next_value: u32,
    pub value_map: HashMap<String, u32>,
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
        let key = format!("{:?}", value);
        if let Some(&v) = self.value_map.get(&key) {
            v
        } else {
            let v = self.next_value;
            self.value_map.insert(key, v);
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

pub struct IRFormatterBase;

impl IRFormatterBase {
    pub fn format_type(ty: &Type) -> String {
        match ty {
            Type::Bool => "i1".to_string(),
            Type::Uint(8) => "i8".to_string(),
            Type::Uint(16) => "i16".to_string(),
            Type::Uint(32) => "i32".to_string(),
            Type::Uint(64) => "i64".to_string(),
            Type::Uint(128) => "i128".to_string(),
            Type::Uint(256) => "i256".to_string(),
            Type::Uint(bits) => format!("i{}", bits),
            Type::Int(8) => "i8".to_string(),
            Type::Int(16) => "i16".to_string(),
            Type::Int(32) => "i32".to_string(),
            Type::Int(64) => "i64".to_string(),
            Type::Int(128) => "i128".to_string(),
            Type::Int(256) => "i256".to_string(),
            Type::Int(bits) => format!("i{}", bits),
            Type::Address => "address".to_string(),
            Type::Bytes(n) => format!("bytes{}", n),
            Type::Bytes4 => "bytes4".to_string(),
            Type::Bytes20 => "bytes20".to_string(),
            Type::Bytes32 => "bytes32".to_string(),
            Type::String => "string".to_string(),
            Type::Array(inner, size) => {
                if let Some(sz) = size {
                    format!("[{} x {}]", Self::format_type(inner), sz)
                } else {
                    format!("[{}]", Self::format_type(inner))
                }
            }
            Type::Mapping(key, value) => {
                format!(
                    "mapping({} => {})",
                    Self::format_type(key),
                    Self::format_type(value)
                )
            }
            _ => format!("{:?}", ty),
        }
    }

    pub fn type_suffix(ty: &Type) -> String {
        match ty {
            Type::Uint(256) | Type::Int(256) => "i256".to_string(),
            Type::Uint(128) | Type::Int(128) => "i128".to_string(),
            Type::Uint(64) | Type::Int(64) => "i64".to_string(),
            Type::Uint(32) | Type::Int(32) => "i32".to_string(),
            Type::Uint(16) | Type::Int(16) => "i16".to_string(),
            Type::Uint(8) | Type::Int(8) => "i8".to_string(),
            Type::Bool => "i1".to_string(),
            _ => Self::format_type(ty),
        }
    }

    pub fn format_value(value: &Value, _ssa: &mut SSAContext, param_vnums: &[u32]) -> String {
        match value {
            Value::Register(id) => match id {
                ValueId::Var(v) => format!("v{}", v.0),
                ValueId::Temp(t) => format!("v{}", t.0),
                ValueId::Param(p) => {
                    let idx = p.0 as usize;
                    if idx < param_vnums.len() {
                        format!("v{}", param_vnums[idx])
                    } else {
                        format!("v{}", p.0)
                    }
                }
                ValueId::BlockParam(bp) => format!("bp{}_{}", bp.block.0, bp.index),
                ValueId::Storage(s) => format!("storage[{}]", s.0),
                ValueId::Memory(m) => format!("mem[{}]", m.0),
                ValueId::Global(g) => format!("@global_{}", g.0),
            },
            Value::Variable(id) => format!("v{}", id.0),
            Value::Temp(id) => format!("v{}", id.0),
            Value::Param(id) => {
                let idx = id.0 as usize;
                if idx < param_vnums.len() {
                    format!("v{}", param_vnums[idx])
                } else {
                    format!("v{}", id.0)
                }
            }
            Value::Constant(c) => Self::format_constant(c),
            Value::BlockParam(bp) => format!("bp{}_{}", bp.block.0, bp.index),
            Value::StorageRef(id) => format!("storage[{}]", id.0),
            Value::MemoryRef(id) => format!("mem[{}]", id.0),
            Value::Global(id) => format!("@global_{}", id.0),
            Value::Undefined => "undef".to_string(),
        }
    }

    pub fn format_constant(constant: &Constant) -> String {
        match constant {
            Constant::Uint(val, _bits) => format!("0x{:x}", val),
            Constant::Int(val, _bits) => format!("{}", val),
            Constant::Bool(b) => b.to_string(),
            Constant::String(s) => format!("\"{}\"", s),
            Constant::Bytes(b) => format!("#{}", Self::format_bytes(b)),
            Constant::Address(addr) => {
                format!("address(0x{})", Self::format_bytes(addr))
            }
            Constant::Null => "null".to_string(),
        }
    }

    pub fn format_bytes(bytes: &[u8]) -> String {
        bytes
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>()
    }

    pub fn format_storage_key(
        key: &StorageKey,
        ssa: &mut SSAContext,
        param_vnums: &[u32],
    ) -> String {
        match key {
            StorageKey::Slot(slot) => format!("slot{}", slot),
            StorageKey::Dynamic(val) => Self::format_value(val, ssa, param_vnums),
            StorageKey::Computed(val) => Self::format_value(val, ssa, param_vnums),
            StorageKey::MappingKey { base, key } => format!(
                "mapping[{}][{}]",
                base,
                Self::format_value(key, ssa, param_vnums)
            ),
            StorageKey::ArrayElement { base, index } => format!(
                "array[{}][{}]",
                base,
                Self::format_value(index, ssa, param_vnums)
            ),
        }
    }

    pub fn format_visibility(visibility: &Visibility) -> &'static str {
        match visibility {
            Visibility::Public => "public",
            Visibility::External => "external",
            Visibility::Internal => "internal",
            Visibility::Private => "private",
        }
    }

    pub fn format_mutability(mutability: &Mutability) -> &'static str {
        match mutability {
            Mutability::Pure => "pure",
            Mutability::View => "view",
            Mutability::Payable => "payable",
            Mutability::NonPayable => "",
        }
    }

    pub fn format_call_target(
        target: &CallTarget,
        ssa: &mut SSAContext,
        param_vnums: &[u32],
    ) -> String {
        match target {
            CallTarget::Internal(name) => format!("%{}", name),
            CallTarget::External(addr) => {
                format!("External({})", Self::format_value(addr, ssa, param_vnums))
            }
            CallTarget::Builtin(name) => format!("{:?}", name),
            CallTarget::Library(name) => format!("Library({})", name),
        }
    }

    pub fn format_terminator(
        terminator: &Terminator,
        ssa: &mut SSAContext,
        param_vnums: &[u32],
    ) -> String {
        match terminator {
            Terminator::Return(None) => "return".to_string(),
            Terminator::Return(Some(val)) => {
                format!("return {}", Self::format_value(val, ssa, param_vnums))
            }
            Terminator::Jump(target, args) => {
                if args.is_empty() {
                    format!("jump block{}", target.0)
                } else {
                    let args_str: Vec<String> = args
                        .iter()
                        .map(|v| Self::format_value(v, ssa, param_vnums))
                        .collect();
                    format!("jump block{}({})", target.0, args_str.join(", "))
                }
            }
            Terminator::Branch {
                condition,
                then_block,
                else_block,
                then_args,
                else_args,
            } => {
                let cond = Self::format_value(condition, ssa, param_vnums);
                if then_args.is_empty() && else_args.is_empty() {
                    format!("brz {}, block{}, block{}", cond, else_block.0, then_block.0)
                } else {
                    format!("brz {}, block{}, block{}", cond, else_block.0, then_block.0)
                }
            }
            Terminator::Switch {
                value,
                default,
                cases,
            } => {
                let val_str = Self::format_value(value, ssa, param_vnums);
                let mut result = format!("switch {} {{", val_str);
                for (case_val, target) in cases {
                    result.push_str(&format!(
                        " {}: block{},",
                        Self::format_value(case_val, ssa, param_vnums),
                        target.0
                    ));
                }
                result.push_str(&format!(" default: block{} }}", default.0));
                result
            }
            Terminator::Revert(msg) => format!("revert \"{}\"", msg),
            Terminator::Panic(msg) => format!("panic \"{}\"", msg),
            Terminator::Invalid => "invalid".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_type() {
        assert_eq!(IRFormatterBase::format_type(&Type::Uint(256)), "i256");
        assert_eq!(IRFormatterBase::format_type(&Type::Address), "address");
        assert_eq!(IRFormatterBase::format_type(&Type::Bool), "i1");
    }

    #[test]
    fn test_format_constant() {
        let constant = Constant::Bool(true);
        assert_eq!(IRFormatterBase::format_constant(&constant), "true");

        let constant = Constant::String("test".to_string());
        assert_eq!(IRFormatterBase::format_constant(&constant), "\"test\"");
    }

    #[test]
    fn test_format_visibility() {
        assert_eq!(
            IRFormatterBase::format_visibility(&Visibility::Public),
            "public"
        );
        assert_eq!(
            IRFormatterBase::format_visibility(&Visibility::External),
            "external"
        );
    }

    #[test]
    fn test_format_mutability() {
        assert_eq!(
            IRFormatterBase::format_mutability(&Mutability::Pure),
            "pure"
        );
        assert_eq!(
            IRFormatterBase::format_mutability(&Mutability::View),
            "view"
        );
        assert_eq!(
            IRFormatterBase::format_mutability(&Mutability::Payable),
            "payable"
        );
    }
}
