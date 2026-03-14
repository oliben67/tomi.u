//! Source span tracking for error reporting
//!
//! Provides [`Span`] for tracking source locations and [`Spanned`] for
//! wrapping AST nodes with location information.

use std::fmt;
use std::ops::Range;

/// A span representing a range in source code.
///
/// Spans are byte offsets into the source string, suitable for
/// use with error reporting libraries like ariadne.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Span {
    /// Start byte offset (inclusive)
    pub start: usize,
    /// End byte offset (exclusive)
    pub end: usize,
}

impl Span {
    /// Create a new span from start and end byte offsets.
    #[inline]
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Create a span covering a single byte position.
    #[inline]
    pub const fn point(pos: usize) -> Self {
        Self {
            start: pos,
            end: pos + 1,
        }
    }

    /// Create a span that covers both self and other.
    #[inline]
    pub fn merge(self, other: Span) -> Self {
        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    /// Check if this span contains a byte offset.
    #[inline]
    pub fn contains(&self, offset: usize) -> bool {
        offset >= self.start && offset < self.end
    }

    /// Get the length of this span in bytes.
    #[inline]
    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    /// Check if span is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }

    /// Convert to a Range for slicing.
    #[inline]
    pub fn as_range(&self) -> Range<usize> {
        self.start..self.end
    }

    /// Create a dummy span (used for generated code).
    pub const DUMMY: Span = Span { start: 0, end: 0 };
}

impl fmt::Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

impl From<Range<usize>> for Span {
    fn from(range: Range<usize>) -> Self {
        Self {
            start: range.start,
            end: range.end,
        }
    }
}

impl From<Span> for Range<usize> {
    fn from(span: Span) -> Self {
        span.start..span.end
    }
}

// For ariadne integration
impl ariadne::Span for Span {
    type SourceId = ();

    fn source(&self) -> &Self::SourceId {
        &()
    }

    fn start(&self) -> usize {
        self.start
    }

    fn end(&self) -> usize {
        self.end
    }
}

/// A value paired with its source span.
///
/// This is used throughout the AST to track where each node came from
/// in the source code, enabling precise error messages.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Spanned<T> {
    /// The wrapped value
    pub node: T,
    /// Where this value appears in source
    pub span: Span,
}

impl<T> Spanned<T> {
    /// Create a new spanned value.
    #[inline]
    pub fn new(node: T, span: Span) -> Self {
        Self { node, span }
    }

    /// Map the inner value while preserving the span.
    #[inline]
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Spanned<U> {
        Spanned {
            node: f(self.node),
            span: self.span,
        }
    }

    /// Get a reference to the inner value.
    #[inline]
    pub fn as_ref(&self) -> Spanned<&T> {
        Spanned {
            node: &self.node,
            span: self.span,
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for Spanned<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} @ {:?}", self.node, self.span)
    }
}

impl<T: fmt::Display> fmt::Display for Spanned<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.node)
    }
}

impl<T> std::ops::Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.node
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn span_merge() {
        let a = Span::new(5, 10);
        let b = Span::new(15, 20);
        let merged = a.merge(b);
        assert_eq!(merged, Span::new(5, 20));
    }

    #[test]
    fn span_contains() {
        let span = Span::new(10, 20);
        assert!(!span.contains(5));
        assert!(span.contains(10));
        assert!(span.contains(15));
        assert!(!span.contains(20));
    }
}
