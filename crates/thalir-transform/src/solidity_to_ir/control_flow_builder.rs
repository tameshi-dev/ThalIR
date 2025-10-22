use super::{
    context::TransformationContext, errors::TransformError,
    expression_transformer::ExpressionTransformer,
};
use anyhow::Result;
use thalir_core::builder::BlockBuilder;
use tree_sitter::Node;

pub struct ControlFlowBuilder;

impl ControlFlowBuilder {
    pub fn new() -> Self {
        Self
    }

    pub fn process_if_statement(
        &mut self,
        node: Node,
        ctx: &mut TransformationContext,
        _current_block: &mut BlockBuilder,
        expr_transformer: &mut ExpressionTransformer,
    ) -> Result<()> {
        let condition_node = node
            .child_by_field_name("condition")
            .ok_or_else(|| anyhow::anyhow!("Missing condition in if statement"))?;

        ctx.add_error(TransformError::UnsupportedFeature(
            "If statements require advanced control flow handling".to_string(),
        ));

        Ok(())
    }

    pub fn process_for_statement(
        &mut self,
        node: Node,
        ctx: &mut TransformationContext,
        _current_block: &mut BlockBuilder,
        expr_transformer: &mut ExpressionTransformer,
    ) -> Result<()> {
        ctx.add_error(TransformError::UnsupportedFeature(
            "For loops require advanced control flow handling".to_string(),
        ));

        Ok(())
    }

    pub fn process_while_statement(
        &mut self,
        node: Node,
        ctx: &mut TransformationContext,
        _current_block: &mut BlockBuilder,
        expr_transformer: &mut ExpressionTransformer,
    ) -> Result<()> {
        ctx.add_error(TransformError::UnsupportedFeature(
            "While loops require advanced control flow handling".to_string(),
        ));

        Ok(())
    }

    pub fn process_do_while_statement(
        &mut self,
        node: Node,
        ctx: &mut TransformationContext,
        _current_block: &mut BlockBuilder,
        expr_transformer: &mut ExpressionTransformer,
    ) -> Result<()> {
        ctx.add_error(TransformError::UnsupportedFeature(
            "Do-while loops require advanced control flow handling".to_string(),
        ));

        Ok(())
    }
}
