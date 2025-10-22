use crate::Rule;
use pest::iterators::Pair;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VisualCue {
    ExternalCall,
    StateWrite,
    Warning,
    Checked,
    Safe,
    Unsafe,
}

impl VisualCue {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "🔴" | "[EXTERNAL_CALL]" => Some(Self::ExternalCall),
            "🟡" | "[STATE_WRITE]" => Some(Self::StateWrite),
            "⚠️" | "[WARNING]" => Some(Self::Warning),
            "✓" | "[CHECKED]" => Some(Self::Checked),
            "🟢" | "[SAFE]" => Some(Self::Safe),
            "❌" | "[UNSAFE]" => Some(Self::Unsafe),
            _ => None,
        }
    }

    pub fn to_emoji(&self) -> &'static str {
        match self {
            Self::ExternalCall => "",
            Self::StateWrite => "🟡",
            Self::Warning => "",
            Self::Checked => "",
            Self::Safe => "🟢",
            Self::Unsafe => "",
        }
    }

    pub fn to_ascii(&self) -> &'static str {
        match self {
            Self::ExternalCall => "[EXTERNAL_CALL]",
            Self::StateWrite => "[STATE_WRITE]",
            Self::Warning => "[WARNING]",
            Self::Checked => "[CHECKED]",
            Self::Safe => "[SAFE]",
            Self::Unsafe => "[UNSAFE]",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct InstructionAnnotations {
    pub position: Option<usize>,
    pub visual_cue: Option<VisualCue>,
}

#[derive(Debug, Clone)]
pub enum AnalysisComment {
    FunctionHeader {
        name: String,
        visibility: Option<String>,
    },

    OrderingHeader,

    ExternalCallPosition(usize),

    StateModificationPosition(usize),

    OrderingComparison {
        position1: usize,
        operator: String,
        position2: usize,
        result: String,
    },

    Other(String),
}

pub fn extract_position(pair: &Pair<Rule>) -> Option<usize> {
    pair.clone()
        .into_inner()
        .find(|p| p.as_rule() == Rule::position_marker)
        .and_then(|p| {
            let text = p.as_str();

            text.trim_matches(|c| c == '[' || c == ']')
                .parse::<usize>()
                .ok()
        })
}

pub fn extract_visual_cue(pair: &Pair<Rule>) -> Option<VisualCue> {
    pair.clone().into_inner().find_map(|p| match p.as_rule() {
        Rule::visual_marker => p
            .into_inner()
            .next()
            .and_then(|inner| VisualCue::from_str(inner.as_str())),
        _ => None,
    })
}

pub fn extract_instruction_annotations(pair: &Pair<Rule>) -> InstructionAnnotations {
    InstructionAnnotations {
        position: extract_position(pair),
        visual_cue: extract_visual_cue(pair),
    }
}

pub fn extract_analysis_comment(pair: &Pair<Rule>) -> Option<AnalysisComment> {
    if pair.as_rule() != Rule::analysis_comment {
        return None;
    }

    let text = pair.as_str();

    if text.contains("### Function:") {
        let parts: Vec<&str> = text.split("Function:").collect();
        if parts.len() > 1 {
            let rest = parts[1].trim();
            let (name, visibility) = if rest.contains('(') {
                let name_parts: Vec<&str> = rest.split('(').collect();
                let name = name_parts[0].trim().to_string();
                let vis = name_parts
                    .get(1)
                    .and_then(|s| s.trim_end_matches(')').trim().split_whitespace().next())
                    .map(|s| s.to_string());
                (name, vis)
            } else {
                (rest.to_string(), None)
            };
            return Some(AnalysisComment::FunctionHeader { name, visibility });
        }
    }

    if text.contains("ORDERING ANALYSIS") {
        return Some(AnalysisComment::OrderingHeader);
    }

    if text.contains("External call at position") {
        if let Some(pos) = extract_position_from_text(text) {
            return Some(AnalysisComment::ExternalCallPosition(pos));
        }
    }

    if text.contains("State modification at position") {
        if let Some(pos) = extract_position_from_text(text) {
            return Some(AnalysisComment::StateModificationPosition(pos));
        }
    }

    if text.contains("→") {
        if let Some(comparison) = parse_ordering_comparison(text) {
            return Some(comparison);
        }
    }

    Some(AnalysisComment::Other(text.to_string()))
}

fn extract_position_from_text(text: &str) -> Option<usize> {
    let start = text.find('[')?;
    let end = text[start..].find(']')?;
    let num_str = &text[start + 1..start + end];
    num_str.parse::<usize>().ok()
}

fn parse_ordering_comparison(text: &str) -> Option<AnalysisComment> {
    let parts: Vec<&str> = text.split('→').collect();
    if parts.len() != 2 {
        return None;
    }

    let comparison_part = parts[0].trim();
    let result_part = parts[1].trim();

    let tokens: Vec<&str> = comparison_part.split_whitespace().collect();
    if tokens.len() < 3 {
        return None;
    }

    let pos1_str = tokens.iter().find(|s| s.starts_with('['))?;
    let pos2_str = tokens.iter().rev().find(|s| s.starts_with('['))?;
    let operator = tokens
        .iter()
        .find(|s| ["<", ">", "==", "!=", "<=", ">="].contains(&s.trim()))?;

    let pos1 = pos1_str
        .trim_matches(|c| c == '[' || c == ']')
        .parse::<usize>()
        .ok()?;
    let pos2 = pos2_str
        .trim_matches(|c| c == '[' || c == ']')
        .parse::<usize>()
        .ok()?;

    Some(AnalysisComment::OrderingComparison {
        position1: pos1,
        operator: operator.to_string(),
        position2: pos2,
        result: result_part.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visual_cue_from_emoji() {
        assert_eq!(VisualCue::from_str("🔴"), Some(VisualCue::ExternalCall));
        assert_eq!(VisualCue::from_str("🟡"), Some(VisualCue::StateWrite));
        assert_eq!(VisualCue::from_str("⚠️"), Some(VisualCue::Warning));
    }

    #[test]
    fn test_visual_cue_from_ascii() {
        assert_eq!(
            VisualCue::from_str("[EXTERNAL_CALL]"),
            Some(VisualCue::ExternalCall)
        );
        assert_eq!(
            VisualCue::from_str("[STATE_WRITE]"),
            Some(VisualCue::StateWrite)
        );
        assert_eq!(VisualCue::from_str("[WARNING]"), Some(VisualCue::Warning));
    }

    #[test]
    fn test_extract_position_from_text() {
        assert_eq!(
            extract_position_from_text("; - External call at position [42]"),
            Some(42)
        );
        assert_eq!(
            extract_position_from_text("; - [5] < [8] → REENTRANCY RISK"),
            Some(5)
        );
    }

    #[test]
    fn test_parse_ordering_comparison() {
        let text = "; - [4] < [8] → REENTRANCY RISK";
        let result = parse_ordering_comparison(text);

        match result {
            Some(AnalysisComment::OrderingComparison {
                position1,
                operator,
                position2,
                result,
            }) => {
                assert_eq!(position1, 4);
                assert_eq!(operator, "<");
                assert_eq!(position2, 8);
                assert_eq!(result, "REENTRANCY RISK");
            }
            _ => panic!("Expected OrderingComparison"),
        }
    }
}
