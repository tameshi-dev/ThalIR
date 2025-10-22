use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use thiserror::Error;

pub const MAX_LINES: usize = 1_000_000;

pub const MAX_FILE_SIZE: usize = u32::MAX as usize;

#[derive(Error, Debug)]
pub enum SourceLocationError {
    #[error("File exceeds maximum line count: {0} lines")]
    TooManyLines(usize),
    #[error("File exceeds maximum size: {0} bytes")]
    FileTooLarge(usize),

    #[error("Invalid UTF-8 at byte offset {0}")]
    InvalidUtf8(usize),
    #[error("Invalid file ID: {0}")]
    InvalidFileId(u32),
    #[error("Invalid source span: file_id={0}, start={1}, len={2}")]
    InvalidSpan(u32, u32, u32),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, SourceLocationError>;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SourceSpan {
    pub file_id: u32,
    pub start: u32,
    pub len: u32,
}

pub const INVALID_SPAN: SourceSpan = SourceSpan {
    file_id: u32::MAX,
    start: 0,
    len: 0,
};

impl SourceSpan {
    pub fn new(file_id: u32, start: u32, len: u32) -> Self {
        Self {
            file_id,
            start,
            len,
        }
    }

    #[inline]
    pub fn is_valid(&self) -> bool {
        self.file_id != u32::MAX && self.len > 0
    }

    #[inline]
    pub fn end(&self) -> u32 {
        self.start.saturating_add(self.len)
    }

    pub fn contains(&self, other: &SourceSpan) -> bool {
        self.file_id == other.file_id && other.start >= self.start && other.end() <= self.end()
    }

    pub fn merge(&self, other: &SourceSpan) -> Option<SourceSpan> {
        if self.file_id != other.file_id {
            return None;
        }

        let start = self.start.min(other.start);
        let end = self.end().max(other.end());

        Some(SourceSpan::new(
            self.file_id,
            start,
            end.saturating_sub(start),
        ))
    }

    pub fn merge_all(spans: &[Option<SourceSpan>]) -> Option<SourceSpan> {
        const MAX_SPAN_MERGE: usize = 1000;

        let valid_spans: Vec<_> = spans
            .iter()
            .filter_map(|s| s.filter(|span| span.is_valid()))
            .take(MAX_SPAN_MERGE)
            .collect();

        if valid_spans.is_empty() {
            return None;
        }

        let first = valid_spans[0];
        valid_spans
            .iter()
            .skip(1)
            .try_fold(first, |acc, span| acc.merge(span))
    }
}

impl Default for SourceSpan {
    fn default() -> Self {
        INVALID_SPAN
    }
}

#[derive(Debug, Clone)]
pub struct SourceFiles {
    files: Arc<RwLock<Vec<SourceFile>>>,
}

impl SourceFiles {
    pub fn new() -> Self {
        Self {
            files: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn add_file(&self, path: PathBuf, text: String) -> Result<u32> {
        if text.len() > MAX_FILE_SIZE {
            return Err(SourceLocationError::FileTooLarge(text.len()));
        }

        let line_starts = compute_line_starts(&text)?;

        let mut files = self.files.write().unwrap();
        let file_id = files.len() as u32;

        files.push(SourceFile {
            path,
            text: Arc::new(text),
            line_starts: Arc::new(line_starts),
        });

        Ok(file_id)
    }

    pub fn get_file(&self, file_id: u32) -> Option<SourceFile> {
        let files = self.files.read().unwrap();
        files.get(file_id as usize).cloned()
    }

    pub fn to_line_col(&self, span: SourceSpan) -> Option<(u32, u32)> {
        if !span.is_valid() {
            return None;
        }

        let file = self.get_file(span.file_id)?;
        let line_starts = file.line_starts.as_ref();

        let line_idx = line_starts
            .partition_point(|&start| start <= span.start as usize)
            .saturating_sub(1);

        if line_idx >= line_starts.len() {
            return None;
        }

        let line_start = line_starts[line_idx];
        let line = (line_idx + 1) as u32;

        let column = file.text[line_start..span.start as usize].chars().count() as u32 + 1;

        Some((line, column))
    }

    pub fn snippet(&self, span: SourceSpan, context_lines: usize) -> Option<String> {
        if !span.is_valid() {
            return None;
        }

        let file = self.get_file(span.file_id)?;
        let (line, _col) = self.to_line_col(span)?;

        let start_line = (line as usize).saturating_sub(context_lines);
        let end_line = (line as usize).saturating_add(context_lines);

        let mut snippet = String::with_capacity(1024);

        for line_no in start_line..=end_line {
            if let Some(line_text) = file.get_line(line_no) {
                snippet.push_str(&format!("{:4} | {}\n", line_no, line_text));
            }
        }

        Some(snippet)
    }

    pub fn relative_path(&self, file_id: u32, workspace_root: &Path) -> Option<PathBuf> {
        let file = self.get_file(file_id)?;

        file.path
            .strip_prefix(workspace_root)
            .ok()
            .map(|p| p.to_path_buf())
            .or_else(|| Some(PathBuf::from(format!("external/file_{}", file_id))))
    }

    pub fn file_count(&self) -> usize {
        self.files.read().unwrap().len()
    }

    pub fn get_all_files(&self) -> Vec<SourceFile> {
        self.files.read().unwrap().clone()
    }
}

impl Default for SourceFiles {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct SourceFile {
    pub path: PathBuf,
    pub text: Arc<String>,
    pub line_starts: Arc<Vec<usize>>,
}

impl SourceFile {
    pub fn get_line(&self, line: usize) -> Option<&str> {
        if line == 0 {
            return None;
        }

        let line_idx = line - 1;
        if line_idx >= self.line_starts.len() {
            return None;
        }

        let start = self.line_starts[line_idx];
        let end = if line_idx + 1 < self.line_starts.len() {
            self.line_starts[line_idx + 1]
        } else {
            self.text.len()
        };

        let text_bytes = self.text.as_bytes();
        let start_boundary = find_char_boundary(text_bytes, start);
        let end_boundary = find_char_boundary(text_bytes, end);

        std::str::from_utf8(&text_bytes[start_boundary..end_boundary])
            .ok()
            .map(|s| s.trim_end_matches(&['\r', '\n'][..]))
    }

    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }
}

fn compute_line_starts(text: &str) -> Result<Vec<usize>> {
    let mut line_starts = Vec::with_capacity(1000);
    line_starts.push(0);

    let bytes = text.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if line_starts.len() >= MAX_LINES {
            return Err(SourceLocationError::TooManyLines(line_starts.len()));
        }

        match bytes[i] {
            b'\r' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
                    i += 2;
                } else {
                    i += 1;
                }
                line_starts.push(i);
            }
            b'\n' => {
                i += 1;
                line_starts.push(i);
            }
            _ => {
                i += 1;
            }
        }
    }

