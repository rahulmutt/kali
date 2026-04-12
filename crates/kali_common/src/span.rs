/// Span representation for source positions.
use super::FileId;

/// A source code span identifying a range in the input.
///
/// Spans are cheap to copy and are used throughout the compiler to track
/// source positions in AST nodes and IR.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, serde::Serialize, serde::Deserialize)]
pub struct Span {
    /// File ID where this span originates.
    pub file_id: FileId,
    /// Start byte position (0-indexed).
    pub start: u32,
    /// End byte position (0-indexed, exclusive).
    pub end: u32,
}

impl Span {
    /// Create a new span from a file ID and byte positions.
    pub const fn new(file_id: FileId, start: u32, end: u32) -> Self {
        Self {
            file_id,
            start,
            end,
        }
    }

    /// Create a span at a single byte position (zero-width).
    pub const fn from_position(file_id: FileId, pos: u32) -> Self {
        Self {
            file_id,
            start: pos,
            end: pos,
        }
    }

    /// Get the length of this span in bytes.
    pub const fn len(&self) -> u32 {
        self.end - self.start
    }

    /// Check if this span is empty.
    pub const fn is_empty(&self) -> bool {
        self.end == self.start
    }

    /// Get the start position of this span.
    pub const fn start(&self) -> u32 {
        self.start
    }

    /// Get the end position of this span.
    pub const fn end(&self) -> u32 {
        self.end
    }

    /// Get the file ID of this span.
    pub const fn file_id(&self) -> FileId {
        self.file_id
    }

    /// Check if this span covers the given position.
    pub fn contains(&self, pos: u32) -> bool {
        pos >= self.start && pos < self.end
    }

    /// Get the line and column information for this span.
    ///
    /// This requires access to the source text. When source is available,
    /// this computes the line/column of the start position.
    pub fn location_info(&self, source: &str) -> Option<LocationInfo> {
        let src = &source[self.start as usize..self.end as usize];

        let line = source[..self.start as usize]
            .chars()
            .filter(|&c| c == '\n')
            .count()
            + 1;

        let column = source[..self.start as usize]
            .split('\n')
            .last()
            .map(|l| l.chars().count() + 1)
            .unwrap_or(1);

        let multi_line = src.chars().any(|c| c == '\n');

        Some(LocationInfo {
            line,
            column,
            multi_line,
        })
    }

    /// Return a self-referential span (alias).
    #[must_use]
    pub const fn alias(&self) -> Self {
        *self
    }
}

impl Default for Span {
    fn default() -> Self {
        Self::new(FileId::default(), 0, 0)
    }
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.file_id, self.start, self.end)
    }
}

/// Line and column information for a span.
#[derive(Debug, Clone, Copy)]
pub struct LocationInfo {
    /// 1-indexed line number.
    pub line: usize,
    /// 1-indexed column number.
    pub column: usize,
    /// Whether this spans multiple lines.
    pub multi_line: bool,
}

impl LocationInfo {
    /// Format this location information for display.
    pub fn display(&self) -> String {
        if self.multi_line {
            format!("line {}:{}", self.line, self.column)
        } else {
            format!("line {}:{}", self.line, self.column)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_creation() {
        let file_id = FileId::new(0);
        let span = Span::new(file_id, 10, 20);

        assert_eq!(span.start(), 10);
        assert_eq!(span.end(), 20);
        assert_eq!(span.len(), 10);
        assert!(!span.is_empty());
    }

    #[test]
    fn test_span_position() {
        let span = Span::from_position(FileId::new(0), 42);

        assert_eq!(span.start(), 42);
        assert_eq!(span.end(), 42);
        assert!(span.is_empty());
    }

    #[test]
    fn test_span_contains() {
        let span = Span::new(FileId::new(0), 10, 20);

        assert!(span.contains(10));
        assert!(span.contains(15));
        assert!(span.contains(19));
        assert!(!span.contains(9));
        assert!(!span.contains(20));
    }

    #[test]
    fn test_empty_span() {
        let span = Span::new(FileId::new(0), 42, 42);
        assert!(span.is_empty());
        assert_eq!(span.len(), 0);
    }
}
