use anyhow::Result;
use std::io::Write;

pub type EmitResult = Result<()>;

#[derive(Debug, Clone)]
pub struct EmitContext {
    pub indent_level: usize,
    pub indent_chars: String,
    pub use_colors: bool,
    pub max_width: Option<usize>,
    pub current_line_pos: usize,
}

impl EmitContext {
    pub fn new() -> Self {
        Self {
            indent_level: 0,
            indent_chars: "    ".to_string(),
            use_colors: true,
            max_width: Some(120),
            current_line_pos: 0,
        }
    }

    pub fn indent(&mut self) {
        self.indent_level += 1;
    }

    pub fn dedent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }

    pub fn get_indent(&self) -> String {
        self.indent_chars.repeat(self.indent_level)
    }

    pub fn nested(&self) -> Self {
        let mut ctx = self.clone();
        ctx.indent();
        ctx
    }
}

impl Default for EmitContext {
    fn default() -> Self {
        Self::new()
    }
}

pub trait Emitter {
    type Item;

    fn emit<W: Write>(
        &self,
        item: &Self::Item,
        writer: &mut W,
        context: &mut EmitContext,
    ) -> EmitResult;

    fn emit_to_string(&self, item: &Self::Item) -> Result<String> {
        let mut buffer = Vec::new();
        let mut context = EmitContext::new();
        self.emit(item, &mut buffer, &mut context)?;
        Ok(String::from_utf8(buffer)?)
    }
}

pub trait Emittable {
    fn emit<W: Write>(&self, writer: &mut W, context: &mut EmitContext) -> EmitResult;

    fn to_formatted_string(&self) -> Result<String> {
        let mut buffer = Vec::new();
        let mut context = EmitContext::new();
        self.emit(&mut buffer, &mut context)?;
        Ok(String::from_utf8(buffer)?)
    }
}

pub struct EmitHelper;

impl EmitHelper {
    pub fn write_line<W: Write>(writer: &mut W, context: &EmitContext, text: &str) -> EmitResult {
        writeln!(writer, "{}{}", context.get_indent(), text)?;
        Ok(())
    }

    pub fn write<W: Write>(writer: &mut W, context: &EmitContext, text: &str) -> EmitResult {
        write!(writer, "{}{}", context.get_indent(), text)?;
        Ok(())
    }

    pub fn write_colored_line<W: Write>(
        writer: &mut W,
        context: &EmitContext,
        text: &str,
        color: &str,
    ) -> EmitResult {
        if context.use_colors {
            use colored::Colorize;
            let colored_text = match color {
                "red" => text.red().to_string(),
                "green" => text.green().to_string(),
                "blue" => text.blue().to_string(),
                "yellow" => text.yellow().to_string(),
                "magenta" => text.magenta().to_string(),
                "cyan" => text.cyan().to_string(),
                "white" => text.white().to_string(),
                "bright_red" => text.bright_red().to_string(),
                "bright_green" => text.bright_green().to_string(),
                "bright_blue" => text.bright_blue().to_string(),
                _ => text.to_string(),
            };
            writeln!(writer, "{}{}", context.get_indent(), colored_text)?;
        } else {
            Self::write_line(writer, context, text)?;
        }
        Ok(())
    }

    pub fn write_comment<W: Write>(
        writer: &mut W,
        context: &EmitContext,
        comment: &str,
    ) -> EmitResult {
        Self::write_colored_line(writer, context, &format!("// {}", comment), "green")
    }

    pub fn write_section<W: Write>(
        writer: &mut W,
        context: &EmitContext,
        title: &str,
    ) -> EmitResult {
        writeln!(writer)?;
        Self::write_colored_line(writer, context, &format!("=== {} ===", title), "cyan")?;
        Ok(())
    }

