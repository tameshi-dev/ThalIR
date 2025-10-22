use crate::ir_formatter_base::IRFormatterBase;
use crate::thalir_emitter::{SSAContext, ThalIREmitter};
use anyhow::Result;
use thalir_core::{
    block::{BasicBlock, Terminator},
    contract::Contract,
    function::Function,
    instructions::{CallTarget, Instruction},
    ObfuscationConfig, ObfuscationMapping,
};

#[derive(Debug, Clone)]
pub struct AnnotationConfig {
    pub emit_position_markers: bool,
    pub emit_visual_cues: bool,
    pub use_ascii_cues: bool,
    pub emit_ordering_analysis: bool,
    pub emit_function_headers: bool,
}

impl Default for AnnotationConfig {
    fn default() -> Self {
        Self {
            emit_position_markers: true,
            emit_visual_cues: true,
            use_ascii_cues: false,
            emit_ordering_analysis: true,
            emit_function_headers: true,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum VisualCue {
    ExternalCall,
    StateWrite,
    Warning,
    Checked,
    Safe,
    Unsafe,
    TxOrigin,
    Delegatecall,
    Selfdestruct,
    UncheckedArith,
    BlockTimestamp,
    BlockVariable,
}

impl VisualCue {
    fn to_emoji(&self) -> &'static str {
        match self {
            Self::ExternalCall => "",
            Self::StateWrite => "🟡",
            Self::Warning => "",
            Self::Checked => "",
            Self::Safe => "🟢",
            Self::Unsafe => "",
            Self::TxOrigin => "",
            Self::Delegatecall => "",
            Self::Selfdestruct => "",
            Self::UncheckedArith => "",
            Self::BlockTimestamp => "⏰",
            Self::BlockVariable => "",
        }
    }

    fn to_ascii(&self) -> &'static str {
        match self {
            Self::ExternalCall => "[EXTERNAL_CALL]",
            Self::StateWrite => "[STATE_WRITE]",
            Self::Warning => "[WARNING]",
            Self::Checked => "[CHECKED]",
            Self::Safe => "[SAFE]",
            Self::Unsafe => "[UNSAFE]",
            Self::TxOrigin => "[TX_ORIGIN]",
            Self::Delegatecall => "[DELEGATECALL]",
            Self::Selfdestruct => "[SELFDESTRUCT]",
            Self::UncheckedArith => "[UNCHECKED]",
            Self::BlockTimestamp => "[TIMESTAMP]",
            Self::BlockVariable => "[BLOCK_VAR]",
        }
    }

    fn format(&self, use_ascii: bool) -> &'static str {
        if use_ascii {
            self.to_ascii()
        } else {
            self.to_emoji()
        }
    }
}

#[derive(Debug)]
struct SecurityAnalysis {
    external_call_positions: Vec<usize>,
    state_modification_positions: Vec<usize>,
    tx_origin_positions: Vec<usize>,
    delegatecall_positions: Vec<usize>,
    selfdestruct_positions: Vec<usize>,
    unchecked_arith_positions: Vec<usize>,
    block_timestamp_positions: Vec<usize>,
    block_variable_positions: Vec<usize>,
}

impl SecurityAnalysis {
    fn new() -> Self {
        Self {
            external_call_positions: Vec::new(),
            state_modification_positions: Vec::new(),
            tx_origin_positions: Vec::new(),
            delegatecall_positions: Vec::new(),
            selfdestruct_positions: Vec::new(),
            unchecked_arith_positions: Vec::new(),
            block_timestamp_positions: Vec::new(),
            block_variable_positions: Vec::new(),
        }
    }

    fn has_reentrancy_risk(&self) -> bool {
        for &call_pos in &self.external_call_positions {
            for &mod_pos in &self.state_modification_positions {
                if call_pos < mod_pos {
                    return true;
                }
            }
        }
        false
    }

    fn has_security_issues(&self) -> bool {
        self.has_reentrancy_risk()
            || !self.tx_origin_positions.is_empty()
            || !self.delegatecall_positions.is_empty()
            || !self.selfdestruct_positions.is_empty()
            || !self.unchecked_arith_positions.is_empty()
            || !self.block_timestamp_positions.is_empty()
            || !self.block_variable_positions.is_empty()
    }
}

pub struct AnnotatedIREmitter {
    base_emitter: ThalIREmitter,
    annotation_config: AnnotationConfig,
    contracts: Vec<Contract>,
}

