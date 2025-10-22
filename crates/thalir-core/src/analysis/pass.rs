use crate::{contract::Contract, function::Function};
use anyhow::Result;
use std::any::Any;
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnalysisID {
    ControlFlow,
    Dominator,
    PostDominator,
    LoopAnalysis,
    AliasAnalysis,
    DefUse,
    Liveness,
    ExternalCalls,
    StateAccess,
    TaintAnalysis,
    Obfuscation,
    Custom(&'static str),
}

pub trait Pass: Send + Sync {
    fn name(&self) -> &'static str;

    fn description(&self) -> &'static str {
        "No description provided"
    }

    fn run_on_contract(&mut self, contract: &mut Contract, manager: &mut PassManager)
        -> Result<()>;

    fn required_analyses(&self) -> Vec<AnalysisID> {
        Vec::new()
    }

    fn preserved_analyses(&self) -> Vec<AnalysisID> {
        Vec::new()
    }

    fn modifies_ir(&self) -> bool {
        false
    }

    fn as_any(&self) -> &dyn std::any::Any;

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

pub trait AnalysisPass: Pass {
    type Result: Clone + Any + Send + Sync;

    fn analyze(&mut self, function: &Function) -> Result<Self::Result>;

    fn analyze_contract(&mut self, contract: &Contract) -> Result<HashMap<String, Self::Result>> {
        let mut results = HashMap::new();
        for (name, function) in &contract.functions {
            results.insert(name.clone(), self.analyze(function)?);
        }
        Ok(results)
    }

    fn analysis_id(&self) -> AnalysisID;
}

#[derive(Debug, Clone)]
pub struct PassStatistics {
    pub name: String,
    pub duration: Duration,
    pub memory_usage: Option<usize>,
}

pub struct PassManager {
    passes: Vec<Box<dyn Pass>>,
    analysis_cache: HashMap<(AnalysisID, String), Box<dyn Any + Send + Sync>>,
    statistics: Vec<PassStatistics>,
    collect_stats: bool,
    valid_analyses: HashMap<String, Vec<AnalysisID>>,
}

impl PassManager {
    pub fn new() -> Self {
        Self {
            passes: Vec::new(),
            analysis_cache: HashMap::new(),
            statistics: Vec::new(),
            collect_stats: false,
            valid_analyses: HashMap::new(),
        }
    }

    pub fn enable_statistics(&mut self) {
        self.collect_stats = true;
    }

    pub fn register_pass<P: Pass + 'static>(&mut self, pass: P) {
        self.passes.push(Box::new(pass));
    }

    pub fn run_all(&mut self, contract: &mut Contract) -> Result<()> {
        for i in 0..self.passes.len() {
            let mut pass = std::mem::replace(&mut self.passes[i], Box::new(DummyPass));

            let start = if self.collect_stats {
                Some(Instant::now())
            } else {
                None
            };

            for required in pass.required_analyses() {
                if !self.is_analysis_valid(&contract.name, required) {
                    self.compute_analysis(contract, required)?;
                }
            }

            pass.run_on_contract(contract, self)?;

            if pass.modifies_ir() {
                self.invalidate_analyses(&contract.name, &pass.preserved_analyses());
            }

            if let Some(start) = start {
                self.statistics.push(PassStatistics {
                    name: pass.name().to_string(),
                    duration: start.elapsed(),
                    memory_usage: None,
                });
            }

            self.passes[i] = pass;
        }

        Ok(())
    }

    pub fn get_analysis<A: AnalysisPass + 'static>(
        &mut self,
        contract: &Contract,
    ) -> Result<&HashMap<String, A::Result>> {
        let analysis_id = self.get_analysis_id::<A>();
        let key = (analysis_id, contract.name.clone());

        if !self.analysis_cache.contains_key(&key) {
            self.compute_analysis(contract, analysis_id)?;
        }

        self.analysis_cache
            .get(&key)
            .and_then(|boxed| boxed.downcast_ref::<HashMap<String, A::Result>>())
            .ok_or_else(|| anyhow::anyhow!("Failed to get analysis result"))
    }

    pub fn get_function_analysis<A: AnalysisPass + 'static>(
        &mut self,
        contract: &Contract,
        function_name: &str,
    ) -> Result<A::Result> {
        let results = self.get_analysis::<A>(contract)?;
        if let Some(func_results) = results.get(function_name) {
            Ok(func_results.clone())
        } else {
            Err(anyhow::anyhow!(
                "Analysis not found for function: {}",
                function_name
            ))
        }
    }

    fn compute_analysis(&mut self, contract: &Contract, analysis_id: AnalysisID) -> Result<()> {
        let pass_idx = self.passes.iter().position(|p| {
            if let Some(analysis_pass) = p
                .as_any()
                .downcast_ref::<super::passes::DominatorAnalysisPass>()
            {
                analysis_pass.analysis_id() == analysis_id
            } else if let Some(analysis_pass) = p
                .as_any()
                .downcast_ref::<super::passes::ControlFlowAnalysisPass>()
            {
                analysis_pass.analysis_id() == analysis_id
            } else if let Some(analysis_pass) = p
                .as_any()
                .downcast_ref::<super::passes::DefUseAnalysisPass>()
            {
                analysis_pass.analysis_id() == analysis_id
            } else if let Some(analysis_pass) = p
                .as_any()
                .downcast_ref::<super::passes::AliasAnalysisPass>()
            {
                analysis_pass.analysis_id() == analysis_id
            } else {
                false
            }
        });

        if let Some(idx) = pass_idx {
            let mut pass = self.passes.remove(idx);

            let results: Box<dyn Any + Send + Sync> = if let Some(analysis_pass) = pass
                .as_any_mut()
                .downcast_mut::<super::passes::DominatorAnalysisPass>()
            {
                let mut typed_results = HashMap::new();
                for (func_name, function) in &contract.functions {
                    typed_results.insert(func_name.clone(), analysis_pass.analyze(function)?);
                }
                Box::new(typed_results)
            } else if let Some(analysis_pass) =
                pass.as_any_mut()
                    .downcast_mut::<super::passes::ControlFlowAnalysisPass>()
            {
                let mut typed_results = HashMap::new();
                for (func_name, function) in &contract.functions {
                    typed_results.insert(func_name.clone(), analysis_pass.analyze(function)?);
                }
                Box::new(typed_results)
            } else if let Some(analysis_pass) = pass
                .as_any_mut()
                .downcast_mut::<super::passes::DefUseAnalysisPass>()
            {
                let mut typed_results = HashMap::new();
                for (func_name, function) in &contract.functions {
                    typed_results.insert(func_name.clone(), analysis_pass.analyze(function)?);
                }
                Box::new(typed_results)
            } else if let Some(analysis_pass) = pass
                .as_any_mut()
                .downcast_mut::<super::passes::AliasAnalysisPass>()
            {
                let mut typed_results = HashMap::new();
                for (func_name, function) in &contract.functions {
                    typed_results.insert(func_name.clone(), analysis_pass.analyze(function)?);
                }
                Box::new(typed_results)
            } else {
                return Err(anyhow::anyhow!("Unknown analysis pass type"));
            };

            let key = (analysis_id, contract.name.clone());
            self.analysis_cache.insert(key, results);

            self.valid_analyses
                .entry(contract.name.clone())
                .or_insert_with(Vec::new)
                .push(analysis_id);

            self.passes.insert(idx, pass);
        } else {
            return Err(anyhow::anyhow!(
                "Analysis pass not found for {:?}",
                analysis_id
            ));
        }

        Ok(())
    }

    pub fn cache_analysis<T: Any + Send + Sync>(
        &mut self,
        analysis_id: AnalysisID,
        contract_name: &str,
        result: T,
    ) {
        let key = (analysis_id, contract_name.to_string());
        self.analysis_cache.insert(key, Box::new(result));

        self.valid_analyses
            .entry(contract_name.to_string())
            .or_insert_with(Vec::new)
            .push(analysis_id);
    }

    fn is_analysis_valid(&self, contract_name: &str, analysis_id: AnalysisID) -> bool {
        self.valid_analyses
            .get(contract_name)
            .map(|valid| valid.contains(&analysis_id))
            .unwrap_or(false)
    }

    fn invalidate_analyses(&mut self, contract_name: &str, preserved: &[AnalysisID]) {
        if let Some(valid) = self.valid_analyses.get_mut(contract_name) {
            valid.retain(|id| preserved.contains(id));
        }

        self.analysis_cache
            .retain(|(id, name), _| name != contract_name || preserved.contains(id));
    }

    fn get_analysis_id<A: AnalysisPass + 'static>(&self) -> AnalysisID {
        if std::any::TypeId::of::<A>()
            == std::any::TypeId::of::<super::passes::DominatorAnalysisPass>()
        {
            AnalysisID::Dominator
        } else if std::any::TypeId::of::<A>()
            == std::any::TypeId::of::<super::passes::ControlFlowAnalysisPass>()
        {
            AnalysisID::ControlFlow
        } else if std::any::TypeId::of::<A>()
            == std::any::TypeId::of::<super::passes::DefUseAnalysisPass>()
        {
            AnalysisID::DefUse
        } else if std::any::TypeId::of::<A>()
            == std::any::TypeId::of::<super::passes::AliasAnalysisPass>()
        {
            AnalysisID::AliasAnalysis
        } else {
            AnalysisID::Custom("unknown")
        }
    }

    pub fn statistics(&self) -> &[PassStatistics] {
        &self.statistics
    }

    pub fn clear_cache(&mut self) {
        self.analysis_cache.clear();
        self.valid_analyses.clear();
    }

    pub fn get_pass_mut<P: Pass + 'static>(&mut self) -> Option<&mut P> {
        self.passes
            .iter_mut()
            .find_map(|p| p.as_any_mut().downcast_mut::<P>())
    }

    pub fn get_pass<P: Pass + 'static>(&self) -> Option<&P> {
        self.passes
            .iter()
            .find_map(|p| p.as_any().downcast_ref::<P>())
    }
}

