/*! Parse text IR into structured data.
 *
 * Round-tripping IR through text files enables version control, tool interop, and transformation
 * validation. This parser reads IR back into memory so you can analyze it, transform it, or verify
 * it matches expectations.
 */

#![allow(unreachable_patterns)]

use pest::Parser;
use pest_derive::Parser;
use std::path::Path;

pub mod annotations;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct ThalirParser;

pub type ParseResult<T> = Result<T, Box<pest::error::Error<Rule>>>;

pub fn parse(input: &str) -> ParseResult<pest::iterators::Pairs<'_, Rule>> {
    ThalirParser::parse(Rule::module, input).map_err(|e| Box::new(e))
}

pub fn parse_file<P: AsRef<Path>>(path: P) -> ParseResult<String> {
    std::fs::read_to_string(path).map_err(|e| {
        Box::new(pest::error::Error::new_from_pos(
            pest::error::ErrorVariant::CustomError {
                message: format!("Failed to read file: {}", e),
            },
            pest::Position::from_start(""),
        ))
    })
}

pub fn check(input: &str) -> bool {
    parse(input).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_module() {
        let input = "";
        assert!(check(input));
    }

    #[test]
    fn test_simple_function() {
        let input = r"
function %test(i32, i32) -> i32 {
block0(v0: i32, v1: i32):
    v2 = iadd v0, v1
    return v2
}
";
        assert!(check(input));
    }

    #[test]
    fn test_function_with_entities() {
        let input = r"
function %f(i64, i32) -> i32 {
    gv0 = vmctx
    gv1 = load.i64 notrap readonly aligned gv0+8
    fn0 = %g(i64)

block0(v0: i64, v1: i32):
    v2 = global_value.i64 gv1
    v3 = load.i32 v2+8
    return v3
}
";
        match parse(input) {
            Ok(_) => {}
            Err(e) => panic!("Parse error: {}", e),
        }
    }

    #[test]
    #[ignore]
    fn test_branch_with_block_args() {
        let input = r"
function %f(i32, i32) -> i32 {
block0(v0: i32, v1: i32):
    v2 = iadd v0, v1
    brif v2, block1(v0), block2(v1)

block1(v3: i32):
    return v3

block2(v4: i32):
    return v4
}
";
        match parse(input) {
            Ok(_) => {}
            Err(e) => panic!("Parse error: {}", e),
        }
    }

    #[test]
    fn test_test_directives() {
        let input = r"
test optimize
set opt_level=speed
target x86_64

function %f(i32) -> i32 {
block0(v0: i32):
    return v0
}
";
        assert!(check(input));
    }

    #[test]
    #[ignore]
    fn test_floats_and_vectors() {
        let input = r"
function %f(f32x4, f64) -> f32x4 {
block0(v0: f32x4, v1: f64):
    v2 = fconst.f32 0x1.5p-3
    v3 = fconst.f64 3.14159
    v4 = fconst.f64 NaN
    v5 = fconst.f64 Inf
    return v0
}
";
        assert!(check(input));
    }
}