    Ok(line_starts)
}

fn find_char_boundary(bytes: &[u8], mut index: usize) -> usize {
    if index >= bytes.len() {
        return bytes.len();
    }

    while index > 0 && (bytes[index] & 0b1100_0000) == 0b1000_0000 {
        index -= 1;
    }

    index
}

#[derive(Debug, Clone)]
pub struct SourceInfo {
    pub file_id: u32,
    pub path: PathBuf,
    pub line: u32,
    pub column: u32,
    pub length: u32,
}

impl SourceFiles {
    pub fn get_source_info(&self, span: SourceSpan) -> Option<SourceInfo> {
        let (line, column) = self.to_line_col(span)?;
        let file = self.get_file(span.file_id)?;

        Some(SourceInfo {
            file_id: span.file_id,
            path: file.path.clone(),
            line,
            column,
            length: span.len,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_span_basics() {
        let span = SourceSpan::new(0, 10, 20);
        assert!(span.is_valid());
        assert_eq!(span.end(), 30);

        assert_eq!(INVALID_SPAN.is_valid(), false);
    }

    #[test]
    fn test_span_merge() {
        let span1 = SourceSpan::new(0, 10, 5);
        let span2 = SourceSpan::new(0, 20, 5);

        let merged = span1.merge(&span2).unwrap();
        assert_eq!(merged.start, 10);
        assert_eq!(merged.len, 15);

        let span3 = SourceSpan::new(1, 10, 5);
        assert!(span1.merge(&span3).is_none());
    }

    #[test]
    fn test_span_contains() {
        let outer = SourceSpan::new(0, 10, 20);
        let inner = SourceSpan::new(0, 15, 5);

        assert!(outer.contains(&inner));
        assert!(!inner.contains(&outer));
    }

    #[test]
    fn test_line_starts_unix() {
        let text = "line 1\nline 2\nline 3";
        let line_starts = compute_line_starts(text).unwrap();
        assert_eq!(line_starts, vec![0, 7, 14]);
    }

    #[test]
    fn test_line_starts_windows() {
        let text = "line 1\r\nline 2\r\nline 3";
        let line_starts = compute_line_starts(text).unwrap();
        assert_eq!(line_starts, vec![0, 8, 16]);
    }

    #[test]
    fn test_line_starts_mixed() {
        let text = "line 1\nline 2\r\nline 3\rline 4";
        let line_starts = compute_line_starts(text).unwrap();
        assert_eq!(line_starts, vec![0, 7, 15, 22]);
    }

    #[test]
    fn test_to_line_col() {
        let files = SourceFiles::new();
        let file_id = files
            .add_file(
                PathBuf::from("test.sol"),
                "line 1\nline 2\nline 3".to_string(),
            )
            .unwrap();

        let span = SourceSpan::new(file_id, 7, 6);
        let (line, col) = files.to_line_col(span).unwrap();
        assert_eq!(line, 2);
        assert_eq!(col, 1);

        let span = SourceSpan::new(file_id, 10, 1);
        let (line, col) = files.to_line_col(span).unwrap();
        assert_eq!(line, 2);
        assert_eq!(col, 4);
    }

    #[test]
    fn test_to_line_col_zero_offset() {
        let files = SourceFiles::new();
        let file_id = files
            .add_file(PathBuf::from("test.sol"), "hello\nworld".to_string())
            .unwrap();

        let span = SourceSpan::new(file_id, 0, 5);
        let (line, col) = files.to_line_col(span).unwrap();
        assert_eq!(line, 1);
        assert_eq!(col, 1);
    }

    #[test]
    fn test_snippet_extraction() {
        let files = SourceFiles::new();
        let file_id = files
            .add_file(
                PathBuf::from("test.sol"),
                "line 1\nline 2\nline 3\nline 4\nline 5".to_string(),
            )
            .unwrap();

        let span = SourceSpan::new(file_id, 7, 6);
        let snippet = files.snippet(span, 1).unwrap();

        assert!(snippet.contains("1 | line 1"));
        assert!(snippet.contains("2 | line 2"));
        assert!(snippet.contains("3 | line 3"));
        assert!(!snippet.contains("4 | line 4"));
    }

    #[test]
    fn test_unicode_handling() {
        let files = SourceFiles::new();
        let text = "hello 👨‍👩‍👧‍👦 world";
        let file_id = files
            .add_file(PathBuf::from("unicode.sol"), text.to_string())
            .unwrap();

        let span = SourceSpan::new(file_id, 6, 25);
        let (line, col) = files.to_line_col(span).unwrap();
        assert_eq!(line, 1);
        assert!(col > 1);
    }

    #[test]
    fn test_resource_limits() {
        let huge_text = "x\n".repeat(MAX_LINES + 1);
        let result = compute_line_starts(&huge_text);
        assert!(matches!(result, Err(SourceLocationError::TooManyLines(_))));
    }

    #[test]
    fn test_get_line() {
        let files = SourceFiles::new();
        let file_id = files
            .add_file(
                PathBuf::from("test.sol"),
                "line 1\nline 2\nline 3".to_string(),
            )
            .unwrap();

        let file = files.get_file(file_id).unwrap();

        assert_eq!(file.get_line(1).unwrap(), "line 1");
        assert_eq!(file.get_line(2).unwrap(), "line 2");
        assert_eq!(file.get_line(3).unwrap(), "line 3");
        assert!(file.get_line(4).is_none());
        assert!(file.get_line(0).is_none());
    }

    #[test]
    fn test_thread_safety() {
        use std::thread;

        let files = SourceFiles::new();
        let file_id = files
            .add_file(
                PathBuf::from("test.sol"),
                "line 1\nline 2\nline 3".to_string(),
            )
            .unwrap();

        let handles: Vec<_> = (0..10)
            .map(|i| {
                let files_clone = files.clone();
                thread::spawn(move || {
                    let span = SourceSpan::new(file_id, (i % 3) * 7, 6);
                    files_clone.to_line_col(span)
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_relative_path_anonymization() {
        let files = SourceFiles::new();
        let file_id = files
            .add_file(
                PathBuf::from("/home/user/secret/contract.sol"),
                "test".to_string(),
            )
            .unwrap();

        let workspace = Path::new("/home/user/project");
        let relative = files.relative_path(file_id, workspace).unwrap();

        assert!(relative.to_string_lossy().contains("external"));
    }
}
