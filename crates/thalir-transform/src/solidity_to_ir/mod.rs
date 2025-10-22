/*! Convert Solidity AST to ThalIR.
 *
 * Tree-sitter gives you a Solidity AST, but that's just syntax. This pipeline walks the AST and builds
 * IR that captures semantics: storage layout, overflow behavior, control flow, and external call patterns.
 * Where Solidity's meaning becomes explicit.
 */

mod context;
mod control_flow_builder;
mod control_flow_cursor;
mod errors;
mod expression_transformer;
mod structural_transformer;
mod structural_transformer_cursor;
mod type_resolver;

use anyhow::{anyhow, Result};
use thalir_core::{builder::IRBuilder, Contract};
use tree_sitter::{Node, Tree};

pub use errors::TransformError;

pub trait IRTransformer {
    fn name(&self) -> &str;

    fn transform(&mut self, builder: &mut IRBuilder, ast: &Node, source: &str) -> Result<()>;

    fn check_prerequisites(&self, _builder: &IRBuilder) -> Result<()> {
        Ok(())
    }
}

pub struct TransformationPipeline {
    source: String,
    ast: Option<Tree>,
    transformers: Vec<Box<dyn IRTransformer>>,
}

impl TransformationPipeline {
    pub fn default(source: &str) -> Self {
        Self {
            source: source.to_string(),
            ast: None,
            transformers: vec![Box::new(
                structural_transformer::StructuralTransformer::new(),
            )],
        }
    }

    pub fn with_filename(source: &str, filename: String) -> Self {
        Self {
            source: source.to_string(),
            ast: None,
            transformers: vec![Box::new(
                structural_transformer::StructuralTransformer::with_filename(filename),
            )],
        }
    }

    pub fn new(source: &str) -> Self {
        Self {
            source: source.to_string(),
            ast: None,
            transformers: vec![],
        }
    }

    pub fn with_transformer(mut self, transformer: Box<dyn IRTransformer>) -> Self {
        self.transformers.push(transformer);
        self
    }

    pub fn transform(mut self) -> Result<Vec<Contract>> {
        if self.ast.is_none() {
            let mut parser = tree_sitter::Parser::new();
            let language = tree_sitter_solidity::LANGUAGE.into();
            parser
                .set_language(&language)
                .map_err(|e| anyhow!("Failed to set language: {}", e))?;

            let tree = parser
                .parse(&self.source, None)
                .ok_or_else(|| anyhow!("Failed to parse source"))?;

            if tree.root_node().has_error() {
                return Err(anyhow!("Failed to parse source: syntax errors detected"));
            }

            self.ast = Some(tree);
        }

        let ast = self
            .ast
            .as_ref()
            .ok_or_else(|| anyhow!("AST not initialized - call parse() first"))?;
        let root_node = ast.root_node();

        let mut builder = IRBuilder::new();

        for transformer in &mut self.transformers {
            transformer.check_prerequisites(&builder)?;
            transformer.transform(&mut builder, &root_node, &self.source)?;
        }

        builder.validate()?;

        let registry = builder.registry();
        let mut contracts = Vec::new();

        for (_name, contract) in registry.contracts() {
            contracts.push(contract.clone());
        }

        Ok(contracts)
    }
}

pub fn transform_solidity_to_ir(source: &str) -> Result<Vec<Contract>> {
    transform_solidity_to_ir_with_filename(source, None)
}

pub fn transform_solidity_to_ir_with_filename(
    source: &str,
    filename: Option<&str>,
) -> Result<Vec<Contract>> {
    let mut contracts = if let Some(file) = filename {
        TransformationPipeline::with_filename(source, file.to_string()).transform()?
    } else {
        TransformationPipeline::default(source).transform()?
    };

    if let Some(file) = filename {
        for contract in &mut contracts {
            contract.metadata.source_file = Some(file.to_string());
            contract.metadata.source_code = Some(source.to_string());
        }
    }

    Ok(contracts)
}

pub fn transform_solidity_to_ir_with_cfg(source: &str) -> Result<Vec<Contract>> {
    let mut parser = tree_sitter::Parser::new();
    let language = tree_sitter_solidity::LANGUAGE.into();
    parser
        .set_language(&language)
        .map_err(|e| anyhow!("Failed to set language: {}", e))?;

    let tree = parser
        .parse(source, None)
        .ok_or_else(|| anyhow!("Failed to parse source"))?;

    let root_node = tree.root_node();

    let mut context = thalir_core::builder::IRContext::new();
    let mut registry = thalir_core::builder::IRRegistry::new();

    let mut transformer = structural_transformer_cursor::StructuralTransformerCursor::new();
    transformer.transform_with_context(&root_node, source, &mut context, &mut registry)?;

    registry.validate()?;

    let mut contracts = Vec::new();

    for (_name, contract) in registry.contracts() {
        contracts.push(contract.clone());
    }

    Ok(contracts)
}

#[cfg(test)]
mod tests;
