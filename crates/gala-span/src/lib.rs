//! Source spans, file database, and string interning for Gala.

pub use lasso::Spur;
use lasso::Rodeo;
use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;

/// A unique identifier for a source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileId(pub u32);

impl FileId {
    pub fn new(id: u32) -> Self {
        FileId(id)
    }

    pub fn as_u32(self) -> u32 {
        self.0
    }
}

impl fmt::Display for FileId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "file#{}", self.0)
    }
}

/// A byte span in a source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ByteSpan {
    pub start: u32,
    pub end: u32,
}

impl ByteSpan {
    pub fn new(start: u32, end: u32) -> Self {
        ByteSpan { start, end }
    }

    pub fn len(&self) -> u32 {
        self.end - self.start
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

impl fmt::Display for ByteSpan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

/// A span with an associated file ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub file_id: FileId,
    pub byte_span: ByteSpan,
}

impl Span {
    pub fn new(file_id: FileId, byte_span: ByteSpan) -> Self {
        Span { file_id, byte_span }
    }

    pub fn dummy() -> Self {
        Span {
            file_id: FileId(0),
            byte_span: ByteSpan { start: 0, end: 0 },
        }
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.file_id, self.byte_span)
    }
}

/// A source file with content.
#[derive(Debug, Clone)]
pub struct SourceFile {
    pub id: FileId,
    pub path: PathBuf,
    pub content: Arc<str>,
}

/// Source map storing all source files.
#[derive(Debug, Clone, Default)]
pub struct SourceMap {
    files: Vec<SourceFile>,
}

impl SourceMap {
    pub fn new() -> Self {
        SourceMap { files: Vec::new() }
    }

    pub fn add_file(&mut self, path: PathBuf, content: String) -> FileId {
        let id = FileId(self.files.len() as u32);
        let file = SourceFile {
            id,
            path,
            content: content.into(),
        };
        self.files.push(file);
        id
    }

    pub fn get_file(&self, file_id: FileId) -> Option<&SourceFile> {
        self.files.get(file_id.0 as usize)
    }

    pub fn get_content(&self, file_id: FileId) -> Option<Arc<str>> {
        self.files.get(file_id.0 as usize).map(|f| f.content.clone())
    }

    pub fn span_to_line_col(&self, file_id: FileId, byte_offset: u32) -> Option<(usize, usize)> {
        let content = self.get_content(file_id)?;
        let content_str = content.as_ref();
        let mut line = 1;
        let mut col = 1;
        for (i, ch) in content_str.char_indices() {
            if i >= byte_offset as usize {
                break;
            }
            if ch == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }
        Some((line, col))
    }
}

/// String interner using lasso.
#[derive(Debug, Clone, Default)]
pub struct Interner {
    inner: Rodeo,
}

impl Interner {
    pub fn new() -> Self {
        Interner {
            inner: Rodeo::default(),
        }
    }

    pub fn get_or_intern(&mut self, s: &str) -> Spur {
        self.inner.get_or_intern(s)
    }

    pub fn resolve(&self, spur: Spur) -> Option<&str> {
        Some(self.inner.resolve(&spur))
    }
}

/// Re-export Spur for convenience

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_byte_span() {
        let span = ByteSpan::new(0, 10);
        assert_eq!(span.len(), 10);
        assert!(!span.is_empty());

        let empty = ByteSpan::new(5, 5);
        assert_eq!(empty.len(), 0);
        assert!(empty.is_empty());
    }

    #[test]
    fn test_source_map() {
        let mut map = SourceMap::new();
        let id = map.add_file("test.gala".into(), "fn main() { return 42; }".to_string());
        assert_eq!(id.0, 0);

        let file = map.get_file(id).unwrap();
        assert_eq!(file.path.to_str(), Some("test.gala"));
        assert_eq!(file.content.as_ref(), "fn main() { return 42; }");
    }

    #[test]
    fn test_interner() {
        let mut interner = Interner::new();
        let spur1 = interner.get_or_intern("hello");
        let spur2 = interner.get_or_intern("hello");
        assert_eq!(spur1, spur2);

        let spur3 = interner.get_or_intern("world");
        assert_ne!(spur1, spur3);

        assert_eq!(interner.resolve(spur1), Some("hello"));
    }
}