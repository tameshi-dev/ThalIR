use anyhow::{anyhow, Result};
use std::collections::HashMap;
use thalir_core::{
    block::BlockId,
    builder::{FunctionBuilderCursor, IRContext, IRRegistry},
    contract::Contract,
    function::{Function, Mutability, Visibility},
    types::Type,
    values::{SourceLocation, Value},
};
use tree_sitter::Node;

pub struct StructuralTransformerCursor {
    current_contract: Option<String>,
    state_vars: HashMap<String, (u32, Type)>,
    filename: String,
}

impl StructuralTransformerCursor {
    pub fn new() -> Self {
        Self {
            current_contract: None,
            state_vars: HashMap::new(),
            filename: "<unknown>".to_string(),
        }
    }

    pub fn with_filename(filename: String) -> Self {
        Self {
            current_contract: None,
            state_vars: HashMap::new(),
            filename,
        }
    }

    pub fn transform_with_context(
        &mut self,
        ast: &Node,
        source: &str,
        context: &mut IRContext,
        registry: &mut IRRegistry,
    ) -> Result<()> {
        let mut cursor = ast.walk();
        for child in ast.children(&mut cursor) {
            if child.kind() == "contract_declaration" {
                self.process_contract(context, registry, child, source)?;
            }
        }

        Ok(())
    }

    fn source_location_from_node(&self, node: Node) -> SourceLocation {
        SourceLocation::from_node(self.filename.clone(), &node)
    }

    fn process_contract(
        &mut self,
        context: &mut IRContext,
        registry: &mut IRRegistry,
        node: Node,
        source: &str,
    ) -> Result<()> {
        let name_node = node
            .child_by_field_name("name")
            .ok_or_else(|| anyhow!("Contract missing name"))?;
        let contract_name = &source[name_node.byte_range()];

        self.current_contract = Some(contract_name.to_string());
        self.state_vars.clear();

        let mut contract = Contract::new(contract_name.to_string());

        if let Some(body) = node.child_by_field_name("body") {
            let mut cursor = body.walk();
            for child in body.children(&mut cursor) {
                match child.kind() {
                    "state_variable_declaration" => {
                        self.process_state_variable(&mut contract, child, source)?;
                    }
                    "function_definition" => {
                        let function = self.process_function_cursor(
                            context,
                            registry,
                            &contract_name,
                            child,
                            source,
                        )?;
                        contract
                            .functions
                            .insert(function.signature.name.clone(), function);
                    }
                    _ => {}
                }
            }
        }

        registry.add_contract(contract)?;
        Ok(())
    }

    fn process_state_variable(
        &mut self,
        contract: &mut Contract,
        node: Node,
        source: &str,
    ) -> Result<()> {
        let name_node = node
            .child_by_field_name("name")
            .ok_or_else(|| anyhow!("State variable missing name"))?;
        let var_name = &source[name_node.byte_range()];

        let type_node = node
            .child_by_field_name("type")
            .ok_or_else(|| anyhow!("State variable missing type"))?;
        let var_type = self.resolve_type(type_node, source)?;

        let slot = self.state_vars.len() as u32;
        contract
            .storage_layout
            .add_variable(var_name.to_string(), var_type.clone(), slot);
        self.state_vars
            .insert(var_name.to_string(), (slot, var_type));

        Ok(())
    }

