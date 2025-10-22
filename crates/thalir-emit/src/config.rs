use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmitterConfig {
    pub use_colors: bool,
    pub indent_style: IndentStyle,
    pub max_line_width: Option<usize>,
    pub include_source_mappings: bool,
    pub include_types: bool,
    pub verbosity: VerbosityLevel,
}

impl Default for EmitterConfig {
    fn default() -> Self {
        Self {
            use_colors: true,
            indent_style: IndentStyle::Spaces(4),
            max_line_width: Some(120),
            include_source_mappings: false,
            include_types: true,
            verbosity: VerbosityLevel::Normal,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndentStyle {
    Spaces(usize),
    Tabs,
}

impl IndentStyle {
    pub fn to_string(&self) -> String {
        match self {
            IndentStyle::Spaces(n) => " ".repeat(*n),
            IndentStyle::Tabs => "\t".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerbosityLevel {
    Quiet,
    Normal,
    Verbose,
    Debug,
}

impl VerbosityLevel {
    pub fn should_print_types(&self) -> bool {
        matches!(self, VerbosityLevel::Verbose | VerbosityLevel::Debug)
    }

    pub fn should_print_source_mappings(&self) -> bool {
        matches!(self, VerbosityLevel::Debug)
    }

    pub fn should_print_ids(&self) -> bool {
        matches!(self, VerbosityLevel::Debug)
    }
}