impl AnnotatedIREmitter {
    pub fn new(contracts: Vec<Contract>) -> Self {
        Self {
            base_emitter: ThalIREmitter::new(contracts.clone()),
            annotation_config: AnnotationConfig::default(),
            contracts,
        }
    }

    pub fn with_annotation_config(mut self, config: AnnotationConfig) -> Self {
        self.annotation_config = config;
        self
    }

    pub fn with_obfuscation(
        contracts: Vec<Contract>,
        obf_config: ObfuscationConfig,
        ann_config: AnnotationConfig,
    ) -> Result<(Self, Option<ObfuscationMapping>)> {
        let (base_emitter, mapping) =
            ThalIREmitter::with_obfuscation(contracts.clone(), obf_config)?;

        let obfuscated_contracts = base_emitter.contracts.clone();

        let annotated = Self {
            base_emitter,
            annotation_config: ann_config,
            contracts: obfuscated_contracts,
        };

        Ok((annotated, mapping))
    }

    pub fn emit_to_string(&self, with_types: bool) -> String {
        let mut output = String::new();

        for contract in &self.contracts {
            self.emit_contract(&mut output, contract, with_types);
        }

        output
    }

    fn emit_contract(&self, output: &mut String, contract: &Contract, with_types: bool) {
        output.push_str(&format!("contract {} {{\n", contract.name));

        if !contract.storage_layout.slots.is_empty() {
            output.push_str("\n  // Storage Layout\n");
            for var in &contract.storage_layout.slots {
                output.push_str(&format!(
                    "  slot {} = {}: {}\n",
                    var.slot,
                    var.name,
                    IRFormatterBase::format_type(&var.var_type)
                ));
            }
        }

        let mut ssa = SSAContext::new();
        for (name, function) in &contract.functions {
            output.push_str("\n");
            self.emit_function(output, name, function, &mut ssa, with_types);
        }

        output.push_str("}\n");
    }

