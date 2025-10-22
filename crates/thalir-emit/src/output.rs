use anyhow::Result;
use std::io::Write;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Text,
    Json,
    Markdown,
    Html,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputStyle {
    Compact,
    Pretty,
    Minimal,
}

pub trait OutputFormatter {
    fn format_pair(&self, key: &str, value: &str) -> String;

    fn format_list(&self, items: &[String]) -> String;

    fn format_section(&self, title: &str) -> String;

    fn format_code(&self, code: &str, language: Option<&str>) -> String;
}

pub struct TextFormatter;

impl OutputFormatter for TextFormatter {
    fn format_pair(&self, key: &str, value: &str) -> String {
        format!("{}: {}", key, value)
    }

    fn format_list(&self, items: &[String]) -> String {
        items
            .iter()
            .map(|item| format!("  - {}", item))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn format_section(&self, title: &str) -> String {
        format!("\n=== {} ===\n", title)
    }

    fn format_code(&self, code: &str, _language: Option<&str>) -> String {
        code.to_string()
    }
}

pub struct MarkdownFormatter;

impl OutputFormatter for MarkdownFormatter {
    fn format_pair(&self, key: &str, value: &str) -> String {
        format!("**{}**: {}", key, value)
    }

    fn format_list(&self, items: &[String]) -> String {
        items
            .iter()
            .map(|item| format!("- {}", item))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn format_section(&self, title: &str) -> String {
        format!("\n## {}\n", title)
    }

    fn format_code(&self, code: &str, language: Option<&str>) -> String {
        let lang = language.unwrap_or("solidity");
        format!("```{}\n{}\n```", lang, code)
    }
}

pub struct JsonFormatter;

impl JsonFormatter {
    pub fn format_object<W: Write>(writer: &mut W, obj: &serde_json::Value) -> Result<()> {
        serde_json::to_writer_pretty(writer, obj)?;
        Ok(())
    }
}
