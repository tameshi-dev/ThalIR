use super::control_flow_builder::ControlFlowBuilder;
use super::expression_transformer::ExpressionTransformer;
use super::{context::SimpleContext, type_resolver::TypeResolver, IRTransformer};
use anyhow::Result;
use std::collections::HashMap;
use thalir_core::{
    builder::{BlockBuilder, ContractBuilder, IRBuilder, InstBuilderExt},
    function::{Mutability, Visibility},
    types::Type,
    values::{SourceLocation, Value},
};
use tree_sitter::Node;

pub struct StructuralTransformer {
    expression_transformer: ExpressionTransformer,
    control_flow_builder: ControlFlowBuilder,
    filename: String,
}

impl StructuralTransformer {
    pub fn new() -> Self {
        Self {
            expression_transformer: ExpressionTransformer::new(),
            control_flow_builder: ControlFlowBuilder::new(),
            filename: "<unknown>".to_string(),
        }
    }

    pub fn with_filename(filename: String) -> Self {
        Self {
            expression_transformer: ExpressionTransformer::new(),
            control_flow_builder: ControlFlowBuilder::new(),
            filename,
        }
    }

    fn source_location_from_node(&self, node: Node) -> SourceLocation {
        SourceLocation::from_node(self.filename.clone(), &node)
    }