    fn emit_function(
        &self,
        output: &mut String,
        name: &str,
        function: &Function,
        ssa: &mut SSAContext,
        _with_types: bool,
    ) {
        ssa.reset();

        let analysis = self.analyze_security(function);

        if self.annotation_config.emit_function_headers {
            output.push_str(&format!(
                "; ### Function: {} ({})\n",
                name,
                IRFormatterBase::format_visibility(&function.visibility).to_uppercase()
            ));

            if !analysis.external_call_positions.is_empty() {
                output.push_str(&format!(
                    "; - External Calls: {}\n",
                    analysis.external_call_positions.len()
                ));
            }
            if !analysis.state_modification_positions.is_empty() {
                output.push_str(&format!(
                    "; - State Modifications: {}\n",
                    analysis.state_modification_positions.len()
                ));
            }
            if !analysis.tx_origin_positions.is_empty() {
                output.push_str(&format!(
                    "; -  tx.origin Usage: {}\n",
                    analysis.tx_origin_positions.len()
                ));
            }
            if !analysis.delegatecall_positions.is_empty() {
                output.push_str(&format!(
                    "; -  Delegatecalls: {}\n",
                    analysis.delegatecall_positions.len()
                ));
            }
            if !analysis.selfdestruct_positions.is_empty() {
                output.push_str(&format!(
                    "; -  Selfdestruct: {}\n",
                    analysis.selfdestruct_positions.len()
                ));
            }
        }

        if self.annotation_config.emit_ordering_analysis && analysis.has_security_issues() {
            self.emit_security_analysis_comment(output, &analysis);
        }

        let param_vnums: Vec<u32> = (0..function.signature.params.len())
            .map(|_| ssa.allocate_new())
            .collect();

        let param_types: Vec<String> = function
            .signature
            .params
            .iter()
            .map(|p| IRFormatterBase::format_type(&p.param_type))
            .collect();

        let return_type = if !function.signature.returns.is_empty() {
            format!(
                " -> {}",
                function
                    .signature
                    .returns
                    .iter()
                    .map(|t| IRFormatterBase::format_type(t))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        } else {
            String::new()
        };

        let visibility = IRFormatterBase::format_visibility(&function.visibility);
        let mutability = IRFormatterBase::format_mutability(&function.mutability);

        output.push_str(&format!(
            "  function %{}({}){} {} {} {{\n",
            name,
            param_types.join(", "),
            return_type,
            visibility,
            mutability
        ));

        if let Some(entry_block) = function.body.blocks.get(&function.body.entry_block) {
            output.push_str(&format!("  block{}(", entry_block.id.0));
            for (i, param) in function.signature.params.iter().enumerate() {
                if i > 0 {
                    output.push_str(", ");
                }
                output.push_str(&format!(
                    "v{}: {}",
                    param_vnums[i],
                    IRFormatterBase::format_type(&param.param_type)
                ));
            }
            output.push_str("):\n");

            let mut position = 0;
            self.emit_block_body(output, entry_block, ssa, &param_vnums, &mut position);

            for (block_id, block) in &function.body.blocks {
                if block_id != &function.body.entry_block {
                    output.push_str(&format!("\n  block{}:\n", block.id.0));
                    self.emit_block_body(output, block, ssa, &param_vnums, &mut position);
                }
            }
        }

        output.push_str("  }\n");
    }

    fn emit_block_body(
        &self,
        output: &mut String,
        block: &BasicBlock,
        ssa: &mut SSAContext,
        param_vnums: &[u32],
        position: &mut usize,
    ) {
        for inst in &block.instructions {
            let visual_cue = self.get_visual_cue(inst);

            output.push_str("    ");

            if self.annotation_config.emit_position_markers {
                output.push_str(&format!("[{}] ", position));
            }

            if self.annotation_config.emit_visual_cues {
                if let Some(cue) = visual_cue {
                    output.push_str(&format!(
                        "{} ",
                        cue.format(self.annotation_config.use_ascii_cues)
                    ));
                }
            }

            let inst_str = self.base_emitter.format_instruction(inst, ssa, param_vnums);
            output.push_str(&inst_str);
            output.push('\n');

            *position += 1;
        }

        output.push_str("    ");
        self.emit_terminator(output, &block.terminator, ssa, param_vnums);
        output.push('\n');
    }

    fn emit_terminator(
        &self,
        output: &mut String,
        terminator: &Terminator,
        ssa: &mut SSAContext,
        param_vnums: &[u32],
    ) {
        match terminator {
            Terminator::Return(None) => {
                output.push_str("return");
            }
            Terminator::Return(Some(val)) => {
                let v = self.base_emitter.format_value(val, ssa, param_vnums);
                output.push_str(&format!("return {}", v));
            }
            Terminator::Jump(target, args) => {
                if args.is_empty() {
                    output.push_str(&format!("jmp block{}", target.0));
                } else {
                    let arg_strs: Vec<String> = args
                        .iter()
                        .map(|v| self.base_emitter.format_value(v, ssa, param_vnums))
                        .collect();
                    output.push_str(&format!("jmp block{}({})", target.0, arg_strs.join(", ")));
                }
            }
            Terminator::Branch {
                condition,
                then_block,
                then_args,
                else_block,
                else_args,
            } => {
                let cond_v = self.base_emitter.format_value(condition, ssa, param_vnums);
                let then_str = if then_args.is_empty() {
                    format!("block{}", then_block.0)
                } else {
                    let args: Vec<String> = then_args
                        .iter()
                        .map(|v| self.base_emitter.format_value(v, ssa, param_vnums))
                        .collect();
                    format!("block{}({})", then_block.0, args.join(", "))
                };
                let else_str = if else_args.is_empty() {
                    format!("block{}", else_block.0)
                } else {
                    let args: Vec<String> = else_args
                        .iter()
                        .map(|v| self.base_emitter.format_value(v, ssa, param_vnums))
                        .collect();
                    format!("block{}({})", else_block.0, args.join(", "))
                };
                output.push_str(&format!("br {}, {}, {}", cond_v, then_str, else_str));
            }
            Terminator::Switch {
                value,
                default,
                cases,
            } => {
                let val_str = self.base_emitter.format_value(value, ssa, param_vnums);
                output.push_str(&format!("switch {}, block{}, [", val_str, default.0));
                for (i, (case_val, block_id)) in cases.iter().enumerate() {
                    if i > 0 {
                        output.push_str(", ");
                    }
                    let case_str = self.base_emitter.format_value(case_val, ssa, param_vnums);
                    output.push_str(&format!("{}: block{}", case_str, block_id.0));
                }
                output.push(']');
            }
            Terminator::Revert(msg) => {
                output.push_str(&format!("revert \"{}\"", msg));
            }
            Terminator::Panic(msg) => {
                output.push_str(&format!("panic \"{}\"", msg));
            }
            Terminator::Invalid => {
                output.push_str("invalid");
            }
        }
    }

    fn get_visual_cue(&self, inst: &Instruction) -> Option<VisualCue> {
        use thalir_core::instructions::ContextVariable;

        match inst {
            Instruction::Call {
                target: CallTarget::External(_),
                ..
            } => Some(VisualCue::ExternalCall),

            Instruction::DelegateCall { .. } => Some(VisualCue::Delegatecall),

            Instruction::Selfdestruct { .. } => Some(VisualCue::Selfdestruct),

            Instruction::StorageStore { .. } | Instruction::MappingStore { .. } => {
                Some(VisualCue::StateWrite)
            }

            Instruction::CheckedAdd { .. }
            | Instruction::CheckedSub { .. }
            | Instruction::CheckedMul { .. }
            | Instruction::CheckedDiv { .. } => Some(VisualCue::Checked),

            Instruction::Add { .. }
            | Instruction::Sub { .. }
            | Instruction::Mul { .. }
            | Instruction::Div { .. } => Some(VisualCue::UncheckedArith),

            Instruction::GetContext { var, .. } => match var {
                ContextVariable::TxOrigin => Some(VisualCue::TxOrigin),
                ContextVariable::BlockTimestamp => Some(VisualCue::BlockTimestamp),
                ContextVariable::BlockNumber
                | ContextVariable::BlockDifficulty
                | ContextVariable::BlockGasLimit
                | ContextVariable::BlockCoinbase
                | ContextVariable::BlockBaseFee => Some(VisualCue::BlockVariable),
                _ => None,
            },
            _ => None,
        }
    }

    fn analyze_security(&self, function: &Function) -> SecurityAnalysis {
        use thalir_core::instructions::ContextVariable;

        let mut analysis = SecurityAnalysis::new();
        let mut position = 0;

        for block in function.body.blocks.values() {
            for inst in &block.instructions {
                match inst {
                    Instruction::Call {
                        target: CallTarget::External(_),
                        ..
                    } => {
                        analysis.external_call_positions.push(position);
                    }
                    Instruction::StorageStore { .. } | Instruction::MappingStore { .. } => {
                        analysis.state_modification_positions.push(position);
                    }

                    Instruction::DelegateCall { .. } => {
                        analysis.delegatecall_positions.push(position);
                    }
                    Instruction::Selfdestruct { .. } => {
                        analysis.selfdestruct_positions.push(position);
                    }
                    Instruction::Add { .. }
                    | Instruction::Sub { .. }
                    | Instruction::Mul { .. }
                    | Instruction::Div { .. } => {
                        analysis.unchecked_arith_positions.push(position);
                    }
                    Instruction::GetContext { var, .. } => match var {
                        ContextVariable::TxOrigin => {
                            analysis.tx_origin_positions.push(position);
                        }
                        ContextVariable::BlockTimestamp => {
                            analysis.block_timestamp_positions.push(position);
                        }
                        ContextVariable::BlockNumber
                        | ContextVariable::BlockDifficulty
                        | ContextVariable::BlockGasLimit
                        | ContextVariable::BlockCoinbase
                        | ContextVariable::BlockBaseFee => {
                            analysis.block_variable_positions.push(position);
                        }
                        _ => {}
                    },
                    _ => {}
                }
                position += 1;
            }
        }

        analysis
    }

    fn emit_security_analysis_comment(&self, output: &mut String, analysis: &SecurityAnalysis) {
        output.push_str(";  SECURITY ANALYSIS:\n");

        for &pos in &analysis.external_call_positions {
            output.push_str(&format!("; - External call at position [{}]\n", pos));
        }
        for &pos in &analysis.state_modification_positions {
            output.push_str(&format!("; - State modification at position [{}]\n", pos));
        }
        for &call_pos in &analysis.external_call_positions {
            for &mod_pos in &analysis.state_modification_positions {
                if call_pos < mod_pos {
                    output.push_str(&format!(
                        "; - [{}] < [{}] → REENTRANCY RISK\n",
                        call_pos, mod_pos
                    ));
                }
            }
        }

        if !analysis.tx_origin_positions.is_empty() {
            output.push_str("; -  TX.ORIGIN USAGE (phishing risk): ");
            for (i, &pos) in analysis.tx_origin_positions.iter().enumerate() {
                if i > 0 {
                    output.push_str(", ");
                }
                output.push_str(&format!("[{}]", pos));
            }
            output.push('\n');
        }

        if !analysis.delegatecall_positions.is_empty() {
            output.push_str("; -  DELEGATECALL (storage hijacking risk): ");
            for (i, &pos) in analysis.delegatecall_positions.iter().enumerate() {
                if i > 0 {
                    output.push_str(", ");
                }
                output.push_str(&format!("[{}]", pos));
            }
            output.push('\n');
        }

        if !analysis.selfdestruct_positions.is_empty() {
            output.push_str("; -  SELFDESTRUCT (deprecated/dangerous): ");
            for (i, &pos) in analysis.selfdestruct_positions.iter().enumerate() {
                if i > 0 {
                    output.push_str(", ");
                }
                output.push_str(&format!("[{}]", pos));
            }
            output.push('\n');
        }

        if !analysis.unchecked_arith_positions.is_empty() {
            output.push_str("; -  UNCHECKED ARITHMETIC (overflow risk): ");
            let count = analysis.unchecked_arith_positions.len();
            if count > 5 {
                output.push_str(&format!("{} operations at ", count));
                for (i, &pos) in analysis
                    .unchecked_arith_positions
                    .iter()
                    .take(3)
                    .enumerate()
                {
                    if i > 0 {
                        output.push_str(", ");
                    }
                    output.push_str(&format!("[{}]", pos));
                }
                output.push_str(&format!(", ... and {} more", count - 3));
            } else {
                for (i, &pos) in analysis.unchecked_arith_positions.iter().enumerate() {
                    if i > 0 {
                        output.push_str(", ");
                    }
                    output.push_str(&format!("[{}]", pos));
                }
            }
            output.push('\n');
        }

        if !analysis.block_timestamp_positions.is_empty() {
            output.push_str("; - ⏰ BLOCK.TIMESTAMP (miner manipulation): ");
            for (i, &pos) in analysis.block_timestamp_positions.iter().enumerate() {
                if i > 0 {
                    output.push_str(", ");
                }
                output.push_str(&format!("[{}]", pos));
            }
            output.push('\n');
        }

        if !analysis.block_variable_positions.is_empty() {
            output.push_str("; -  BLOCK VARIABLES (miner influence): ");
            for (i, &pos) in analysis.block_variable_positions.iter().enumerate() {
                if i > 0 {
                    output.push_str(", ");
                }
                output.push_str(&format!("[{}]", pos));
            }
            output.push('\n');
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visual_cue_emoji() {
        assert_eq!(VisualCue::ExternalCall.to_emoji(), "");
        assert_eq!(VisualCue::StateWrite.to_emoji(), "🟡");
        assert_eq!(VisualCue::Warning.to_emoji(), "");
    }

    #[test]
    fn test_visual_cue_ascii() {
        assert_eq!(VisualCue::ExternalCall.to_ascii(), "[EXTERNAL_CALL]");
        assert_eq!(VisualCue::StateWrite.to_ascii(), "[STATE_WRITE]");
        assert_eq!(VisualCue::Warning.to_ascii(), "[WARNING]");
    }

    #[test]
    fn test_security_analysis_reentrancy() {
        let mut analysis = SecurityAnalysis::new();
        analysis.external_call_positions.push(5);
        analysis.state_modification_positions.push(10);

        assert!(analysis.has_reentrancy_risk());
        assert!(analysis.has_security_issues());
    }

    #[test]
    fn test_security_analysis_safe() {
        let mut analysis = SecurityAnalysis::new();
        analysis.state_modification_positions.push(3);
        analysis.external_call_positions.push(7);

        assert!(!analysis.has_reentrancy_risk());
        assert!(!analysis.has_security_issues());
    }

    #[test]
    fn test_security_analysis_tier1_patterns() {
        let mut analysis = SecurityAnalysis::new();

        assert!(!analysis.has_security_issues());

        analysis.tx_origin_positions.push(3);
        assert!(analysis.has_security_issues());

        let mut analysis2 = SecurityAnalysis::new();
        analysis2.delegatecall_positions.push(5);
        assert!(analysis2.has_security_issues());

        let mut analysis3 = SecurityAnalysis::new();
        analysis3.selfdestruct_positions.push(8);
        assert!(analysis3.has_security_issues());

        let mut analysis4 = SecurityAnalysis::new();
        analysis4.unchecked_arith_positions.push(2);
        assert!(analysis4.has_security_issues());

        let mut analysis5 = SecurityAnalysis::new();
        analysis5.block_timestamp_positions.push(4);
        assert!(analysis5.has_security_issues());

        let mut analysis6 = SecurityAnalysis::new();
        analysis6.block_variable_positions.push(6);
        assert!(analysis6.has_security_issues());
    }
}