impl Default for PassManager {
    fn default() -> Self {
        Self::new()
    }
}

struct DummyPass;

impl Pass for DummyPass {
    fn name(&self) -> &'static str {
        "dummy"
    }

    fn run_on_contract(
        &mut self,
        _contract: &mut Contract,
        _manager: &mut PassManager,
    ) -> Result<()> {
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

pub struct PassTimer {
    name: String,
    start: Instant,
    manager: Option<*mut PassManager>,
}

impl PassTimer {
    pub fn start(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            start: Instant::now(),
            manager: None,
        }
    }

    pub fn start_with_manager(name: impl Into<String>, manager: &mut PassManager) -> Self {
        Self {
            name: name.into(),
            start: Instant::now(),
            manager: Some(manager as *mut _),
        }
    }
}

impl Drop for PassTimer {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        if let Some(manager_ptr) = self.manager {
            unsafe {
                if let Some(manager) = manager_ptr.as_mut() {
                    manager.statistics.push(PassStatistics {
                        name: self.name.clone(),
                        duration,
                        memory_usage: None,
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pass_registration() {
        let mut manager = PassManager::new();

        struct TestPass;
        impl Pass for TestPass {
            fn name(&self) -> &'static str {
                "test"
            }
            fn run_on_contract(&mut self, _: &mut Contract, _: &mut PassManager) -> Result<()> {
                Ok(())
            }
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }
        }

        manager.register_pass(TestPass);
        assert_eq!(manager.passes.len(), 1);
    }

    #[test]
    fn test_analysis_caching() {
        let mut manager = PassManager::new();

        manager.cache_analysis(AnalysisID::ControlFlow, "test_contract", vec![1, 2, 3]);
        assert!(manager.is_analysis_valid("test_contract", AnalysisID::ControlFlow));

        manager.invalidate_analyses("test_contract", &[]);
        assert!(!manager.is_analysis_valid("test_contract", AnalysisID::ControlFlow));
    }
}
