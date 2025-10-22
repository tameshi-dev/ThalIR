use crate::block::BlockId;
use crate::values::Value;
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SecurityMetadata {
    pub external_calls: Vec<ExternalCallSite>,
    pub state_mutations: Vec<StateMutation>,
    pub reentrancy_guards: Vec<ReentrancyGuard>,
    pub access_controls: Vec<AccessControl>,
    pub checked_operations: Vec<CheckedOp>,
    pub vulnerability_patterns: Vec<VulnerabilityPattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalCallSite {
    pub location: InstructionLocation,
    pub target: CallTarget,
    pub value_transfer: bool,
    pub state_changes_before: Vec<StateChange>,
    pub state_changes_after: Vec<StateChange>,
    pub can_reenter: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InstructionLocation {
    pub block: BlockId,
    pub index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CallTarget {
    Known(String),
    Unknown,
    UserControlled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateChange {
    pub storage_key: StorageKeyInfo,
    pub change_type: StateChangeType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageKeyInfo {
    Slot(BigUint),
    Mapping { base: BigUint, key: String },
    Dynamic(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StateChangeType {
    Write,
    Delete,
    Increment,
    Decrement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateMutation {
    pub location: InstructionLocation,
    pub mutated_var: String,
    pub mutation_type: MutationType,
    pub depends_on_input: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MutationType {
    Assignment,
    Arithmetic,
    ArrayPush,
    ArrayPop,
    MappingUpdate,
    Deletion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReentrancyGuard {
    pub guard_type: ReentrancyGuardType,
    pub protected_blocks: Vec<BlockId>,
    pub guard_variable: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReentrancyGuardType {
    Mutex,
    CheckEffectsInteraction,
    NonReentrantModifier,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControl {
    pub control_type: AccessControlType,
    pub location: InstructionLocation,
    pub condition: AccessCondition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessControlType {
    OwnerOnly,
    RoleBased(String),
    Whitelist,
    Pausable,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessCondition {
    RequireStatement,
    Modifier(String),
    IfStatement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckedOp {
    pub location: InstructionLocation,
    pub operation: CheckedOperation,
    pub overflow_behavior: OverflowBehavior,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CheckedOperation {
    Addition,
    Subtraction,
    Multiplication,
    Division,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OverflowBehavior {
    Revert,
    Wrap,
    Saturate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VulnerabilityPattern {
    Reentrancy {
        call_site: InstructionLocation,
        state_changes: Vec<StateChange>,
    },
    IntegerOverflow {
        location: InstructionLocation,
        operation: String,
    },
    UnprotectedTransfer {
        location: InstructionLocation,
        recipient: String,
    },
    AccessControl {
        function: String,
        missing_check: String,
    },
    TimestampDependence {
        location: InstructionLocation,
        usage: String,
    },
    UncheckedReturn {
        call_location: InstructionLocation,
        ignored_value: String,
    },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OptimizationHints {
    pub pure_functions: HashSet<String>,
    pub view_functions: HashSet<String>,
    pub constant_expressions: Vec<ConstantExpression>,
    pub loop_bounds: HashMap<BlockId, LoopBound>,
    pub invariants: Vec<Invariant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstantExpression {
    pub location: InstructionLocation,
    pub value: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopBound {
    pub min_iterations: Option<u64>,
    pub max_iterations: Option<u64>,
    pub bound_expression: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invariant {
    pub condition: String,
    pub scope: InvariantScope,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InvariantScope {
    Function(String),
    Loop(BlockId),
    Contract,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnalysisMetadata {
    pub data_dependencies: DataDependencies,
    pub control_dependencies: ControlDependencies,
    pub taint_analysis: TaintAnalysis,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DataDependencies {
    pub def_use_chains: HashMap<Value, HashSet<InstructionLocation>>,
    pub use_def_chains: HashMap<InstructionLocation, HashSet<Value>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ControlDependencies {
    pub dominators: HashMap<BlockId, HashSet<BlockId>>,
    pub post_dominators: HashMap<BlockId, HashSet<BlockId>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaintAnalysis {
    pub tainted_values: HashSet<Value>,
    pub taint_sources: Vec<TaintSource>,
    pub taint_sinks: Vec<TaintSink>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaintSource {
    pub source_type: TaintSourceType,
    pub location: InstructionLocation,
    pub value: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaintSourceType {
    UserInput,
    ExternalCall,
    Storage,
    Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaintSink {
    pub sink_type: TaintSinkType,
    pub location: InstructionLocation,
    pub value: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaintSinkType {
    StateChange,
    ExternalCall,
    Transfer,
    Selfdestruct,
}

impl SecurityMetadata {
    pub fn has_high_risk_patterns(&self) -> bool {
        for call in &self.external_calls {
            if call.value_transfer && !call.state_changes_after.is_empty() {
                return true;
            }
        }

        for mutation in &self.state_mutations {
            if mutation.depends_on_input && self.access_controls.is_empty() {
                return true;
            }
        }

        false
    }

    pub fn state_modifying_functions(&self) -> HashSet<String> {
        let functions = HashSet::new();

        for _mutation in &self.state_mutations {}

        functions
    }
}
