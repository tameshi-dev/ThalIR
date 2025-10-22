use std::collections::HashMap;
use thalir_core::{block::BlockId, builder::IRBuilder, types::Type, values::Value};
use tree_sitter::Node;

pub trait TypeContext {
    fn get_node_text(&self, node: Node) -> &str;
    fn lookup_symbol(&self, name: &str) -> Option<&Symbol>;
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub ty: Type,
    pub value: Value,
    pub is_state_var: bool,
    pub slot: Option<u32>,
}

#[derive(Debug)]
pub struct Scope {
    symbols: HashMap<String, Symbol>,
    parent: Option<Box<Scope>>,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            parent: None,
        }
    }

    pub fn with_parent(parent: Scope) -> Self {
        Self {
            symbols: HashMap::new(),
            parent: Some(Box::new(parent)),
        }
    }

    pub fn insert(&mut self, name: String, symbol: Symbol) {
        self.symbols.insert(name, symbol);
    }

    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        self.symbols
            .get(name)
            .or_else(|| self.parent.as_ref().and_then(|p| p.lookup(name)))
    }

    pub fn lookup_mut(&mut self, name: &str) -> Option<&mut Symbol> {
        if self.symbols.contains_key(name) {
            self.symbols.get_mut(name)
        } else {
            self.parent.as_mut().and_then(|p| p.lookup_mut(name))
        }
    }
}

#[derive(Debug, Clone)]
pub struct ControlFlowContext {
    pub current_block: Option<BlockId>,
    pub loop_header: Option<BlockId>,
    pub loop_exit: Option<BlockId>,
    pub function_exit: Option<BlockId>,
}

pub struct TransformationContext<'a> {
    pub source: &'a str,
    pub builder: &'a mut IRBuilder,
    pub current_contract: Option<String>,
    pub current_function: Option<String>,
    pub scope_stack: Vec<Scope>,
    pub control_flow: ControlFlowContext,
    pub next_storage_slot: u32,
    pub errors: Vec<super::TransformError>,
}

pub struct SimpleContext<'a> {
    pub source: &'a str,
}

impl<'a> SimpleContext<'a> {
    pub fn new(source: &'a str) -> Self {
        Self { source }
    }
}

impl<'a> TypeContext for SimpleContext<'a> {
    fn get_node_text(&self, node: Node) -> &str {
        &self.source[node.byte_range()]
    }

    fn lookup_symbol(&self, _name: &str) -> Option<&Symbol> {
        None
    }
}

impl<'a> TransformationContext<'a> {
    pub fn new(source: &'a str, builder: &'a mut IRBuilder) -> Self {
        Self {
            source,
            builder,
            current_contract: None,
            current_function: None,
            scope_stack: vec![Scope::new()],
            control_flow: ControlFlowContext {
                current_block: None,
                loop_header: None,
                loop_exit: None,
                function_exit: None,
            },
            next_storage_slot: 0,
            errors: Vec::new(),
        }
    }

    pub fn get_node_text(&self, node: Node) -> &'a str {
        &self.source[node.byte_range()]
    }

    pub fn push_scope(&mut self) {
        let parent = self.scope_stack.pop().unwrap();
        let new_scope = Scope::with_parent(parent);
        self.scope_stack.push(new_scope);
    }

    pub fn pop_scope(&mut self) {
        if self.scope_stack.len() > 1 {
            let current = self.scope_stack.pop().unwrap();
            if let Some(parent) = current.parent {
                self.scope_stack.push(*parent);
            } else {
                self.scope_stack.push(Scope::new());
            }
        }
    }

    pub fn add_symbol(&mut self, name: String, symbol: Symbol) {
        if let Some(scope) = self.scope_stack.last_mut() {
            scope.insert(name, symbol);
        }
    }

    pub fn lookup_symbol(&self, name: &str) -> Option<&Symbol> {
        self.scope_stack.last()?.lookup(name)
    }

    pub fn allocate_storage_slot(&mut self) -> u32 {
        let slot = self.next_storage_slot;
        self.next_storage_slot += 1;
        slot
    }

    pub fn add_error(&mut self, error: super::TransformError) {
        self.errors.push(error);
    }

    pub fn get_source_location(&self, node: Node) -> (usize, usize) {
        let start = node.start_position();
        (start.row + 1, start.column + 1)
    }
}

impl<'a> TypeContext for TransformationContext<'a> {
    fn get_node_text(&self, node: Node) -> &str {
        &self.source[node.byte_range()]
    }

    fn lookup_symbol(&self, name: &str) -> Option<&Symbol> {
        for scope in self.scope_stack.iter().rev() {
            if let Some(symbol) = scope.lookup(name) {
                return Some(symbol);
            }
        }
        None
    }
}