    fn process_function_cursor(
        &mut self,
        context: &mut IRContext,
        registry: &mut IRRegistry,
        contract_name: &str,
        node: Node,
        source: &str,
    ) -> Result<Function> {
        let name_node = node
            .child_by_field_name("name")
            .ok_or_else(|| anyhow!("Function missing name"))?;
        let func_name = &source[name_node.byte_range()];

        let mut func_builder = FunctionBuilderCursor::new(
            contract_name.to_string(),
            func_name.to_string(),
            context,
            registry,
        );

        let mut param_map = HashMap::new();
        if let Some(params_node) = node.child_by_field_name("parameters") {
            let mut param_idx = 0;
            let mut cursor = params_node.walk();
            for param_node in params_node.children(&mut cursor) {
                if param_node.kind() == "parameter" {
                    if let Some(param_name_node) = param_node.child_by_field_name("name") {
                        let param_name = &source[param_name_node.byte_range()];
                        let param_type =
                            if let Some(type_node) = param_node.child_by_field_name("type") {
                                self.resolve_type(type_node, source)?
                            } else {
                                Type::Uint(256)
                            };

                        func_builder.param(param_name, param_type);
                        param_map.insert(param_name.to_string(), param_idx);
                        param_idx += 1;
                    }
                }
            }
        }

        if let Some(returns_node) = node.child_by_field_name("return_type") {
            let mut cursor = returns_node.walk();
            for return_node in returns_node.children(&mut cursor) {
                if return_node.kind() == "parameter" {
                    if let Some(type_node) = return_node.child_by_field_name("type") {
                        let return_type = self.resolve_type(type_node, source)?;
                        func_builder.returns(return_type);
                    }
                }
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "visibility" => {
                    let vis_text = &source[child.byte_range()];
                    let visibility = match vis_text {
                        "public" => Visibility::Public,
                        "private" => Visibility::Private,
                        "internal" => Visibility::Internal,
                        "external" => Visibility::External,
                        _ => Visibility::Public,
                    };
                    func_builder.visibility(visibility);
                }
                "state_mutability" => {
                    let mut_text = &source[child.byte_range()];
                    let mutability = match mut_text {
                        "pure" => Mutability::Pure,
                        "view" => Mutability::View,
                        "payable" => Mutability::Payable,
                        _ => Mutability::NonPayable,
                    };
                    func_builder.mutability(mutability);
                }
                _ => {}
            }
        }

        if let Some(body_node) = node.child_by_field_name("body") {
            let entry_block = func_builder.entry_block();
            func_builder.switch_to_block(entry_block)?;

            let exit_block = self.process_block_statements(
                &mut func_builder,
                body_node,
                source,
                &param_map,
                &mut HashMap::new(),
            )?;

            if let Some(final_block) = exit_block {
                func_builder.switch_to_block(final_block)?;
                if !func_builder.is_terminated() {
                    let mut inst = func_builder.ins()?;
                    inst.return_void()?;
                }
            }
        }

        let function = func_builder.build()?;
        Ok(function)
    }

