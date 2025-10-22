use crate::{block::BlockId, function::Function, instructions::Instruction, values::Value};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstKind {
    Call,
    DelegateCall,
    StorageStore,
    StorageLoad,
    Store,
    Load,
    Add,
    Sub,
    Mul,
    Div,
    Jump,
    Return,
    Revert,
    Any,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Inst(InstPattern),
    Value(ValuePattern),
    Block(BlockPattern),
    Sequence(Vec<Pattern>),
    Any(Vec<Pattern>),
    Not(Box<Pattern>),
    Constrained {
        pattern: Box<Pattern>,
        constraints: Vec<Constraint>,
    },

    ControlFlow(CfgPattern),

    DataFlow(DataFlowPattern),

    Wildcard,

    Capture {
        name: String,
        pattern: Box<Pattern>,
    },
}

#[derive(Debug, Clone)]
pub struct InstPattern {
    pub opcode: Option<InstKind>,
    pub args: Vec<ValuePattern>,
    pub result: Option<ValuePattern>,
    pub predicates: Vec<InstPredicate>,
}

#[derive(Debug, Clone)]
pub enum ValuePattern {
    Exact(Value),
    Constant,
    Parameter,
    Temporary,
    StateVar { name: Option<String> },
    External,
    Ref(String),
    Any,
}

#[derive(Debug, Clone)]
pub struct BlockPattern {
    pub instructions: Vec<Pattern>,
    pub terminator: Option<TerminatorPattern>,
    pub predicates: Vec<BlockPredicate>,
}

#[derive(Debug, Clone)]
pub enum CfgPattern {
    IfThenElse {
        condition: Box<Pattern>,
        then_branch: Box<Pattern>,
        else_branch: Option<Box<Pattern>>,
    },

    Loop {
        header: Box<Pattern>,
        body: Box<Pattern>,
        exit_condition: Option<Box<Pattern>>,
    },

    Dominates {
        dominator: Box<Pattern>,
        dominated: Box<Pattern>,
    },

    PathExists {
        from: Box<Pattern>,
        to: Box<Pattern>,
        constraints: Vec<PathConstraint>,
    },
}

#[derive(Debug, Clone)]
pub enum DataFlowPattern {
    DefUse {
        definition: Box<Pattern>,
        use_site: Box<Pattern>,
    },

    Tainted {
        source: Box<Pattern>,
        sink: Box<Pattern>,
        kind: TaintKind,
    },

    Depends {
        value: Box<Pattern>,
        on: Box<Pattern>,
    },
}

#[derive(Debug, Clone)]
pub enum TerminatorPattern {
    Return(Option<ValuePattern>),
    Branch {
        condition: ValuePattern,
        then_target: Option<String>,
        else_target: Option<String>,
    },
    Jump {
        target: Option<String>,
    },
    Unreachable,
}

#[derive(Debug, Clone)]
pub enum InstPredicate {
    IsPure,
    HasSideEffects,
    IsCall,
    IsStateModifying,
    IsExternal,
}

#[derive(Debug, Clone)]
pub enum BlockPredicate {
    IsEntry,
    IsExit,
    IsLoopHeader,
    InLoop,
    HasMultiplePredecessors,
}