    pub fn write_block<W: Write, F>(
        writer: &mut W,
        context: &mut EmitContext,
        header: &str,
        body: F,
    ) -> EmitResult
    where
        F: FnOnce(&mut W, &mut EmitContext) -> EmitResult,
    {
        Self::write_line(writer, context, &format!("{} {{", header))?;
        context.indent();
        body(writer, context)?;
        context.dedent();
        Self::write_line(writer, context, "}")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_context_indentation() {
        let mut ctx = EmitContext::new();
        assert_eq!(ctx.indent_level, 0);
        assert_eq!(ctx.get_indent(), "");

        ctx.indent();
        assert_eq!(ctx.indent_level, 1);
        assert_eq!(ctx.get_indent(), "    ");

        ctx.indent();
        assert_eq!(ctx.indent_level, 2);
        assert_eq!(ctx.get_indent(), "        ");

        ctx.dedent();
        assert_eq!(ctx.indent_level, 1);
        assert_eq!(ctx.get_indent(), "    ");

        ctx.dedent();
        ctx.dedent();
        assert_eq!(ctx.indent_level, 0);
        assert_eq!(ctx.get_indent(), "");
    }

    #[test]
    fn test_nested_context() {
        let ctx = EmitContext::new();
        let nested = ctx.nested();

        assert_eq!(ctx.indent_level, 0);
        assert_eq!(nested.indent_level, 1);

        let double_nested = nested.nested();
        assert_eq!(double_nested.indent_level, 2);
    }

    #[test]
    fn test_custom_indent_chars() {
        let mut ctx = EmitContext::new();
        ctx.indent_chars = "\t".to_string();

        ctx.indent();
        assert_eq!(ctx.get_indent(), "\t");

        ctx.indent();
        assert_eq!(ctx.get_indent(), "\t\t");
    }

    #[test]
    fn test_print_helper_write_line() {
        let mut buffer = Vec::new();
        let ctx = EmitContext::new();

        EmitHelper::write_line(&mut buffer, &ctx, "test line").unwrap();
        assert_eq!(String::from_utf8(buffer).unwrap(), "test line\n");
    }

    #[test]
    fn test_print_helper_write_indented() {
        let mut buffer = Vec::new();
        let mut ctx = EmitContext::new();
        ctx.indent();

        EmitHelper::write_line(&mut buffer, &ctx, "indented line").unwrap();
        assert_eq!(String::from_utf8(buffer).unwrap(), "    indented line\n");
    }

    #[test]
    fn test_print_helper_write_comment() {
        let mut buffer = Vec::new();
        let mut ctx = EmitContext::new();
        ctx.use_colors = false;

        EmitHelper::write_comment(&mut buffer, &ctx, "This is a comment").unwrap();
        assert_eq!(String::from_utf8(buffer).unwrap(), "// This is a comment\n");
    }

    #[test]
    fn test_print_helper_write_section() {
        let mut buffer = Vec::new();
        let mut ctx = EmitContext::new();
        ctx.use_colors = false;

        EmitHelper::write_section(&mut buffer, &ctx, "Test Section").unwrap();
        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("=== Test Section ==="));
    }

    #[test]
    fn test_print_helper_write_block() {
        let mut buffer = Vec::new();
        let mut ctx = EmitContext::new();
        ctx.use_colors = false;

        EmitHelper::write_block(&mut buffer, &mut ctx, "test", |w, c| {
            EmitHelper::write_line(w, c, "inside block")
        })
        .unwrap();

        let output = String::from_utf8(buffer).unwrap();
        assert_eq!(output, "test {\n    inside block\n}\n");
    }

    #[test]
    fn test_print_helper_colored_output() {
        let mut buffer = Vec::new();
        let mut ctx = EmitContext::new();
        ctx.use_colors = true;

        EmitHelper::write_colored_line(&mut buffer, &ctx, "red text", "red").unwrap();
        EmitHelper::write_colored_line(&mut buffer, &ctx, "blue text", "blue").unwrap();

        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("red text"));
        assert!(output.contains("blue text"));
    }
}