    fn process_block_statements(
        &mut self,
        func_builder: &mut FunctionBuilderCursor,
        node: Node,
        source: &str,
        param_map: &HashMap<String, u32>,
        local_vars: &mut HashMap<String, Value>,
    ) -> Result<Option<BlockId>> {
        let mut current_block = func_builder.current_block();

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "statement" {
                if let Some(actual_stmt) = child.child(0) {
                    match actual_stmt.kind() {
                        "if_statement" => {
                            current_block = Some(self.process_if_statement(
                                func_builder,
                                actual_stmt,
                                source,
                                param_map,
                                local_vars,
                            )?);

                            if let Some(block) = current_block {
                                func_builder.switch_to_block(block)?;
                            }
                            continue;
                        }
                        "while_statement" => {
                            current_block = Some(self.process_while_loop(
                                func_builder,
                                actual_stmt,
                                source,
                                param_map,
                                local_vars,
                            )?);

                            if let Some(block) = current_block {
                                func_builder.switch_to_block(block)?;
                            }
                            continue;
                        }
                        "for_statement" => {
                            current_block = Some(self.process_for_loop(
                                func_builder,
                                actual_stmt,
                                source,
                                param_map,
                                local_vars,
                            )?);

                            if let Some(block) = current_block {
                                func_builder.switch_to_block(block)?;
                            }
                            continue;
                        }
                        "return_statement" => {
                            let return_value = if actual_stmt.child_count() > 1 {
                                if let Some(value_node) = actual_stmt.child(1) {
                                    Some(self.process_expression(
                                        func_builder,
                                        value_node,
                                        source,
                                        param_map,
                                        local_vars,
                                    )?)
                                } else {
                                    None
                                }
                            } else {
                                None
                            };

                            let mut inst = func_builder.ins()?;
                            if let Some(value) = return_value {
                                inst.return_value(value)?;
                            } else {
                                inst.return_void()?;
                            }

                            return Ok(None);
                        }
                        "variable_declaration_statement" => {
                            self.process_variable_declaration(
                                func_builder,
                                actual_stmt,
                                source,
                                param_map,
                                local_vars,
                            )?;
                            continue;
                        }
                        "expression_statement" => {
                            if let Some(expr) = actual_stmt.child(0) {
                                self.process_expression(
                                    func_builder,
                                    expr,
                                    source,
                                    param_map,
                                    local_vars,
                                )?;
                            }
                            continue;
                        }
                        _ => {
                            self.process_expression(
                                func_builder,
                                actual_stmt,
                                source,
                                param_map,
                                local_vars,
                            )?;
                            continue;
                        }
                    }
                }
            }

            if !child.kind().ends_with("_statement") && child.kind() != "{" && child.kind() != "}" {
                continue;
            }

            if child.kind() == "{" || child.kind() == "}" {
                continue;
            }

            match child.kind() {
                "if_statement" => {
                    current_block = Some(self.process_if_statement(
                        func_builder,
                        child,
                        source,
                        param_map,
                        local_vars,
                    )?);

                    if let Some(block) = current_block {
                        func_builder.switch_to_block(block)?;
                    }
                }
                "while_statement" => {
                    current_block = Some(self.process_while_loop(
                        func_builder,
                        child,
                        source,
                        param_map,
                        local_vars,
                    )?);

                    if let Some(block) = current_block {
                        func_builder.switch_to_block(block)?;
                    }
                }
                "for_statement" => {
                    current_block = Some(self.process_for_loop(
                        func_builder,
                        child,
                        source,
                        param_map,
                        local_vars,
                    )?);

                    if let Some(block) = current_block {
                        func_builder.switch_to_block(block)?;
                    }
                }
                "return_statement" => {
                    let return_value = if child.child_count() > 1 {
                        if let Some(value_node) = child.child(1) {
                            Some(self.process_expression(
                                func_builder,
                                value_node,
                                source,
                                param_map,
                                local_vars,
                            )?)
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    let mut inst = func_builder.ins()?;
                    if let Some(value) = return_value {
                        inst.return_value(value)?;
                    } else {
                        inst.return_void()?;
                    }

                    return Ok(None);
                }
                "expression_statement" => {
                    if let Some(expr_node) = child.child(0) {
                        self.process_expression(
                            func_builder,
                            expr_node,
                            source,
                            param_map,
                            local_vars,
                        )?;
                    }
                }
                "variable_declaration_statement" => {
                    self.process_variable_declaration(
                        func_builder,
                        child,
                        source,
                        param_map,
                        local_vars,
                    )?;
                }
                _ => {}
            }
        }

        Ok(current_block)
    }

    fn process_if_statement(
        &mut self,
        func_builder: &mut FunctionBuilderCursor,
        node: Node,
        source: &str,
        param_map: &HashMap<String, u32>,
        local_vars: &mut HashMap<String, Value>,
    ) -> Result<BlockId> {
        let then_block = func_builder.create_block();
        let merge_block = func_builder.create_block();

        let else_node = node
            .child_by_field_name("alternative")
            .or_else(|| node.child_by_field_name("else"))
            .or_else(|| {
                let cursor = node.walk();
                for i in 0..node.child_count() {
                    if let Some(child) = node.child(i) {
                        if child.kind() == "else" || child.kind() == "else_clause" {
                            return Some(child);
                        }

                        if i > 0 && source[child.byte_range()].trim() == "else" {
                            return node.child(i + 1);
                        }
                    }
                }
                None
            });

        let else_block = if else_node.is_some() {
            Some(func_builder.create_block())
        } else {
            None
        };

        let condition = if let Some(cond_node) = node.child_by_field_name("condition") {
            self.process_expression(func_builder, cond_node, source, param_map, local_vars)?
        } else {
            let mut inst = func_builder.ins()?;
            inst.constant_bool(true)
        };

        {
            let mut inst = func_builder.ins()?;
            if let Some(else_block_id) = else_block {
                inst.branch(condition, then_block, else_block_id)?;
            } else {
                inst.branch(condition, then_block, merge_block)?;
            }
        }

        func_builder.switch_to_block(then_block)?;
        let mut then_needs_jump = true;
        if let Some(then_node) = node.child_by_field_name("consequence") {
            let final_block = self.process_block_statements(
                func_builder,
                then_node,
                source,
                param_map,
                local_vars,
            )?;

            if final_block.is_none() {
                then_needs_jump = false;
            } else if let Some(block) = final_block {
                func_builder.switch_to_block(block)?;
            }
        }

        if then_needs_jump && !func_builder.is_terminated() {
            let mut inst = func_builder.ins()?;
            inst.jump(merge_block)?;
        }

        if let Some(else_block_id) = else_block {
            func_builder.switch_to_block(else_block_id)?;
            let mut else_needs_jump = true;

            if let Some(else_node) = node.child_by_field_name("alternative") {
                let final_block = self.process_block_statements(
                    func_builder,
                    else_node,
                    source,
                    param_map,
                    local_vars,
                )?;

                if final_block.is_none() {
                    else_needs_jump = false;
                } else if let Some(block) = final_block {
                    func_builder.switch_to_block(block)?;
                }
            }

            if else_needs_jump && !func_builder.is_terminated() {
                let mut inst = func_builder.ins()?;
                inst.jump(merge_block)?;
            }
        }

        Ok(merge_block)
    }

    fn process_while_loop(
        &mut self,
        func_builder: &mut FunctionBuilderCursor,
        node: Node,
        source: &str,
        param_map: &HashMap<String, u32>,
        local_vars: &mut HashMap<String, Value>,
    ) -> Result<BlockId> {
        let header_block = func_builder.create_block();
        let body_block = func_builder.create_block();
        let exit_block = func_builder.create_block();

        {
            let mut inst = func_builder.ins()?;
            inst.jump(header_block)?;
        }

        func_builder.switch_to_block(header_block)?;
        let condition = if let Some(cond_node) = node.child_by_field_name("condition") {
            self.process_expression(func_builder, cond_node, source, param_map, local_vars)?
        } else {
            let mut inst = func_builder.ins()?;
            inst.constant_bool(true)
        };

        {
            let mut inst = func_builder.ins()?;
            inst.branch(condition, body_block, exit_block)?;
        }

        func_builder.switch_to_block(body_block)?;
        if let Some(body_node) = node.child_by_field_name("body") {
            let final_block = self.process_block_statements(
                func_builder,
                body_node,
                source,
                param_map,
                local_vars,
            )?;

            if let Some(block) = final_block {
                func_builder.switch_to_block(block)?;
                if !func_builder.is_terminated() {
                    let mut inst = func_builder.ins()?;
                    inst.jump(header_block)?;
                }
            }
        } else {
            let mut inst = func_builder.ins()?;
            inst.jump(header_block)?;
        }

        Ok(exit_block)
    }

    fn process_for_loop(
        &mut self,
        func_builder: &mut FunctionBuilderCursor,
        node: Node,
        source: &str,
        param_map: &HashMap<String, u32>,
        local_vars: &mut HashMap<String, Value>,
    ) -> Result<BlockId> {
        if let Some(init_node) = node.child_by_field_name("initializer") {
            if init_node.kind() == "variable_declaration_statement" {
                self.process_variable_declaration(
                    func_builder,
                    init_node,
                    source,
                    param_map,
                    local_vars,
                )?;
            } else {
                self.process_expression(func_builder, init_node, source, param_map, local_vars)?;
            }
        }

        let header_block = func_builder.create_block();
        let body_block = func_builder.create_block();
        let update_block = func_builder.create_block();
        let exit_block = func_builder.create_block();

        {
            let mut inst = func_builder.ins()?;
            inst.jump(header_block)?;
        }

        func_builder.switch_to_block(header_block)?;
        let condition = if let Some(cond_node) = node.child_by_field_name("condition") {
            self.process_expression(func_builder, cond_node, source, param_map, local_vars)?
        } else {
            let mut inst = func_builder.ins()?;
            inst.constant_bool(true)
        };

        {
            let mut inst = func_builder.ins()?;
            inst.branch(condition, body_block, exit_block)?;
        }

        func_builder.switch_to_block(body_block)?;
        if let Some(body_node) = node.child_by_field_name("body") {
            let final_block = self.process_block_statements(
                func_builder,
                body_node,
                source,
                param_map,
                local_vars,
            )?;

            if let Some(block) = final_block {
                func_builder.switch_to_block(block)?;
                if !func_builder.is_terminated() {
                    let mut inst = func_builder.ins()?;
                    inst.jump(update_block)?;
                }
            }
        } else {
            let mut inst = func_builder.ins()?;
            inst.jump(update_block)?;
        }

        func_builder.switch_to_block(update_block)?;
        if let Some(update_node) = node.child_by_field_name("update") {
            self.process_expression(func_builder, update_node, source, param_map, local_vars)?;
        }

        {
            let mut inst = func_builder.ins()?;
            inst.jump(header_block)?;
        }

        Ok(exit_block)
    }

    fn process_variable_declaration(
        &mut self,
        func_builder: &mut FunctionBuilderCursor,
        node: Node,
        source: &str,
        param_map: &HashMap<String, u32>,
        local_vars: &mut HashMap<String, Value>,
    ) -> Result<()> {
        if let Some(decl_node) = node
            .child_by_field_name("declaration")
            .or_else(|| node.child(0))
        {
            let name_node = decl_node.child_by_field_name("name").or_else(|| {
                let mut cursor = decl_node.walk();
                for child in decl_node.children(&mut cursor) {
                    if child.kind() == "identifier" {
                        return Some(child);
                    }
                }
                None
            });

            let init_value = if node.child_count() > 2 {
                if let Some(init_expr) = node.child(2) {
                    Some(self.process_expression(
                        func_builder,
                        init_expr,
                        source,
                        param_map,
                        local_vars,
                    )?)
                } else {
                    None
                }
            } else {
                None
            };

            if let (Some(name_node), Some(value)) = (name_node, init_value) {
                let name = &source[name_node.byte_range()];
                local_vars.insert(name.to_string(), value);
            }
        }

        Ok(())
    }

    fn process_expression(
        &mut self,
        func_builder: &mut FunctionBuilderCursor,
        node: Node,
        source: &str,
        param_map: &HashMap<String, u32>,
        local_vars: &mut HashMap<String, Value>,
    ) -> Result<Value> {
        func_builder.set_source_location(self.source_location_from_node(node));

        let mut inst = func_builder.ins()?;

        match node.kind() {
            "true" => Ok(inst.constant_bool(true)),
            "false" => Ok(inst.constant_bool(false)),

            "number_literal" => {
                let text = &source[node.byte_range()];
                let value = text.parse::<u64>().unwrap_or(0);
                Ok(inst.constant_uint(value, 256))
            }

            "identifier" => {
                let name = &source[node.byte_range()];

                if let Some(value) = local_vars.get(name) {
                    return Ok(value.clone());
                }

                if let Some(&param_idx) = param_map.get(name) {
                    return Ok(func_builder.get_param(param_idx as usize));
                }

                if let Some(&(slot, _)) = self.state_vars.get(name) {
                    let slot_val = inst.constant_uint(slot as u64, 256);
                    return Ok(inst.sload(slot_val));
                }

                match name {
                    "msg.sender" | "sender" => return Ok(inst.msg_sender()),
                    "msg.value" | "value" => return Ok(inst.msg_value()),
                    _ => {}
                }

                Ok(inst.constant_uint(0, 256))
            }

            "binary_expression" => {
                let left_node = node
                    .child_by_field_name("left")
                    .ok_or_else(|| anyhow!("Binary expression missing left operand"))?;
                let right_node = node
                    .child_by_field_name("right")
                    .ok_or_else(|| anyhow!("Binary expression missing right operand"))?;
                let op_node = node
                    .child_by_field_name("operator")
                    .ok_or_else(|| anyhow!("Binary expression missing operator"))?;

                let left = self.process_expression(
                    func_builder,
                    left_node,
                    source,
                    param_map,
                    local_vars,
                )?;
                let right = self.process_expression(
                    func_builder,
                    right_node,
                    source,
                    param_map,
                    local_vars,
                )?;

                let mut inst = func_builder.ins()?;
                let op = &source[op_node.byte_range()];

                match op {
                    "+" => Ok(inst.add(left, right, Type::Uint(256))),
                    "-" => Ok(inst.sub(left, right, Type::Uint(256))),
                    "*" => Ok(inst.mul(left, right, Type::Uint(256))),
                    "/" => Ok(inst.div(left, right, Type::Uint(256))),
                    "==" => Ok(inst.eq(left, right)),
                    "!=" => {
                        let eq_result = inst.eq(left.clone(), right.clone());
                        Ok(inst.not(eq_result))
                    }
                    "<" => Ok(inst.lt(left, right)),
                    ">" => Ok(inst.gt(left, right)),
                    "<=" => {
                        let gt_result = inst.gt(left.clone(), right.clone());
                        Ok(inst.not(gt_result))
                    }
                    ">=" => {
                        let lt_result = inst.lt(left.clone(), right.clone());
                        Ok(inst.not(lt_result))
                    }
                    "&&" => Ok(inst.and(left, right)),
                    "||" => Ok(inst.or(left, right)),
                    _ => Ok(inst.constant_uint(0, 256)),
                }
            }

            "assignment_expression" => {
                let left_node = node
                    .child_by_field_name("left")
                    .ok_or_else(|| anyhow!("Assignment missing left side"))?;
                let right_node = node
                    .child_by_field_name("right")
                    .ok_or_else(|| anyhow!("Assignment missing right side"))?;

                let value = self.process_expression(
                    func_builder,
                    right_node,
                    source,
                    param_map,
                    local_vars,
                )?;

                if left_node.kind() == "identifier" {
                    let name = &source[left_node.byte_range()];

                    local_vars.insert(name.to_string(), value.clone());

                    if let Some(&(slot, _)) = self.state_vars.get(name) {
                        let mut inst = func_builder.ins()?;
                        let slot_val = inst.constant_uint(slot as u64, 256);
                        inst.sstore(slot_val, value.clone())?;
                    }
                }

                Ok(value)
            }

            "call_expression" => {
                let mut inst = func_builder.ins()?;
                Ok(inst.constant_uint(0, 256))
            }

            _ => {
                let mut inst = func_builder.ins()?;
                Ok(inst.constant_uint(0, 256))
            }
        }
    }

    fn resolve_type(&self, node: Node, source: &str) -> Result<Type> {
        match node.kind() {
            "type_name" | "elementary_type_name" => {
                let type_text = &source[node.byte_range()];
                match type_text {
                    "bool" => Ok(Type::Bool),
                    "address" => Ok(Type::Address),
                    "string" => Ok(Type::String),
                    "bytes" => Ok(Type::Bytes(32)),
                    s if s.starts_with("uint") => {
                        let bits = s[4..].parse::<u16>().unwrap_or(256);
                        Ok(Type::Uint(bits))
                    }
                    s if s.starts_with("int") => {
                        let bits = s[3..].parse::<u16>().unwrap_or(256);
                        Ok(Type::Int(bits))
                    }
                    s if s.starts_with("bytes") => {
                        let size = s[5..].parse::<u8>().unwrap_or(32);
                        Ok(Type::Bytes(size))
                    }
                    _ => Ok(Type::Uint(256)),
                }
            }
            _ => Ok(Type::Uint(256)),
        }
    }
}
