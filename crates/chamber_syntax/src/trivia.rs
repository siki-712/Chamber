//! Trivia types for non-semantic syntax elements.

use chamber_text_size::TextRange;

use crate::SyntaxKind;

/// A piece of trivia (whitespace, comment, newline).
///
/// Trivia is attached to tokens and represents non-semantic content
/// that must be preserved for lossless round-tripping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trivia {
    /// The kind of trivia.
    pub kind: SyntaxKind,
    /// The source range of this trivia.
    pub range: TextRange,
}

impl Trivia {
    /// Creates a new trivia piece.
    pub fn new(kind: SyntaxKind, range: TextRange) -> Self {
        debug_assert!(kind.is_trivia(), "Trivia must have a trivia kind");
        Self { kind, range }
    }

    /// Returns the text of this trivia from the source.
    pub fn text<'a>(&self, source: &'a str) -> &'a str {
        let start: u32 = self.range.start().into();
        let end: u32 = self.range.end().into();
        &source[start as usize..end as usize]
    }

    /// Returns true if this is whitespace (spaces/tabs).
    pub fn is_whitespace(&self) -> bool {
        self.kind == SyntaxKind::WHITESPACE
    }

    /// Returns true if this is a newline.
    pub fn is_newline(&self) -> bool {
        self.kind == SyntaxKind::NEWLINE
    }

    /// Returns true if this is a comment.
    pub fn is_comment(&self) -> bool {
        self.kind == SyntaxKind::COMMENT
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chamber_text_size::TextSize;

    #[test]
    fn test_trivia_creation() {
        let range = TextRange::new(TextSize::from(0), TextSize::from(2));
        let trivia = Trivia::new(SyntaxKind::WHITESPACE, range);

        assert!(trivia.is_whitespace());
        assert!(!trivia.is_newline());
        assert!(!trivia.is_comment());
    }

    #[test]
    fn test_trivia_text() {
        let source = "  hello";
        let range = TextRange::new(TextSize::from(0), TextSize::from(2));
        let trivia = Trivia::new(SyntaxKind::WHITESPACE, range);

        assert_eq!(trivia.text(source), "  ");
    }
}
