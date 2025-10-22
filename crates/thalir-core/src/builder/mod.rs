/*! Fluent API for constructing IR programmatically.
 *
 * Hand-wiring IR structures is tedious and error-prone. These builders handle SSA value numbering,
 * block parameters, and instruction sequencing automatically—letting you focus on program logic
 * rather than bookkeeping.
 */

pub mod block_builder;
pub mod contract_builder;
pub mod cursor;
pub mod function_builder;
pub mod function_builder_cursor;
pub mod inst_builder;
pub mod ir_context;
pub mod ir_registry;

pub use block_builder::BlockBuilder;
pub use contract_builder::ContractBuilder;
pub use cursor::{CursorPosition, FunctionCursor};
pub use function_builder::FunctionBuilder;
pub use function_builder_cursor::{FunctionBuilderCursor, FunctionInstBuilder};
pub use inst_builder::{InstBuilder, InstBuilderBase, InstBuilderExt};
pub use ir_context::{IRContext, SSATracker, SourceMapping};
pub use ir_registry::{IRRegistry, RegistryStats};

use crate::{IrError, Result};

pub struct IRBuilder {
    context: IRContext,
    registry: IRRegistry,
}

impl IRBuilder {
    pub fn new() -> Self {
        Self {
            context: IRContext::new(),
            registry: IRRegistry::new(),
        }
    }

    pub fn contract(&mut self, name: &str) -> ContractBuilder<'_> {
        self.context.set_current_contract(name.to_string());
        ContractBuilder::new(name.to_string(), &mut self.context, &mut self.registry)
    }

    pub fn registry(&self) -> &IRRegistry {
        &self.registry
    }

    pub fn registry_mut(&mut self) -> &mut IRRegistry {
        &mut self.registry
    }

    pub fn context(&self) -> &IRContext {
        &self.context
    }

    pub fn context_mut(&mut self) -> &mut IRContext {
        &mut self.context
    }

    pub fn validate(&self) -> Result<()> {
        self.registry.validate()?;

        if self.context.has_errors() {
            return Err(IrError::BuilderError(format!(
                "IR building errors: {:?}",
                self.context.errors()
            )));
        }

        Ok(())
    }

    pub fn clear(&mut self) {
        self.context.clear();
        self.registry.clear();
    }

    pub fn stats(&self) -> RegistryStats {
        self.registry.stats()
    }
}

impl Default for IRBuilder {
    fn default() -> Self {
        Self::new()
    }
}