    fn process_source_file(
        &mut self,
        node: Node,
        source: &str,
        builder: &mut IRBuilder,
    ) -> Result<()> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "contract_declaration" | "interface_declaration" | "library_declaration" => {
                    self.process_contract(child, source, builder)?;
                }
                "pragma_directive" | "import_directive" => {}
                _ => {}
            }
        }
        Ok(())
    }

    fn process_contract(
        &mut self,
        node: Node,
        source: &str,
        builder: &mut IRBuilder,
    ) -> Result<()> {
        let name = node
            .child_by_field_name("name")
            .map(|n| &source[n.byte_range()])
            .unwrap_or("UnnamedContract");

        let mut contract_builder = builder.contract(name);

        if let Some(body_node) = node.child_by_field_name("body") {
            let mut slot = 0u32;
            let mut cursor = body_node.walk();
            let mut state_vars = std::collections::HashMap::new();

            for child in body_node.children(&mut cursor) {
                if child.kind() == "state_variable_declaration" {
                    let var_name = child
                        .child_by_field_name("name")
                        .map(|n| &source[n.byte_range()])
                        .unwrap_or("unnamed");

                    let ty = if let Some(type_node) = child.child_by_field_name("type") {
                        let ctx = SimpleContext::new(source);
                        TypeResolver::resolve_type(type_node, &ctx)?
                    } else {
                        Type::Uint(256)
                    };

                    contract_builder.state_variable(var_name, ty.clone(), slot);
                    state_vars.insert(var_name.to_string(), (slot, ty));
                    slot += 1;
                }
            }

            cursor = body_node.walk();
            for child in body_node.children(&mut cursor) {
                match child.kind() {
                    "function_definition" => {
                        self.process_function_in_contract(
                            child,
                            source,
                            &mut contract_builder,
                            &state_vars,
                        )?;
                    }
                    "constructor_definition" => {
                        self.process_function_in_contract(
                            child,
                            source,
                            &mut contract_builder,
                            &state_vars,
                        )?;
                    }
                    _ => {}
                }
            }
        }

        contract_builder.build()?;
        Ok(())
    }

    fn process_function_in_contract(
        &mut self,
        node: Node,
        source: &str,
        contract_builder: &mut ContractBuilder,
        state_vars: &std::collections::HashMap<String, (u32, Type)>,
    ) -> Result<()> {
        let base_func_name = if node.kind() == "constructor_definition" {
            "constructor"
        } else {
            node.child_by_field_name("name")
                .map(|n| &source[n.byte_range()])
                .unwrap_or("unnamed")
        };

        let param_type_names = self.extract_parameter_type_names(node, source);

        let func_name = if param_type_names.is_empty() {
            base_func_name.to_string()
        } else {
            Self::mangle_function_name_from_strings(base_func_name, &param_type_names)
        };

        let mut func_builder = contract_builder.function(&func_name);

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            let text = &source[child.byte_range()];
            match text {
                "public" => func_builder.visibility(Visibility::Public),
                "external" => func_builder.visibility(Visibility::External),
                "internal" => func_builder.visibility(Visibility::Internal),
                "private" => func_builder.visibility(Visibility::Private),
                "pure" => func_builder.mutability(Mutability::Pure),
                "view" => func_builder.mutability(Mutability::View),
                "payable" => func_builder.mutability(Mutability::Payable),
                _ => &mut func_builder,
            };
        }

        let params_node = node.child_by_field_name("parameters").or_else(|| {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "parameter_list" || child.kind() == "call_argument" {
                    return Some(child);
                }
            }
            None
        });

        if let Some(params_node) = params_node {
            let mut cursor = params_node.walk();
            for child in params_node.children(&mut cursor) {
                if child.kind() == "parameter" {
                    let param_name = child
                        .child_by_field_name("name")
                        .map(|n| &source[n.byte_range()])
                        .unwrap_or("unnamed");

                    let ty = if let Some(type_node) = child.child_by_field_name("type") {
                        let ctx = SimpleContext::new(source);
                        TypeResolver::resolve_type(type_node, &ctx)?
                    } else {
                        Type::Uint(256)
                    };

                    func_builder.param(param_name, ty);
                }
            }
        } else {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "parameter" {
                    let param_name = child
                        .child_by_field_name("name")
                        .map(|n| &source[n.byte_range()])
                        .unwrap_or("unnamed");

                    let ty = if let Some(type_node) = child.child_by_field_name("type") {
                        let ctx = SimpleContext::new(source);
                        TypeResolver::resolve_type(type_node, &ctx)?
                    } else {
                        Type::Uint(256)
                    };

                    func_builder.param(param_name, ty);
                }
            }
        }

        if let Some(returns_node) = node.child_by_field_name("return_type") {
            if let Some(type_node) = returns_node.child_by_field_name("type") {
                let ctx = SimpleContext::new(source);
                let ty = TypeResolver::resolve_type(type_node, &ctx)?;
                func_builder.returns(ty);
            }
        }

        if let Some(body_node) = node.child_by_field_name("body") {
            let mut param_map = std::collections::HashMap::new();
            for (idx, param) in func_builder.get_params().iter().enumerate() {
                param_map.insert(param.name.clone(), idx as u32);
            }

            let has_control_flow = self.has_control_flow_statements(body_node);

            let mut entry_block = func_builder.entry_block();
            self.process_function_body(
                body_node,
                source,
                &mut entry_block,
                &param_map,
                state_vars,
            )?;
        } else {
            let mut entry_block = func_builder.entry_block();
            entry_block.return_void()?;
        }

        func_builder.build()?;
        Ok(())
    }

    fn has_control_flow_statements(&self, node: Node) -> bool {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            let actual_statement = if child.kind() == "statement" && child.child_count() > 0 {
                child.child(0).unwrap()
            } else {
                child
            };

            match actual_statement.kind() {
                "if_statement" | "for_statement" | "while_statement" => return true,
                _ => {}
            }
        }
        false
    }

    fn count_control_flow_statements(&self, node: Node, source: &str) -> usize {
        let mut count = 0;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            let actual_statement = if child.kind() == "statement" && child.child_count() > 0 {
                child.child(0).unwrap()
            } else {
                child
            };

            match actual_statement.kind() {
                "if_statement" => count += 1,
                "for_statement" | "while_statement" => count += 1,
                _ => {}
            }
        }
        count
    }

    #[allow(dead_code)]
    fn process_function_body_with_control_flow(
        &mut self,
        node: Node,
        source: &str,
        block: &mut thalir_core::builder::BlockBuilder,
        param_map: &std::collections::HashMap<String, u32>,
        state_vars: &std::collections::HashMap<String, (u32, Type)>,
        func_builder: &mut thalir_core::builder::FunctionBuilder,
        block_id_iter: &mut std::vec::IntoIter<thalir_core::block::BlockId>,
    ) -> Result<()> {
        use thalir_core::values::Value;
        let mut has_return = false;
        let mut return_value = None;

        let mut local_vars: std::collections::HashMap<String, Value> =
            std::collections::HashMap::new();

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            let actual_statement = if child.kind() == "statement" && child.child_count() > 0 {
                child.child(0).unwrap()
            } else {
                child
            };

            match actual_statement.kind() {
                "if_statement" => {
                    if let Some(condition_node) = actual_statement.child_by_field_name("condition")
                    {
                        let _ = self.process_expression(
                            condition_node,
                            source,
                            block,
                            param_map,
                            state_vars,
                            &mut local_vars,
                        )?;
                    }
                    // Not yet implemented: break/continue require loop context stack
                }
                "return_statement" => {
                    has_return = true;

                    if actual_statement.child_count() > 1 {
                        if let Some(value_node) = actual_statement.child(1) {
                            if value_node.kind() == "expression"
                                || value_node.kind().ends_with("_expression")
                            {
                                let actual_expr = if value_node.kind() == "expression"
                                    && value_node.child_count() > 0
                                {
                                    value_node.child(0).unwrap()
                                } else {
                                    value_node
                                };
                                return_value = Some(self.process_expression(
                                    actual_expr,
                                    source,
                                    block,
                                    param_map,
                                    state_vars,
                                    &mut local_vars,
                                )?);
                            }
                        }
                    }

                    break;
                }
                "expression_statement" => {
                    if let Some(expr_node) = actual_statement.child(0) {
                        let _ = self.process_expression(
                            expr_node,
                            source,
                            block,
                            param_map,
                            state_vars,
                            &mut local_vars,
                        )?;
                    }
                }
                "variable_declaration_statement" => {
                    if let Some(decl_node) = actual_statement.child_by_field_name("declaration") {
                        let name_node = decl_node.child_by_field_name("name");

                        let init_expr = actual_statement.child(2);

                        if let Some(init_expr) = init_expr {
                            let value = self.process_expression(
                                init_expr,
                                source,
                                block,
                                param_map,
                                state_vars,
                                &mut local_vars,
                            )?;

                            if let Some(name_node) = name_node {
                                let name = &source[name_node.byte_range()];
                                local_vars.insert(name.to_string(), value);
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        if let Some(val) = return_value {
            block.return_value(val)?;
        } else if !has_return {
            block.return_void()?;
        }

        Ok(())
    }

    #[allow(dead_code)]
    fn process_if_with_blocks(
        &mut self,
        node: Node,
        source: &str,
        mut current_block: thalir_core::builder::BlockBuilder,
        param_map: &std::collections::HashMap<String, u32>,
        state_vars: &std::collections::HashMap<String, (u32, Type)>,
        func_builder: &mut thalir_core::builder::FunctionBuilder,
        block_id_iter: &mut std::vec::IntoIter<thalir_core::block::BlockId>,
    ) -> Result<()> {
        let then_block_id = block_id_iter
            .next()
            .ok_or_else(|| anyhow::anyhow!("No block ID for then branch"))?;
        let else_block_id = block_id_iter
            .next()
            .ok_or_else(|| anyhow::anyhow!("No block ID for else branch"))?;
        let merge_block_id = block_id_iter
            .next()
            .ok_or_else(|| anyhow::anyhow!("No block ID for merge block"))?;

        let condition_node = node
            .child_by_field_name("condition")
            .ok_or_else(|| anyhow::anyhow!("If statement missing condition"))?;
        let condition = self.process_expression_simple(
            condition_node,
            source,
            &mut current_block,
            param_map,
            state_vars,
        )?;

        current_block.branch(condition, then_block_id, else_block_id)?;

        let then_body = node
            .child_by_field_name("body")
            .ok_or_else(|| anyhow::anyhow!("If statement missing then body"))?;
        let mut then_block = func_builder.block_with_id(then_block_id);
        let mut local_vars = std::collections::HashMap::new();
        self.process_block_with_jump(
            then_body,
            source,
            &mut then_block,
            param_map,
            state_vars,
            &mut local_vars,
            merge_block_id,
        )?;

        if let Some(else_node) = node.child_by_field_name("else") {
            let else_body = if else_node.kind() == "else_clause" && else_node.child_count() > 0 {
                else_node.child(else_node.child_count() - 1).unwrap()
            } else {
                else_node
            };

            let mut else_block = func_builder.block_with_id(else_block_id);
            self.process_block_with_jump(
                else_body,
                source,
                &mut else_block,
                param_map,
                state_vars,
                &mut local_vars,
                merge_block_id,
            )?;
        } else {
            let mut else_block = func_builder.block_with_id(else_block_id);
            else_block.jump(merge_block_id)?;
        }

        let mut merge_block = func_builder.block_with_id(merge_block_id);
        merge_block.return_void()?;

        Ok(())
    }

    fn process_block_with_jump(
        &mut self,
        node: Node,
        source: &str,
        block: &mut thalir_core::builder::BlockBuilder,
        param_map: &std::collections::HashMap<String, u32>,
        state_vars: &std::collections::HashMap<String, (u32, Type)>,
        local_vars: &mut std::collections::HashMap<String, thalir_core::values::Value>,
        jump_target: thalir_core::block::BlockId,
    ) -> Result<()> {
        let mut has_return = false;
        let mut return_value = None;

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            let actual_statement = if child.kind() == "statement" && child.child_count() > 0 {
                child.child(0).unwrap()
            } else {
                child
            };

            match actual_statement.kind() {
                "return_statement" => {
                    has_return = true;
                    if actual_statement.child_count() > 1 {
                        if let Some(value_node) = actual_statement.child(1) {
                            if value_node.kind() == "expression"
                                || value_node.kind().ends_with("_expression")
                            {
                                let actual_expr = if value_node.kind() == "expression"
                                    && value_node.child_count() > 0
                                {
                                    value_node.child(0).unwrap()
                                } else {
                                    value_node
                                };
                                return_value = Some(self.process_expression(
                                    actual_expr,
                                    source,
                                    block,
                                    param_map,
                                    state_vars,
                                    local_vars,
                                )?);
                            }
                        }
                    }
                    break;
                }
                "expression_statement" => {
                    if let Some(expr) = actual_statement.child(0) {
                        let _ = self.process_expression(
                            expr, source, block, param_map, state_vars, local_vars,
                        )?;
                    }
                }
                _ => {}
            }
        }

        if let Some(val) = return_value {
            block.return_value(val)?;
        } else {
            block.jump(jump_target)?;
        }

        Ok(())
    }

    fn process_function_body(
        &mut self,
        node: Node,
        source: &str,
        block: &mut thalir_core::builder::BlockBuilder,
        param_map: &std::collections::HashMap<String, u32>,
        state_vars: &std::collections::HashMap<String, (u32, Type)>,
    ) -> Result<()> {
        let mut local_vars: std::collections::HashMap<String, thalir_core::values::Value> =
            std::collections::HashMap::new();
        self.process_function_body_impl(
            node,
            source,
            block,
            param_map,
            state_vars,
            &mut local_vars,
            true,
        )
    }

    fn process_function_body_impl(
        &mut self,
        node: Node,
        source: &str,
        block: &mut thalir_core::builder::BlockBuilder,
        param_map: &std::collections::HashMap<String, u32>,
        state_vars: &std::collections::HashMap<String, (u32, Type)>,
        mut local_vars: &mut std::collections::HashMap<String, thalir_core::values::Value>,
        add_terminator: bool,
    ) -> Result<()> {
        let mut has_return = false;
        let mut return_value = None;

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            let actual_statement = if child.kind() == "statement" && child.child_count() > 0 {
                child.child(0).unwrap()
            } else {
                child
            };

            match actual_statement.kind() {
                "return_statement" => {
                    has_return = true;

                    if actual_statement.child_count() > 1 {
                        if let Some(value_node) = actual_statement.child(1) {
                            if value_node.kind() == "expression"
                                || value_node.kind().ends_with("_expression")
                            {
                                let actual_expr = if value_node.kind() == "expression"
                                    && value_node.child_count() > 0
                                {
                                    value_node.child(0).unwrap()
                                } else {
                                    value_node
                                };
                                return_value = Some(self.process_expression(
                                    actual_expr,
                                    source,
                                    block,
                                    param_map,
                                    state_vars,
                                    &mut local_vars,
                                )?);
                            }
                        }
                    }

                    break;
                }
                "expression_statement" => {
                    if let Some(expr) = actual_statement.child(0) {
                        let actual_expr = if expr.kind() == "expression" && expr.child_count() > 0 {
                            expr.child(0).unwrap()
                        } else {
                            expr
                        };
                        let _ = self.process_expression(
                            actual_expr,
                            source,
                            block,
                            param_map,
                            state_vars,
                            &mut local_vars,
                        )?;
                    }
                }
                "emit_statement" => {
                    if let Some(expr_node) = actual_statement.child(1) {
                        let call_node =
                            if expr_node.kind() == "expression" && expr_node.child_count() > 0 {
                                expr_node.child(0).unwrap()
                            } else {
                                expr_node
                            };

                        if call_node.kind() == "call_expression"
                            || call_node.kind() == "function_call_expression"
                        {
                            let event_name_node = call_node
                                .child_by_field_name("function")
                                .or_else(|| call_node.child(0));

                            if let Some(name_node) = event_name_node {
                                let event_name = &source[name_node.byte_range()];

                                let mut args = Vec::new();
                                let mut cursor = actual_statement.walk();
                                for child in actual_statement.children(&mut cursor) {
                                    if child.kind() == "call_argument" {
                                        let arg_value = self.process_expression(
                                            child,
                                            source,
                                            block,
                                            param_map,
                                            state_vars,
                                            &mut local_vars,
                                        )?;
                                        args.push(arg_value);
                                    }
                                }

                                let event_id = thalir_core::contract::EventId(0);
                                block.emit_event(event_id, Vec::new(), args);
                            }
                        }
                    }
                }
                "if_statement" => {
                    if let Some(condition_node) = actual_statement.child_by_field_name("condition")
                    {
                        self.process_expression(
                            condition_node,
                            source,
                            block,
                            param_map,
                            state_vars,
                            local_vars,
                        )?;

                        let then_node = actual_statement
                            .child_by_field_name("consequence")
                            .or_else(|| actual_statement.child_by_field_name("body"))
                            .or_else(|| {
                                let mut cursor = actual_statement.walk();
                                for child in actual_statement.children(&mut cursor) {
                                    let kind = child.kind();
                                    if kind == "statement"
                                        || kind == "statement_block"
                                        || kind == "block"
                                    {
                                        return Some(child);
                                    }
                                }
                                None
                            });

                        if let Some(then_node) = then_node {
                            if then_node.kind() == "statement" && then_node.child_count() > 0 {
                                if let Some(actual_body) = then_node.child(0) {
                                    self.process_function_body_impl(
                                        actual_body,
                                        source,
                                        block,
                                        param_map,
                                        state_vars,
                                        local_vars,
                                        false,
                                    )?;
                                }
                            } else {
                                self.process_function_body_impl(
                                    then_node, source, block, param_map, state_vars, local_vars,
                                    false,
                                )?;
                            }
                        }

                        if let Some(else_node) = actual_statement.child_by_field_name("alternative")
                        {
                            if else_node.kind() == "statement" && else_node.child_count() > 0 {
                                if let Some(actual_body) = else_node.child(0) {
                                    self.process_function_body_impl(
                                        actual_body,
                                        source,
                                        block,
                                        param_map,
                                        state_vars,
                                        local_vars,
                                        false,
                                    )?;
                                }
                            } else {
                                self.process_function_body_impl(
                                    else_node, source, block, param_map, state_vars, local_vars,
                                    false,
                                )?;
                            }
                        }
                    }
                }
                "while_statement" | "for_statement" => {
                    if let Some(condition_node) = actual_statement.child_by_field_name("condition")
                    {
                        self.process_expression(
                            condition_node,
                            source,
                            block,
                            param_map,
                            state_vars,
                            &mut local_vars,
                        )?;
                    }
                }
                "variable_declaration_statement" => {
                    if let Some(decl) = actual_statement
                        .child_by_field_name("declaration")
                        .or_else(|| actual_statement.child(0))
                    {
                        let name_node = decl.child_by_field_name("name").or_else(|| {
                            let mut cursor = decl.walk();
                            for child in decl.children(&mut cursor) {
                                if child.kind() == "identifier" {
                                    return Some(child);
                                }
                            }
                            None
                        });

                        let init_expr = actual_statement.child(2);

                        if let Some(init_expr) = init_expr {
                            let value = self.process_expression(
                                init_expr,
                                source,
                                block,
                                param_map,
                                state_vars,
                                &mut local_vars,
                            )?;

                            if let Some(name_node) = name_node {
                                let name = &source[name_node.byte_range()];
                                local_vars.insert(name.to_string(), value);
                            }
                        }
                    }
                }
                "expression_statement" => {
                    if let Some(expr_node) = actual_statement.child(0) {
                        let _ = self.process_expression(
                            expr_node,
                            source,
                            block,
                            param_map,
                            state_vars,
                            &mut local_vars,
                        )?;
                    }
                }
                _ => {}
            }
        }

        if add_terminator {
            if let Some(val) = return_value {
                block.return_value(val)?;
            } else if !has_return {
                block.return_void()?;
            }
        }

        Ok(())
    }

    fn process_expression_simple(
        &mut self,
        node: Node,
        source: &str,
        block: &mut thalir_core::builder::BlockBuilder,
        param_map: &std::collections::HashMap<String, u32>,
        state_vars: &std::collections::HashMap<String, (u32, Type)>,
    ) -> Result<thalir_core::values::Value> {
        let mut empty_locals = std::collections::HashMap::new();
        self.process_expression(
            node,
            source,
            block,
            param_map,
            state_vars,
            &mut empty_locals,
        )
    }

    fn process_expression(
        &mut self,
        node: Node,
        source: &str,
        block: &mut thalir_core::builder::BlockBuilder,
        param_map: &std::collections::HashMap<String, u32>,
        state_vars: &std::collections::HashMap<String, (u32, Type)>,
        local_vars: &mut std::collections::HashMap<String, thalir_core::values::Value>,
    ) -> Result<thalir_core::values::Value> {
        use thalir_core::types::Type;
        use thalir_core::values::Value;

        block.set_source_location(self.source_location_from_node(node));

        let actual_node = if node.kind() == "expression" && node.child_count() > 0 {
            node.child(0).unwrap()
        } else {
            node
        };

        match actual_node.kind() {
            "number_literal" => {
                let text = &source[actual_node.byte_range()];
                let value = text.parse::<u64>().unwrap_or(0);
                Ok(block.constant_uint(value, 256))
            }
            "identifier" => {
                let name = &source[actual_node.byte_range()];

                if let Some(value) = local_vars.get(name) {
                    Ok(value.clone())
                } else if let Some(&param_idx) = param_map.get(name) {
                    Ok(Value::Param(thalir_core::values::ParamId(param_idx)))
                } else if let Some(&(slot, ref ty)) = state_vars.get(name) {
                    let slot_bigint = num_bigint::BigUint::from(slot);
                    Ok(block.storage_load(slot_bigint))
                } else {
                    Ok(block.constant_uint(0, 256))
                }
            }
            "binary_expression" => {
                let left_node = actual_node.child_by_field_name("left").unwrap();
                let right_node = actual_node.child_by_field_name("right").unwrap();
                let op_node = actual_node.child_by_field_name("operator").unwrap();

                let left = self.process_expression(
                    left_node, source, block, param_map, state_vars, local_vars,
                )?;
                let right = self.process_expression(
                    right_node, source, block, param_map, state_vars, local_vars,
                )?;
                let op = &source[op_node.byte_range()];

                match op {
                    "+" => Ok(block.add(left, right, Type::Uint(256))),
                    "-" => Ok(block.sub(left, right, Type::Uint(256))),
                    "*" => Ok(block.mul(left, right, Type::Uint(256))),
                    "/" => Ok(block.div(left, right, Type::Uint(256))),
                    "%" => Ok(block.mod_(left, right, Type::Uint(256))),
                    "==" => Ok(block.eq(left, right)),
                    "!=" => Ok(block.ne(left, right)),
                    "<" => Ok(block.lt(left, right)),
                    "<=" => Ok(block.le(left, right)),
                    ">" => Ok(block.gt(left, right)),
                    ">=" => Ok(block.ge(left, right)),

                    "||" => {
                        let true_val = block.constant_bool(true);
                        Ok(block.select(left, true_val, right))
                    }
                    "&&" => {
                        let false_val = block.constant_bool(false);
                        Ok(block.select(left, right, false_val))
                    }

                    "&" => Ok(block.and(left, right)),
                    "|" => Ok(block.or(left, right)),
                    "^" => Ok(block.xor(left, right)),
                    "<<" => Ok(block.shl(left, right)),
                    ">>" => Ok(block.shr(left, right)),
                    _ => Ok(left),
                }
            }
            "assignment_expression" => {
                let left_node = actual_node.child_by_field_name("left").unwrap();
                let right_node = actual_node.child_by_field_name("right").unwrap();

                let value = self.process_expression(
                    right_node, source, block, param_map, state_vars, local_vars,
                )?;

                let actual_left = if left_node.kind() == "expression" && left_node.child_count() > 0
                {
                    left_node.child(0).unwrap()
                } else {
                    left_node
                };
                match actual_left.kind() {
                    "identifier" => {
                        let name = &source[actual_left.byte_range()];

                        if let Some(&(slot, ref ty)) = state_vars.get(name) {
                            let slot_bigint = num_bigint::BigUint::from(slot);
                            block.storage_store(slot_bigint, value.clone());
                        }
                    }
                    "index_access_expression" | "subscript_expression" | "array_access" => {
                        let base_node = actual_left
                            .child_by_field_name("base")
                            .or_else(|| actual_left.child_by_field_name("object"))
                            .or_else(|| actual_left.child(0));
                        let index_node = actual_left.child_by_field_name("index").or_else(|| {
                            let mut cursor = actual_left.walk();
                            for child in actual_left.children(&mut cursor) {
                                if child.kind() != "["
                                    && child.kind() != "]"
                                    && child.kind() != "identifier"
                                    && child.kind() != "member_expression"
                                {
                                    return Some(child);
                                }
                            }
                            None
                        });

                        if let (Some(base), Some(index)) = (base_node, index_node) {
                            let base_name = &source[base.byte_range()];

                            if let Some(&(slot, ref ty)) = state_vars.get(base_name) {
                                match ty {
                                    Type::Mapping(_, _) => {
                                        let key = self.process_expression(
                                            index, source, block, param_map, state_vars, local_vars,
                                        )?;

                                        let mapping =
                                            Value::Constant(thalir_core::values::Constant::Uint(
                                                num_bigint::BigUint::from(slot),
                                                256,
                                            ));
                                        block.mapping_store(mapping, key, value.clone());
                                    }
                                    Type::Array(_, _) => {
                                        let index_val = self.process_expression(
                                            index, source, block, param_map, state_vars, local_vars,
                                        )?;
                                        let array =
                                            Value::Constant(thalir_core::values::Constant::Uint(
                                                num_bigint::BigUint::from(slot),
                                                256,
                                            ));
                                        block.array_store(array, index_val, value.clone());
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    _ => {}
                }

                Ok(value)
            }
            "augmented_assignment_expression" => {
                let left_node = actual_node.child_by_field_name("left").unwrap();
                let right_node = actual_node.child_by_field_name("right").unwrap();

                let operator_node = actual_node.child(1).unwrap();

                let right_value = self.process_expression(
                    right_node, source, block, param_map, state_vars, local_vars,
                )?;
                let operator = &source[operator_node.byte_range()];

                let actual_left = if left_node.kind() == "expression" && left_node.child_count() > 0
                {
                    left_node.child(0).unwrap()
                } else {
                    left_node
                };

                match actual_left.kind() {
                    "identifier" => {
                        let name = &source[actual_left.byte_range()];

                        if let Some(&(slot, ref _ty)) = state_vars.get(name) {
                            let slot_bigint = num_bigint::BigUint::from(slot);

                            let current = block.storage_load(slot_bigint.clone());

                            let new_value = match operator {
                                "+=" => block.add(current, right_value.clone(), Type::Uint(256)),
                                "-=" => block.sub(current, right_value.clone(), Type::Uint(256)),
                                "*=" => block.mul(current, right_value.clone(), Type::Uint(256)),
                                "/=" => block.div(current, right_value.clone(), Type::Uint(256)),
                                "%=" => block.mod_(current, right_value.clone(), Type::Uint(256)),
                                _ => right_value.clone(),
                            };

                            block.storage_store(slot_bigint, new_value.clone());
                            Ok(new_value)
                        } else {
                            Ok(right_value)
                        }
                    }
                    "index_access_expression" | "subscript_expression" | "array_access" => {
                        let base_node = actual_left
                            .child_by_field_name("base")
                            .or_else(|| actual_left.child_by_field_name("object"))
                            .or_else(|| actual_left.child(0));
                        let index_node = actual_left.child_by_field_name("index").or_else(|| {
                            let mut cursor = actual_left.walk();
                            for child in actual_left.children(&mut cursor) {
                                if child.kind() != "["
                                    && child.kind() != "]"
                                    && child.kind() != "identifier"
                                    && child.kind() != "member_expression"
                                {
                                    return Some(child);
                                }
                            }
                            None
                        });

                        if let (Some(base), Some(index)) = (base_node, index_node) {
                            let base_name = &source[base.byte_range()];

                            if let Some(&(slot, ref ty)) = state_vars.get(base_name) {
                                match ty {
                                    Type::Mapping(_, _) => {
                                        let key = self.process_expression(
                                            index, source, block, param_map, state_vars, local_vars,
                                        )?;
                                        let mapping =
                                            Value::Constant(thalir_core::values::Constant::Uint(
                                                num_bigint::BigUint::from(slot),
                                                256,
                                            ));

                                        let current =
                                            block.mapping_load(mapping.clone(), key.clone());

                                        let new_value = match operator {
                                            "+=" => block.add(
                                                current,
                                                right_value.clone(),
                                                Type::Uint(256),
                                            ),
                                            "-=" => block.sub(
                                                current,
                                                right_value.clone(),
                                                Type::Uint(256),
                                            ),
                                            "*=" => block.mul(
                                                current,
                                                right_value.clone(),
                                                Type::Uint(256),
                                            ),
                                            "/=" => block.div(
                                                current,
                                                right_value.clone(),
                                                Type::Uint(256),
                                            ),
                                            "%=" => block.mod_(
                                                current,
                                                right_value.clone(),
                                                Type::Uint(256),
                                            ),
                                            _ => right_value.clone(),
                                        };

                                        block.mapping_store(mapping, key, new_value.clone());
                                        Ok(new_value)
                                    }
                                    Type::Array(_, _) => {
                                        let index_val = self.process_expression(
                                            index, source, block, param_map, state_vars, local_vars,
                                        )?;
                                        let array =
                                            Value::Constant(thalir_core::values::Constant::Uint(
                                                num_bigint::BigUint::from(slot),
                                                256,
                                            ));

                                        let current =
                                            block.array_load(array.clone(), index_val.clone());

                                        let new_value = match operator {
                                            "+=" => block.add(
                                                current,
                                                right_value.clone(),
                                                Type::Uint(256),
                                            ),
                                            "-=" => block.sub(
                                                current,
                                                right_value.clone(),
                                                Type::Uint(256),
                                            ),
                                            "*=" => block.mul(
                                                current,
                                                right_value.clone(),
                                                Type::Uint(256),
                                            ),
                                            "/=" => block.div(
                                                current,
                                                right_value.clone(),
                                                Type::Uint(256),
                                            ),
                                            "%=" => block.mod_(
                                                current,
                                                right_value.clone(),
                                                Type::Uint(256),
                                            ),
                                            _ => right_value.clone(),
                                        };

                                        block.array_store(array, index_val, new_value.clone());
                                        Ok(new_value)
                                    }
                                    _ => Ok(right_value),
                                }
                            } else {
                                Ok(right_value)
                            }
                        } else {
                            Ok(right_value)
                        }
                    }
                    _ => Ok(right_value),
                }
            }
            "call_expression" | "function_call_expression" => {
                let function_node = actual_node
                    .child_by_field_name("function")
                    .or_else(|| actual_node.child(0));

                if let Some(func_node) = function_node {
                    let func_text = &source[func_node.byte_range()];

                    if func_node.kind() == "member_expression"
                        || func_node.kind() == "member_access_expression"
                    {
                        let obj_node = func_node
                            .child_by_field_name("object")
                            .or_else(|| func_node.child(0));
                        let member_node = func_node
                            .child_by_field_name("property")
                            .or_else(|| func_node.child_by_field_name("member"))
                            .or_else(|| func_node.child(2));

                        if let (Some(obj), Some(member)) = (obj_node, member_node) {
                            let member_name = &source[member.byte_range()];

                            if member_name == "transfer"
                                || member_name == "send"
                                || member_name == "call"
                            {
                                let target = if obj.kind() == "call_expression" {
                                    let obj_text = &source[obj.byte_range()];
                                    if obj_text.starts_with("payable(") {
                                        let mut target_value = None;
                                        let mut cursor = obj.walk();
                                        for (i, child) in obj.children(&mut cursor).enumerate() {
                                            if child.kind() == "call_argument"
                                                || (child.kind() == "identifier"
                                                    && &source[child.byte_range()] != "payable")
                                            {
                                                let arg_expr = if child.kind() == "call_argument"
                                                    && child.child_count() > 0
                                                {
                                                    child.child(0).unwrap()
                                                } else {
                                                    child
                                                };

                                                if let Ok(value) = self.process_expression(
                                                    arg_expr, source, block, param_map, state_vars,
                                                    local_vars,
                                                ) {
                                                    target_value = Some(value);
                                                    break;
                                                }
                                            }
                                        }
                                        target_value.unwrap_or_else(|| block.constant_uint(0, 160))
                                    } else {
                                        block.constant_uint(0, 160)
                                    }
                                } else {
                                    self.process_expression(
                                        obj, source, block, param_map, state_vars, local_vars,
                                    )
                                    .unwrap_or_else(|_| block.constant_uint(0, 160))
                                };

                                match member_name {
                                    "transfer" => {
                                        let amount = block.constant_uint(100, 256);
                                        let selector = block.constant_uint(0, 32);
                                        let result = block.call_external(
                                            target,
                                            selector,
                                            vec![],
                                            Some(amount),
                                        );
                                        block.require(result.clone(), "Transfer failed");
                                        return Ok(block.constant_uint(0, 256));
                                    }
                                    "send" => {
                                        let amount = block.constant_uint(100, 256);
                                        let selector = block.constant_uint(0, 32);
                                        return Ok(block.call_external(
                                            target,
                                            selector,
                                            vec![],
                                            Some(amount),
                                        ));
                                    }
                                    "call" => {
                                        let amount = block.constant_uint(100, 256);
                                        let selector = block.constant_uint(0, 32);
                                        let result = block.call_external(
                                            target,
                                            selector,
                                            vec![],
                                            Some(amount),
                                        );

                                        return Ok(result);
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }

                    if func_text == "payable" {
                        if let Some(args_node) = actual_node.child_by_field_name("arguments") {
                            let mut cursor = args_node.walk();
                            for child in args_node.children(&mut cursor) {
                                if child.kind() != "," && child.kind() != "(" && child.kind() != ")"
                                {
                                    return self.process_expression(
                                        child, source, block, param_map, state_vars, local_vars,
                                    );
                                }
                            }
                        }

                        return Ok(block.constant_uint(0, 160));
                    }

                    let actual_func =
                        if func_node.kind() == "expression" && func_node.child_count() > 0 {
                            func_node.child(0).unwrap()
                        } else {
                            func_node
                        };

                    if actual_func.kind() == "member_expression"
                        || actual_func.kind() == "member_access_expression"
                        || actual_func.kind() == "struct_expression"
                    {
                        let (obj_node, member_node) = if actual_func.kind() == "struct_expression" {
                            let base = actual_func.child(0);
                            if let Some(base) = base {
                                let actual_base =
                                    if base.kind() == "expression" && base.child_count() > 0 {
                                        base.child(0).unwrap()
                                    } else {
                                        base
                                    };

                                if actual_base.kind() == "member_expression"
                                    || actual_base.kind() == "member_access_expression"
                                {
                                    let obj = actual_base
                                        .child_by_field_name("object")
                                        .or_else(|| actual_base.child(0));
                                    let member = actual_base
                                        .child_by_field_name("property")
                                        .or_else(|| actual_base.child_by_field_name("member"))
                                        .or_else(|| actual_base.child(2));
                                    (obj, member)
                                } else {
                                    (None, None)
                                }
                            } else {
                                (None, None)
                            }
                        } else {
                            let obj = actual_func
                                .child_by_field_name("object")
                                .or_else(|| actual_func.child(0));
                            let member = actual_func
                                .child_by_field_name("property")
                                .or_else(|| actual_func.child_by_field_name("member"))
                                .or_else(|| actual_func.child(2));
                            (obj, member)
                        };

                        if let (Some(obj), Some(member)) = (obj_node, member_node) {
                            let obj_name = &source[obj.byte_range()];
                            let member_name = &source[member.byte_range()];

                            if obj_name == "super" {
                                let mut args = Vec::new();
                                if let Some(args_node) =
                                    actual_node.child_by_field_name("arguments")
                                {
                                    let mut cursor = args_node.walk();
                                    for child in args_node.children(&mut cursor) {
                                        if child.kind() != ","
                                            && child.kind() != "("
                                            && child.kind() != ")"
                                        {
                                            let arg_value = self.process_expression(
                                                child, source, block, param_map, state_vars,
                                                local_vars,
                                            )?;
                                            args.push(arg_value);
                                        }
                                    }
                                }

                                let parent_func_name = format!("_super_{}", member_name);
                                return Ok(block.call_internal(&parent_func_name, args));
                            }

                            if member_name == "transfer"
                                || member_name == "send"
                                || member_name == "call"
                            {
                                let target = if obj_name.starts_with("payable(")
                                    && obj_name.ends_with(")")
                                {
                                    if obj.kind() == "call_expression" {
                                        let mut target_value = None;
                                        let mut cursor = obj.walk();
                                        for child in obj.children(&mut cursor) {
                                            if child.kind() == "call_argument" {
                                                let arg_expr = if child.child_count() > 0 {
                                                    child.child(0).unwrap()
                                                } else {
                                                    child
                                                };

                                                if let Ok(value) = self.process_expression(
                                                    arg_expr, source, block, param_map, state_vars,
                                                    local_vars,
                                                ) {
                                                    target_value = Some(value);
                                                    break;
                                                }
                                            }
                                        }
                                        target_value.unwrap_or_else(|| block.constant_uint(0, 160))
                                    } else {
                                        let inner = obj_name
                                            .trim_start_matches("payable(")
                                            .trim_end_matches(")");
                                        if let Some(&param_idx) = param_map.get(inner) {
                                            Value::Param(thalir_core::values::ParamId(param_idx))
                                        } else if let Some(value) = local_vars.get(inner) {
                                            value.clone()
                                        } else {
                                            block.constant_uint(0, 160)
                                        }
                                    }
                                } else {
                                    if let Ok(value) = self.process_expression(
                                        obj, source, block, param_map, state_vars, local_vars,
                                    ) {
                                        value
                                    } else {
                                        block.constant_uint(0, 160)
                                    }
                                };

                                match member_name {
                                    "transfer" => {
                                        let amount = block.constant_uint(100, 256);
                                        let selector = block.constant_uint(0, 32);
                                        let result = block.call_external(
                                            target.clone(),
                                            selector,
                                            vec![],
                                            Some(amount),
                                        );
                                        block.require(result.clone(), "Transfer failed");
                                        return Ok(block.constant_uint(0, 256));
                                    }
                                    "send" => {
                                        let amount = block.constant_uint(100, 256);
                                        let selector = block.constant_uint(0, 32);
                                        let result = block.call_external(
                                            target.clone(),
                                            selector,
                                            vec![],
                                            Some(amount),
                                        );
                                        return Ok(result);
                                    }
                                    "call" => {
                                        let amount = block.constant_uint(100, 256);
                                        let selector = block.constant_uint(0, 32);
                                        let result = block.call_external(
                                            target.clone(),
                                            selector,
                                            vec![],
                                            Some(amount),
                                        );
                                        return Ok(result);
                                    }
                                    _ => {}
                                }
                            }

                            let obj_base_name = obj_name.split('.').next().unwrap_or(obj_name);

                            if let Some(&(slot, ref ty)) = state_vars.get(obj_base_name) {
                                if let Type::Array(_, _) = ty {
                                    match member_name {
                                        "push" => {
                                            let mut args = Vec::new();
                                            let mut cursor = actual_node.walk();
                                            for child in actual_node.children(&mut cursor) {
                                                if child.kind() == "call_argument" {
                                                    let arg_expr = if child.child_count() > 0 {
                                                        child.child(0).unwrap()
                                                    } else {
                                                        child
                                                    };
                                                    let arg_value = self.process_expression(
                                                        arg_expr, source, block, param_map,
                                                        state_vars, local_vars,
                                                    )?;
                                                    args.push(arg_value);
                                                }
                                            }

                                            if let Some(value) = args.first() {
                                                let array = Value::Constant(
                                                    thalir_core::values::Constant::Uint(
                                                        num_bigint::BigUint::from(slot),
                                                        256,
                                                    ),
                                                );
                                                block.array_push(array, value.clone());
                                            }
                                            return Ok(block.constant_uint(0, 256));
                                        }
                                        "pop" => {
                                            let array = Value::Constant(
                                                thalir_core::values::Constant::Uint(
                                                    num_bigint::BigUint::from(slot),
                                                    256,
                                                ),
                                            );
                                            return Ok(block.array_pop(array));
                                        }
                                        _ => {}
                                    }
                                } else if matches!(
                                    ty,
                                    Type::Address | Type::Contract(_) | Type::String
                                ) {
                                    let mut args = Vec::new();
                                    let mut cursor = actual_node.walk();
                                    for child in actual_node.children(&mut cursor) {
                                        if child.kind() == "call_argument" {
                                            let arg_expr = if child.child_count() > 0 {
                                                child.child(0).unwrap()
                                            } else {
                                                child
                                            };
                                            let arg_value = self.process_expression(
                                                arg_expr, source, block, param_map, state_vars,
                                                local_vars,
                                            )?;
                                            args.push(arg_value);
                                        }
                                    }

                                    let target =
                                        block.storage_load(num_bigint::BigUint::from(slot));

                                    let selector = block.constant_uint(0, 32);
                                    return Ok(block.call_external(target, selector, args, None));
                                }
                            }
                        }
                    }

                    let func_name = &source[func_node.byte_range()];

                    if func_name == "type" {
                        return Ok(block.constant_uint(0, 32));
                    }

                    match func_name {
                        "require" => {
                            let mut cursor = actual_node.walk();
                            let mut condition = None;
                            let mut message = None;
                            let mut arg_count = 0;

                            for child in actual_node.children(&mut cursor) {
                                if child.kind() == "call_argument" {
                                    arg_count += 1;
                                    if arg_count == 1 {
                                        let mut cond_expr = child;
                                        if child.child_count() > 0 {
                                            cond_expr = child.child(0).unwrap();
                                        }

                                        if cond_expr.kind() == "expression"
                                            && cond_expr.child_count() > 0
                                        {
                                            cond_expr = cond_expr.child(0).unwrap();
                                        }

                                        condition = Some(self.process_expression(
                                            cond_expr, source, block, param_map, state_vars,
                                            local_vars,
                                        )?);
                                    } else if arg_count == 2 && message.is_none() {
                                        if child.kind() == "string_literal" {
                                            let text = &source[child.byte_range()];

                                            let msg =
                                                text.trim_start_matches('"').trim_end_matches('"');
                                            message = Some(msg.to_string());
                                        }
                                    }
                                }
                            }

                            if let Some(cond) = condition {
                                block.require(
                                    cond,
                                    message.as_deref().unwrap_or("Require condition failed"),
                                );
                            }
                            Ok(block.constant_uint(0, 256))
                        }
                        "assert" => {
                            let mut cursor = actual_node.walk();
                            for child in actual_node.children(&mut cursor) {
                                if child.kind() == "call_argument" {
                                    let condition = self.process_expression(
                                        child, source, block, param_map, state_vars, local_vars,
                                    )?;
                                    block.assert(condition, "Assertion failed");
                                    break;
                                }
                            }
                            Ok(block.constant_uint(0, 256))
                        }
                        "revert" => {
                            let mut message = "Transaction reverted";
                            if let Some(args_node) = actual_node.child_by_field_name("arguments") {
                                let mut cursor = args_node.walk();
                                for child in args_node.children(&mut cursor) {
                                    if child.kind() == "string_literal" {
                                        let text = &source[child.byte_range()];
                                        message =
                                            text.trim_start_matches('"').trim_end_matches('"');
                                        break;
                                    }
                                }
                            }

                            Ok(block.constant_uint(0, 256))
                        }
                        _ => {
                            if func_name.ends_with(".transfer")
                                || func_name.ends_with(".send")
                                || func_name.contains(".call")
                            {
                                let method = if func_name.ends_with(".transfer") {
                                    "transfer"
                                } else if func_name.ends_with(".send") {
                                    "send"
                                } else if func_name.contains(".call") {
                                    "call"
                                } else {
                                    ""
                                };

                                let mut cursor = actual_node.walk();
                                for (i, child) in actual_node.children(&mut cursor).enumerate() {}

                                let target = if let Some(func_node) =
                                    actual_node.child_by_field_name("function")
                                {
                                    if let Some(obj_node) = func_node.child_by_field_name("object")
                                    {
                                        if obj_node.kind() == "call_expression" {
                                            let mut target_value = None;
                                            let mut cursor = obj_node.walk();
                                            for child in obj_node.children(&mut cursor) {
                                                if child.kind() == "call_argument" {
                                                    let arg_expr = if child.child_count() > 0 {
                                                        child.child(0).unwrap()
                                                    } else {
                                                        child
                                                    };

                                                    if let Ok(value) = self.process_expression(
                                                        arg_expr, source, block, param_map,
                                                        state_vars, local_vars,
                                                    ) {
                                                        target_value = Some(value);
                                                    }
                                                    break;
                                                } else if child.kind() == "identifier" {
                                                    let name = &source[child.byte_range()];
                                                    if name != "payable" {
                                                        if let Some(&param_idx) =
                                                            param_map.get(name)
                                                        {
                                                            target_value = Some(Value::Param(
                                                                thalir_core::values::ParamId(
                                                                    param_idx,
                                                                ),
                                                            ));
                                                        } else if let Some(value) =
                                                            local_vars.get(name)
                                                        {
                                                            target_value = Some(value.clone());
                                                        }
                                                        break;
                                                    }
                                                }
                                            }
                                            target_value
                                                .unwrap_or_else(|| block.constant_uint(0, 160))
                                        } else {
                                            self.process_expression(
                                                obj_node, source, block, param_map, state_vars,
                                                local_vars,
                                            )
                                            .unwrap_or_else(|_| block.constant_uint(0, 160))
                                        }
                                    } else {
                                        block.constant_uint(0, 160)
                                    }
                                } else {
                                    block.constant_uint(0, 160)
                                };

                                match method {
                                    "transfer" => {
                                        let amount = if let Some(args_node) =
                                            actual_node.child_by_field_name("arguments")
                                        {
                                            let mut cursor = args_node.walk();
                                            for child in args_node.children(&mut cursor) {
                                                if child.kind() != ","
                                                    && child.kind() != "("
                                                    && child.kind() != ")"
                                                {
                                                    return Ok(self.process_expression(
                                                        child, source, block, param_map,
                                                        state_vars, local_vars,
                                                    )?);
                                                }
                                            }
                                            block.constant_uint(0, 256)
                                        } else {
                                            block.constant_uint(0, 256)
                                        };

                                        let selector = block.constant_uint(0, 32);
                                        let result = block.call_external(
                                            target,
                                            selector,
                                            vec![],
                                            Some(amount),
                                        );

                                        block.require(result.clone(), "Transfer failed");
                                        return Ok(block.constant_uint(0, 256));
                                    }
                                    "send" => {
                                        let amount = if let Some(args_node) =
                                            actual_node.child_by_field_name("arguments")
                                        {
                                            let mut cursor = args_node.walk();
                                            for child in args_node.children(&mut cursor) {
                                                if child.kind() != ","
                                                    && child.kind() != "("
                                                    && child.kind() != ")"
                                                {
                                                    return Ok(self.process_expression(
                                                        child, source, block, param_map,
                                                        state_vars, local_vars,
                                                    )?);
                                                }
                                            }
                                            block.constant_uint(0, 256)
                                        } else {
                                            block.constant_uint(0, 256)
                                        };

                                        let selector = block.constant_uint(0, 32);
                                        return Ok(block.call_external(
                                            target,
                                            selector,
                                            vec![],
                                            Some(amount),
                                        ));
                                    }
                                    "call" => {
                                        let selector = block.constant_uint(0, 32);
                                        let value = block.msg_value();
                                        return Ok(block.call_external(
                                            target,
                                            selector,
                                            vec![],
                                            Some(value),
                                        ));
                                    }
                                    _ => {}
                                }

                                return Ok(block.constant_uint(0, 256));
                            }

                            if func_name.contains("==")
                                || func_name.contains("||")
                                || func_name.contains("&&")
                                || func_name.contains('\n')
                                || func_name.contains("type(")
                                || func_name.contains("super.")
                            {
                                let expr_start = func_node.start_byte();
                                let expr_end = actual_node.end_byte();
                                let expr_text = &source[expr_start..expr_end];

                                return self.recover_malformed_expression(
                                    expr_text, source, block, param_map, state_vars, local_vars,
                                );
                            }

                            let mut args = Vec::new();
                            if let Some(args_node) = actual_node.child_by_field_name("arguments") {
                                let mut cursor = args_node.walk();
                                for child in args_node.children(&mut cursor) {
                                    if child.kind() != ","
                                        && child.kind() != "("
                                        && child.kind() != ")"
                                    {
                                        let arg_value = self.process_expression(
                                            child, source, block, param_map, state_vars, local_vars,
                                        )?;
                                        args.push(arg_value);
                                    }
                                }
                            }

                            Ok(block.call_internal(func_name, args))
                        }
                    }
                } else {
                    Ok(block.constant_uint(0, 256))
                }
            }
            "member_access_expression" | "member_expression" => {
                let object_node = actual_node
                    .child_by_field_name("object")
                    .or_else(|| actual_node.child(0));
                let member_node = actual_node
                    .child_by_field_name("property")
                    .or_else(|| actual_node.child_by_field_name("member"))
                    .or_else(|| actual_node.child(2));

                if let (Some(obj), Some(prop)) = (object_node, member_node) {
                    let obj_name = &source[obj.byte_range()];
                    let prop_name = &source[prop.byte_range()];

                    if obj.kind() == "call_expression" && obj_name.starts_with("type(") {
                        if prop_name == "interfaceId" {
                            return Ok(block.constant_uint(0, 32));
                        }

                        return Ok(block.constant_uint(0, 256));
                    }

                    match (obj_name, prop_name) {
                        ("msg", "sender") => Ok(block.msg_sender()),
                        ("msg", "value") => Ok(block.msg_value()),
                        ("msg", "data") => Ok(block.msg_data()),
                        ("msg", "sig") => Ok(block.msg_sig()),
                        ("block", "number") => Ok(block.block_number()),
                        ("block", "timestamp") => Ok(block.block_timestamp()),
                        ("block", "difficulty") => Ok(block.block_difficulty()),
                        ("block", "gaslimit") => Ok(block.block_gaslimit()),
                        ("block", "coinbase") => Ok(block.block_coinbase()),
                        ("block", "chainid") => Ok(block.block_chainid()),
                        ("block", "basefee") => Ok(block.block_basefee()),
                        ("tx", "origin") => Ok(block.tx_origin()),
                        ("tx", "gasprice") => Ok(block.tx_gasprice()),
                        _ => {
                            if prop_name == "length" {
                                if let Some(&(slot, ref ty)) = state_vars.get(obj_name) {
                                    if let Type::Array(_, _) = ty {
                                        let array =
                                            Value::Constant(thalir_core::values::Constant::Uint(
                                                num_bigint::BigUint::from(slot),
                                                256,
                                            ));
                                        return Ok(block.array_length(array));
                                    }
                                }

                                if let Some(array_val) = local_vars.get(obj_name) {
                                    return Ok(block.array_length(array_val.clone()));
                                }
                            }

                            if let Some(&(slot, ref ty)) = state_vars.get(obj_name) {
                                let slot_bigint = num_bigint::BigUint::from(slot);
                                Ok(block.storage_load(slot_bigint))
                            } else {
                                Ok(block.constant_uint(0, 256))
                            }
                        }
                    }
                } else {
                    Ok(block.constant_uint(0, 256))
                }
            }
            "index_access_expression" | "subscript_expression" | "array_access" => {
                let base_node = actual_node
                    .child_by_field_name("base")
                    .or_else(|| actual_node.child_by_field_name("object"))
                    .or_else(|| actual_node.child(0));
                let index_node = actual_node.child_by_field_name("index").or_else(|| {
                    let mut cursor = actual_node.walk();
                    for child in actual_node.children(&mut cursor) {
                        if child.kind() != "["
                            && child.kind() != "]"
                            && child.kind() != "identifier"
                            && child.kind() != "member_expression"
                        {
                            return Some(child);
                        }
                    }
                    None
                });

                if let (Some(base), Some(index)) = (base_node, index_node) {
                    let base_name = &source[base.byte_range()];

                    if let Some(&(slot, ref ty)) = state_vars.get(base_name) {
                        match ty {
                            Type::Mapping(_, _) => {
                                let key = self.process_expression(
                                    index, source, block, param_map, state_vars, local_vars,
                                )?;

                                Ok(block.mapping_load(
                                    Value::Constant(thalir_core::values::Constant::Uint(
                                        num_bigint::BigUint::from(slot),
                                        256,
                                    )),
                                    key,
                                ))
                            }
                            Type::Array(_, _) => {
                                let index_val = self.process_expression(
                                    index, source, block, param_map, state_vars, local_vars,
                                )?;

                                Ok(block.array_load(
                                    Value::Constant(thalir_core::values::Constant::Uint(
                                        num_bigint::BigUint::from(slot),
                                        256,
                                    )),
                                    index_val,
                                ))
                            }
                            _ => Ok(block.constant_uint(0, 256)),
                        }
                    } else {
                        Ok(block.constant_uint(0, 256))
                    }
                } else {
                    Ok(block.constant_uint(0, 256))
                }
            }
            "boolean_literal" => {
                let text = &source[actual_node.byte_range()];
                let value = text == "true";
                Ok(block.constant_bool(value))
            }
            _ => Ok(block.constant_uint(0, 256)),
        }
    }

    fn extract_parameter_type_names(&self, node: Node, source: &str) -> Vec<String> {
        let mut param_type_names = Vec::new();

        let params_node = node.child_by_field_name("parameters").or_else(|| {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "parameter_list" || child.kind() == "call_argument" {
                    return Some(child);
                }
            }
            None
        });

        if let Some(params_node) = params_node {
            let mut cursor = params_node.walk();
            for child in params_node.children(&mut cursor) {
                if child.kind() == "parameter" {
                    if let Some(type_node) = child.child_by_field_name("type") {
                        let type_text = &source[type_node.byte_range()];
                        let clean_type = type_text
                            .replace("storage", "")
                            .replace("memory", "")
                            .replace("calldata", "")
                            .trim()
                            .to_string();
                        param_type_names.push(clean_type);
                    } else {
                        param_type_names.push("uint256".to_string());
                    }
                }
            }
        } else {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "parameter" {
                    if let Some(type_node) = child.child_by_field_name("type") {
                        let type_text = &source[type_node.byte_range()];
                        let clean_type = type_text
                            .replace("storage", "")
                            .replace("memory", "")
                            .replace("calldata", "")
                            .trim()
                            .to_string();
                        param_type_names.push(clean_type);
                    } else {
                        param_type_names.push("uint256".to_string());
                    }
                }
            }
        }

        param_type_names
    }

    fn extract_parameter_types(&self, node: Node, source: &str) -> Result<Vec<Type>> {
        let mut param_types = Vec::new();

        let params_node = node.child_by_field_name("parameters").or_else(|| {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "parameter_list" || child.kind() == "call_argument" {
                    return Some(child);
                }
            }
            None
        });

        if let Some(params_node) = params_node {
            let mut cursor = params_node.walk();
            for child in params_node.children(&mut cursor) {
                if child.kind() == "parameter" {
                    let ty = if let Some(type_node) = child.child_by_field_name("type") {
                        let ctx = SimpleContext::new(source);
                        TypeResolver::resolve_type(type_node, &ctx)?
                    } else {
                        Type::Uint(256)
                    };
                    param_types.push(ty);
                }
            }
        } else {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "parameter" {
                    let ty = if let Some(type_node) = child.child_by_field_name("type") {
                        let ctx = SimpleContext::new(source);
                        TypeResolver::resolve_type(type_node, &ctx)?
                    } else {
                        Type::Uint(256)
                    };
                    param_types.push(ty);
                }
            }
        }

        Ok(param_types)
    }

    fn mangle_function_name_from_strings(base_name: &str, type_names: &[String]) -> String {
        if type_names.is_empty() {
            return base_name.to_string();
        }

        let type_suffix = type_names
            .iter()
            .map(|name| Self::sanitize_type_name_for_mangling(name))
            .collect::<Vec<_>>()
            .join("_");

        format!("{}_{}", base_name, type_suffix)
    }

    fn sanitize_type_name_for_mangling(type_name: &str) -> String {
        type_name
            .replace("[", "_arr")
            .replace("]", "")
            .replace("(", "_")
            .replace(")", "")
            .replace(",", "_")
            .replace(" ", "")
            .replace("=>", "_to_")
    }

    fn mangle_function_name(base_name: &str, param_types: &[Type]) -> String {
        if param_types.is_empty() {
            return base_name.to_string();
        }

        let type_suffix = param_types
            .iter()
            .map(|t| Self::type_to_mangle_string(t))
            .collect::<Vec<_>>()
            .join("_");

        format!("{}_{}", base_name, type_suffix)
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

    fn recover_malformed_expression(
        &mut self,
        expr_text: &str,
        source: &str,
        block: &mut BlockBuilder,
        param_map: &HashMap<String, u32>,
        state_vars: &HashMap<String, (u32, Type)>,
        local_vars: &HashMap<String, Value>,
    ) -> Result<Value> {
        let parts: Vec<&str> = expr_text.split("||").collect();

        let mut result_value: Option<Value> = None;

        for part in parts {
            let part = part.trim();

            let part_value = if part.contains("==") {
                self.parse_equality_expression(
                    part, source, block, param_map, state_vars, local_vars,
                )?
            } else if part.contains("super.") {
                self.parse_super_call_from_text(
                    part, source, block, param_map, state_vars, local_vars,
                )?
            } else if part.contains("(") {
                if let Some(paren_pos) = part.find('(') {
                    let var_name = part[..paren_pos].trim();
                    if let Some(value) = local_vars.get(var_name) {
                        value.clone()
                    } else if let Some(value) = param_map
                        .get(var_name)
                        .map(|&id| Value::Param(thalir_core::values::ParamId(id)))
                    {
                        value
                    } else {
                        block.constant_bool(false)
                    }
                } else {
                    block.constant_bool(false)
                }
            } else {
                if let Some(value) = local_vars.get(part) {
                    value.clone()
                } else if let Some(value) = param_map
                    .get(part)
                    .map(|&id| Value::Param(thalir_core::values::ParamId(id)))
                {
                    value
                } else {
                    block.constant_bool(false)
                }
            };

            result_value = if let Some(prev) = result_value {
                let true_val = block.constant_bool(true);
                Some(block.select(prev, true_val, part_value))
            } else {
                Some(part_value)
            };
        }

        Ok(result_value.unwrap_or_else(|| block.constant_bool(false)))
    }

    fn parse_equality_expression(
        &mut self,
        expr: &str,
        _source: &str,
        block: &mut BlockBuilder,
        param_map: &HashMap<String, u32>,
        _state_vars: &HashMap<String, (u32, Type)>,
        local_vars: &HashMap<String, Value>,
    ) -> Result<Value> {
        if let Some(eq_pos) = expr.find("==") {
            let left = expr[..eq_pos].trim();
            let right = expr[eq_pos + 2..].trim();

            let left_val = if let Some(value) = local_vars.get(left) {
                value.clone()
            } else if let Some(value) = param_map
                .get(left)
                .map(|&id| Value::Param(thalir_core::values::ParamId(id)))
            {
                value
            } else {
                return Ok(block.constant_bool(false));
            };

            // Not yet implemented: ERC-165 interfaceId computation requires type system support
            let right_val = if right.starts_with("type(") && right.contains(").interfaceId") {
                block.constant_uint(0, 32)
            } else {
                if let Some(value) = local_vars.get(right) {
                    value.clone()
                } else if let Some(value) = param_map
                    .get(right)
                    .map(|&id| Value::Param(thalir_core::values::ParamId(id)))
                {
                    value
                } else {
                    block.constant_uint(0, 256)
                }
            };

            Ok(block.eq(left_val, right_val))
        } else {
            Ok(block.constant_bool(false))
        }
    }

    fn parse_super_call_from_text(
        &mut self,
        expr: &str,
        _source: &str,
        block: &mut BlockBuilder,
        param_map: &HashMap<String, u32>,
        _state_vars: &HashMap<String, (u32, Type)>,
        local_vars: &HashMap<String, Value>,
    ) -> Result<Value> {
        if let Some(super_pos) = expr.find("super.") {
            let after_super = &expr[super_pos + 6..];
            if let Some(paren_pos) = after_super.find('(') {
                let method_name = &after_super[..paren_pos];

                let args_start = super_pos + 6 + paren_pos + 1;
                let args_end = expr.rfind(')').unwrap_or(expr.len());
                let args_text = &expr[args_start..args_end];

                let mut args = Vec::new();
                for arg in args_text.split(',') {
                    let arg = arg.trim();
                    if !arg.is_empty() {
                        if let Some(value) = local_vars.get(arg) {
                            args.push(value.clone());
                        } else if let Some(value) = param_map
                            .get(arg)
                            .map(|&id| Value::Param(thalir_core::values::ParamId(id)))
                        {
                            args.push(value);
                        }
                    }
                }

                let parent_func = format!("_super_{}", method_name);
                return Ok(block.call_internal(&parent_func, args));
            }
        }

        Ok(block.constant_bool(false))
    }

    fn compute_function_selector(signature: &str) -> u32 {
        use tiny_keccak::{Hasher, Keccak};

        let mut keccak = Keccak::v256();
        let mut output = [0u8; 32];
        keccak.update(signature.as_bytes());
        keccak.finalize(&mut output);

        u32::from_be_bytes([output[0], output[1], output[2], output[3]])
    }

    fn compute_interface_id(function_signatures: &[&str]) -> u32 {
        function_signatures
            .iter()
            .map(|sig| Self::compute_function_selector(sig))
            .fold(0u32, |acc, selector| acc ^ selector)
    }
}

impl IRTransformer for StructuralTransformer {
    fn name(&self) -> &str {
        "StructuralTransformer"
    }

    fn transform(&mut self, builder: &mut IRBuilder, ast: &Node, source: &str) -> Result<()> {
        if ast.kind() == "source_file" {
            self.process_source_file(*ast, source, builder)?;
        }
        Ok(())
    }
}
