use super::{
    AliasAnalysis, AnalysisID, AnalysisPass, ControlFlowGraph, DefUseChains, DominatorTree, Pass,
    PassManager,
};
use crate::{contract::Contract, function::Function};
use anyhow::Result;

pub struct DominatorAnalysisPass;

impl Pass for DominatorAnalysisPass {
    fn name(&self) -> &'static str {
        "dominator-analysis"
    }

    fn run_on_contract(
        &mut self,
        _contract: &mut Contract,
        _manager: &mut PassManager,
    ) -> Result<()> {
        Ok(())
    }

    fn required_analyses(&self) -> Vec<AnalysisID> {
        vec![]
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl AnalysisPass for DominatorAnalysisPass {
    type Result = DominatorTree;

    fn analyze(&mut self, function: &Function) -> Result<Self::Result> {
        Ok(DominatorTree::build(function))
    }

    fn analysis_id(&self) -> AnalysisID {
        AnalysisID::Dominator
    }
}

pub struct ControlFlowAnalysisPass;

impl Pass for ControlFlowAnalysisPass {
    fn name(&self) -> &'static str {
        "control-flow-analysis"
    }

    fn run_on_contract(
        &mut self,
        _contract: &mut Contract,
        _manager: &mut PassManager,
    ) -> Result<()> {
        Ok(())
    }

    fn required_analyses(&self) -> Vec<AnalysisID> {
        vec![]
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl AnalysisPass for ControlFlowAnalysisPass {
    type Result = ControlFlowGraph;

    fn analyze(&mut self, function: &Function) -> Result<Self::Result> {
        Ok(ControlFlowGraph::build(function))
    }

    fn analysis_id(&self) -> AnalysisID {
        AnalysisID::ControlFlow
    }
}

pub struct DefUseAnalysisPass;

impl Pass for DefUseAnalysisPass {
    fn name(&self) -> &'static str {
        "def-use-analysis"
    }

    fn run_on_contract(
        &mut self,
        _contract: &mut Contract,
        _manager: &mut PassManager,
    ) -> Result<()> {
        Ok(())
    }

    fn required_analyses(&self) -> Vec<AnalysisID> {
        vec![]
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl AnalysisPass for DefUseAnalysisPass {
    type Result = DefUseChains;

    fn analyze(&mut self, function: &Function) -> Result<Self::Result> {
        Ok(DefUseChains::build(function))
    }

    fn analysis_id(&self) -> AnalysisID {
        AnalysisID::DefUse
    }
}

pub struct AliasAnalysisPass;

impl Pass for AliasAnalysisPass {
    fn name(&self) -> &'static str {
        "alias-analysis"
    }

    fn run_on_contract(
        &mut self,
        _contract: &mut Contract,
        _manager: &mut PassManager,
    ) -> Result<()> {
        Ok(())
    }

    fn required_analyses(&self) -> Vec<AnalysisID> {
        vec![]
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl AnalysisPass for AliasAnalysisPass {
    type Result = AliasAnalysis;

    fn analyze(&mut self, function: &Function) -> Result<Self::Result> {
        Ok(AliasAnalysis::build(function))
    }

    fn analysis_id(&self) -> AnalysisID {
        AnalysisID::AliasAnalysis
    }
}

pub fn register_standard_analyses(manager: &mut PassManager) {
    manager.register_pass(ControlFlowAnalysisPass);
    manager.register_pass(DominatorAnalysisPass);
    manager.register_pass(DefUseAnalysisPass);
    manager.register_pass(AliasAnalysisPass);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::IRBuilder;

    #[test]
    fn test_analysis_passes() {
        let mut builder = IRBuilder::new();
        let mut contract_builder = builder.contract("TestContract");

        let mut func_builder = contract_builder.function("test");
        let mut entry_builder = func_builder.entry_block();
        entry_builder.return_void().unwrap();
        let function = func_builder.build().unwrap();

        let contract = contract_builder.build().unwrap();

        let mut manager = PassManager::new();
        register_standard_analyses(&mut manager);

        let dom_tree_result =
            manager.get_function_analysis::<DominatorAnalysisPass>(&contract, "test");
        assert!(dom_tree_result.is_ok());

        let cfg_result =
            manager.get_function_analysis::<ControlFlowAnalysisPass>(&contract, "test");
        assert!(cfg_result.is_ok());
    }
}
