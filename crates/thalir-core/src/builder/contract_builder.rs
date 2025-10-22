use super::{FunctionBuilder, IRContext, IRRegistry};
use crate::{
    contract::{Contract, EventDefinition, EventId},
    types::Type,
    Result,
};

pub struct ContractBuilder<'a> {
    contract_name: String,
    context: &'a mut IRContext,
    registry: &'a mut IRRegistry,
}

impl<'a> ContractBuilder<'a> {
    pub fn new(name: String, context: &'a mut IRContext, registry: &'a mut IRRegistry) -> Self {
        let contract = Contract::new(name.clone());
        registry.add_contract(contract).unwrap();
        Self {
            contract_name: name,
            context,
            registry,
        }
    }

    pub fn function(&mut self, name: &str) -> FunctionBuilder<'_> {
        let qualified_name = format!("{}::{}", self.contract_name, name);
        self.context.set_current_function(qualified_name.clone());

        FunctionBuilder::new(
            self.contract_name.clone(),
            name.to_string(),
            self.context,
            self.registry,
        )
    }

    pub fn state_variable(&mut self, name: &str, ty: Type, slot: u32) -> &mut Self {
        if let Some(contract) = self.registry.get_contract_mut(&self.contract_name) {
            contract
                .storage_layout
                .add_variable(name.to_string(), ty, slot);
        }
        self
    }

    pub fn event(&mut self, name: &str) -> EventBuilder {
        let event_id = EventId(self.context.next_id() as u32);
        let event_builder = EventBuilder {
            event: EventDefinition {
                id: event_id,
                name: name.to_string(),
                parameters: Vec::new(),
                anonymous: false,
            },
        };
        event_builder
    }

    pub fn add_event(&mut self, event: EventDefinition) {
        if let Some(contract) = self.registry.get_contract_mut(&self.contract_name) {
            contract.events.push(event);
        }
    }

    pub fn metadata(&mut self, version: &str) -> &mut Self {
        if let Some(contract) = self.registry.get_contract_mut(&self.contract_name) {
            contract.metadata.version = version.to_string();
        }
        self
    }

    pub fn build(self) -> Result<Contract> {
        self.registry
            .get_contract(&self.contract_name)
            .ok_or_else(|| crate::IrError::ContractNotFound(self.contract_name.clone()))
            .map(|c| c.clone())
    }
}

pub struct EventBuilder {
    event: EventDefinition,
}

impl EventBuilder {
    pub fn indexed(mut self, name: &str, ty: Type) -> Self {
        self.event.parameters.push(crate::contract::EventParameter {
            name: name.to_string(),
            param_type: ty,
            indexed: true,
        });
        self
    }

    pub fn data(mut self, name: &str, ty: Type) -> Self {
        self.event.parameters.push(crate::contract::EventParameter {
            name: name.to_string(),
            param_type: ty,
            indexed: false,
        });
        self
    }

    pub fn anonymous(mut self) -> Self {
        self.event.anonymous = true;
        self
    }

    pub fn build(self) -> EventDefinition {
        self.event
    }
}