#[derive(Debug, Clone)]
pub enum PathConstraint {
    MaxLength(usize),
    MustPassThrough(BlockId),
    MustNotPassThrough(BlockId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaintKind {
    UserInput,
    ExternalCall,
    UntrustedData,
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum Constraint {
    Equal(String, String),
    NotEqual(String, String),
    Arithmetic {
        left: String,
        op: ArithOp,
        right: String,
    },

    Custom(String),
}

#[derive(Debug, Clone)]
pub enum ArithOp {
    Lt,
    Le,
    Gt,
    Ge,
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone)]
pub struct Match {
    pub pattern: Pattern,
    pub location: MatchLocation,
    pub captures: HashMap<String, CapturedValue>,
}

#[derive(Debug, Clone)]
pub enum MatchLocation {
    Instruction { block: BlockId, index: usize },
    Block(BlockId),
    Value(Value),
    Function(String),
}

#[derive(Debug, Clone)]
pub enum CapturedValue {
    Instruction(Instruction),
    Value(Value),
    Block(BlockId),
    String(String),
}

pub struct PatternMatcher {
    compiled: Vec<CompiledPattern>,
}

struct CompiledPattern {
    matcher: Box<dyn Fn(&Function, &MatchContext) -> Vec<Match> + Send + Sync>,
}

/*
The MatchContext provides pattern matching state during analysis passes.
Fields are intentionally unused as they're reserved for future advanced matching capabilities.
*/
pub struct MatchContext<'a> {
    #[allow(dead_code)]
    function: &'a Function,
    #[allow(dead_code)]
    captures: HashMap<String, CapturedValue>,
    #[allow(dead_code)]
    current_block: Option<BlockId>,
    #[allow(dead_code)]
    current_inst_idx: Option<usize>,
}

impl PatternMatcher {
    pub fn new() -> Self {
        Self {
            compiled: Vec::new(),
        }
    }

    pub fn compile(&mut self, pattern: Pattern) {
        let matcher = self.compile_pattern(&pattern);
        self.compiled.push(CompiledPattern { matcher });
    }

    pub fn match_all(&self, function: &Function) -> Vec<Match> {
        let mut all_matches = Vec::new();
        let context = MatchContext {
            function,
            captures: HashMap::new(),
            current_block: None,
            current_inst_idx: None,
        };

        for compiled in &self.compiled {
            all_matches.extend((compiled.matcher)(function, &context));
        }

        all_matches
    }

    pub fn match_pattern(&self, pattern: &Pattern, function: &Function) -> Vec<Match> {
        let context = MatchContext {
            function,
            captures: HashMap::new(),
            current_block: None,
            current_inst_idx: None,
        };

        let matcher = self.compile_pattern(pattern);
        matcher(function, &context)
    }

    fn compile_pattern(
        &self,
        _pattern: &Pattern,
    ) -> Box<dyn Fn(&Function, &MatchContext) -> Vec<Match> + Send + Sync> {
        Box::new(|_function, _context| {
            vec![Match {
                pattern: Pattern::Wildcard,
                location: MatchLocation::Function("*".to_string()),
                captures: HashMap::new(),
            }]
        })
    }
}

pub struct PatternBuilder {
    pattern: Pattern,
}

impl PatternBuilder {
    pub fn new() -> Self {
        Self {
            pattern: Pattern::Wildcard,
        }
    }

    pub fn inst(mut self, opcode: InstKind) -> Self {
        self.pattern = Pattern::Inst(InstPattern {
            opcode: Some(opcode),
            args: Vec::new(),
            result: None,
            predicates: Vec::new(),
        });
        self
    }

    pub fn external_call(mut self) -> Self {
        self.pattern = Pattern::Inst(InstPattern {
            opcode: Some(InstKind::Call),
            args: Vec::new(),
            result: None,
            predicates: vec![InstPredicate::IsExternal],
        });
        self
    }

    pub fn state_write(mut self) -> Self {
        self.pattern = Pattern::Inst(InstPattern {
            opcode: Some(InstKind::StorageStore),
            args: Vec::new(),
            result: None,
            predicates: vec![InstPredicate::IsStateModifying],
        });
        self
    }

    pub fn then(mut self, next: Pattern) -> Self {
        self.pattern = Pattern::Sequence(vec![self.pattern, next]);
        self
    }

    pub fn build(self) -> Pattern {
        self.pattern
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_builder() {
        let pattern = PatternBuilder::new()
            .external_call()
            .then(PatternBuilder::new().state_write().build())
            .build();

        match pattern {
            Pattern::Sequence(patterns) => {
                assert_eq!(patterns.len(), 2);
            }
            _ => panic!("Expected sequence pattern"),
        }
    }

    /*
    Pattern matching tests should cover:
    - Wildcard matching
    - Instruction pattern matching with opcodes
    - Predicate matching (IsCall, IsStateModifying, IsExternal)
    - Sequence patterns
    - Capture and binding
    */
    #[test]
    #[ignore]
    fn test_pattern_matching() {}
}
