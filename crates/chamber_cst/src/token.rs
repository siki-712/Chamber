//! CST token type.

use chamber_syntax::{SyntaxKind, Trivia};
use chamber_text_size::TextRange;

/// A terminal token in the CST.
///
/// Tokens are the leaves of the syntax tree. Each token carries:
/// - Its kind (what type of token it is)
/// - Its source range (where it appears in the source)
/// - Leading trivia (whitespace/comments before the token)
/// - Trailing trivia (whitespace/comments after the token)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CstToken {
    /// The kind of this token.
    kind: SyntaxKind,
    /// The source range of the token text (excluding trivia).
    range: TextRange,
    /// Trivia before this token.
    leading_trivia: Vec<Trivia>,
    /// Trivia after this token.
    trailing_trivia: Vec<Trivia>,
}

impl CstToken {
    /// Creates a new token with no trivia.
    pub fn new(kind: SyntaxKind, range: TextRange) -> Self {
        debug_assert!(kind.is_token(), "CstToken must have a token kind");
        Self {
            kind,
            range,
            leading_trivia: Vec::new(),
            trailing_trivia: Vec::new(),
        }
    }

    /// Creates a new token with trivia.
    pub fn with_trivia(
        kind: SyntaxKind,
        range: TextRange,
        leading_trivia: Vec<Trivia>,
        trailing_trivia: Vec<Trivia>,
    ) -> Self {
        debug_assert!(kind.is_token(), "CstToken must have a token kind");
        Self {
            kind,
            range,
            leading_trivia,
            trailing_trivia,
        }
    }

    /// Returns the kind of this token.
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        self.kind
    }

    /// Returns the source range of the token text (excluding trivia).
    #[inline]
    pub fn range(&self) -> TextRange {
        self.range
    }

    /// Returns the full range including leading and trailing trivia.
    pub fn full_range(&self) -> TextRange {
        let start = self
            .leading_trivia
            .first()
            .map(|t| t.range.start())
            .unwrap_or_else(|| self.range.start());
        let end = self
            .trailing_trivia
            .last()
            .map(|t| t.range.end())
            .unwrap_or_else(|| self.range.end());
        TextRange::new(start, end)
    }

    /// Returns the leading trivia.
    #[inline]
    pub fn leading_trivia(&self) -> &[Trivia] {
        &self.leading_trivia
    }

    /// Returns the trailing trivia.
    #[inline]
    pub fn trailing_trivia(&self) -> &[Trivia] {
        &self.trailing_trivia
    }

    /// Returns true if this token has any leading trivia.
    #[inline]
    pub fn has_leading_trivia(&self) -> bool {
        !self.leading_trivia.is_empty()
    }

    /// Returns true if this token has any trailing trivia.
    #[inline]
    pub fn has_trailing_trivia(&self) -> bool {
        !self.trailing_trivia.is_empty()
    }

    /// Returns the token text from the source.
    pub fn text<'a>(&self, source: &'a str) -> &'a str {
        let start: u32 = self.range.start().into();
        let end: u32 = self.range.end().into();
        &source[start as usize..end as usize]
    }

    /// Adds leading trivia to this token.
    pub fn add_leading_trivia(&mut self, trivia: Trivia) {
        self.leading_trivia.push(trivia);
    }

    /// Adds trailing trivia to this token.
    pub fn add_trailing_trivia(&mut self, trivia: Trivia) {
        self.trailing_trivia.push(trivia);
    }

    /// Sets the leading trivia.
    pub fn set_leading_trivia(&mut self, trivia: Vec<Trivia>) {
        self.leading_trivia = trivia;
    }

    /// Sets the trailing trivia.
    pub fn set_trailing_trivia(&mut self, trivia: Vec<Trivia>) {
        self.trailing_trivia = trivia;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chamber_text_size::TextSize;

    #[test]
    fn test_token_creation() {
        let range = TextRange::new(TextSize::from(0), TextSize::from(1));
        let token = CstToken::new(SyntaxKind::NOTE_NAME, range);

        assert_eq!(token.kind(), SyntaxKind::NOTE_NAME);
        assert_eq!(token.range(), range);
        assert!(!token.has_leading_trivia());
        assert!(!token.has_trailing_trivia());
    }

    #[test]
    fn test_token_text() {
        let source = "C D E";
        let range = TextRange::new(TextSize::from(0), TextSize::from(1));
        let token = CstToken::new(SyntaxKind::NOTE_NAME, range);

        assert_eq!(token.text(source), "C");
    }

    #[test]
    fn test_token_with_trivia() {
        let source = "  C  ";
        let leading = vec![Trivia::new(
            SyntaxKind::WHITESPACE,
            TextRange::new(TextSize::from(0), TextSize::from(2)),
        )];
        let trailing = vec![Trivia::new(
            SyntaxKind::WHITESPACE,
            TextRange::new(TextSize::from(3), TextSize::from(5)),
        )];
        let token = CstToken::with_trivia(
            SyntaxKind::NOTE_NAME,
            TextRange::new(TextSize::from(2), TextSize::from(3)),
            leading,
            trailing,
        );

        assert_eq!(token.text(source), "C");
        assert!(token.has_leading_trivia());
        assert!(token.has_trailing_trivia());
        assert_eq!(
            token.full_range(),
            TextRange::new(TextSize::from(0), TextSize::from(5))
        );
    }
}
