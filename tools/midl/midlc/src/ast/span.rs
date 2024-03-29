use crate::source_file::SourceId;

/// Represents a location in a MIDL's text representation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub source: SourceId,
    pub data: String,
}

impl Span {
    pub fn from_pest(span: pest::Span<'_>, source: SourceId) -> Self {
        Span {
            start: span.start(),
            end: span.end(),
            source,
            data: span.as_str().to_owned(),
        }
    }

    /// Constructor.
    pub fn new(start: usize, end: usize, source: SourceId) -> Span {
        Span {
            start,
            end,
            source,
            data: String::new(),
        }
    }

    pub fn new_raw(data: &str, source: SourceId) -> Span {
        Span {
            start: 0,
            end: 0,
            source,
            data: data.to_string(),
        }
    }

    /// Creates a new empty span.
    pub fn empty() -> Span {
        Span {
            start: 0,
            end: 0,
            source: SourceId(0),
            data: String::new(),
        }
    }

    /// Is the given position inside the span? (boundaries included)
    pub fn contains(&self, position: usize) -> bool {
        position >= self.start && position <= self.end
    }

    /// Is the given span overlapping with the current span.
    pub fn overlaps(self, other: Span) -> bool {
        self.contains(other.start) || self.contains(other.end)
    }
}
