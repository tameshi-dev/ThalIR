/*! Analyze control flow, data flow, and program structure.
 *
 * Finding vulnerabilities requires understanding how values flow through a program and which code
 * paths are reachable. These passes provide CFG construction, dominance trees, and alias analysis—
 * the foundation for pattern matching and verification.
 */

pub mod alias;
pub mod cache;
pub mod cfg;
pub mod control_flow;
pub mod cursor;
pub mod dataflow;
pub mod def_use;
pub mod dominator;
pub mod pass;
pub mod passes;
pub mod pattern;

pub use alias::{AliasAnalysis, AliasResult, AliasSet, PointsToSet};
pub use cache::{AnalysisCache, CacheKey};
pub use control_flow::{ControlFlowGraph, Loop};
pub use cursor::{CursorPosition, IRCursor, ScannerCursor};
pub use def_use::{DefKind, DefUseChains, Definition, Use, UseKind};
pub use dominator::DominatorTree;
pub use pass::{AnalysisID, AnalysisPass, Pass, PassManager};
pub use pattern::{Pattern, PatternBuilder, PatternMatcher};
