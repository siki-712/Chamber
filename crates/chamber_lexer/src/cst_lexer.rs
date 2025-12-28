//! CST-aware tokenization with trivia attachment.

use chamber_cst::CstToken;
use chamber_syntax::Trivia;

use crate::{Lexer, Token, TokenKind};

/// Tokenizes source into CST tokens with trivia attached.
///
/// This function converts the raw token stream into CST tokens,
/// attaching whitespace, comments, and newlines as trivia.
///
/// Trivia attachment strategy:
/// - Leading trivia is attached to the first non-trivia token
/// - Trailing trivia (on the same line) is attached to the preceding token
/// - Newlines are treated as trailing trivia
pub fn tokenize_cst(source: &str) -> Vec<CstToken> {
    let lexer = Lexer::new(source);
    let raw_tokens = lexer.tokenize();

    attach_trivia(&raw_tokens)
}

/// Attaches trivia to tokens.
fn attach_trivia(tokens: &[Token]) -> Vec<CstToken> {
    let mut result = Vec::new();
    let mut pending_leading_trivia: Vec<Trivia> = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        let token = &tokens[i];

        // Skip EOF token - it's not a real source token
        if token.kind == TokenKind::Eof {
            i += 1;
            continue;
        }

        if token.kind.is_trivia() {
            // Collect trivia
            let trivia = Trivia::new(token.kind.to_syntax_kind(), token.range);
            pending_leading_trivia.push(trivia);
            i += 1;
            continue;
        }

        // Non-trivia token - create CstToken with leading trivia
        let leading_trivia = std::mem::take(&mut pending_leading_trivia);

        // Collect trailing trivia (same line - whitespace and comments until newline)
        let mut trailing_trivia = Vec::new();
        let mut j = i + 1;
        while j < tokens.len() {
            let next_token = &tokens[j];
            match next_token.kind {
                TokenKind::Whitespace | TokenKind::Comment => {
                    trailing_trivia.push(Trivia::new(
                        next_token.kind.to_syntax_kind(),
                        next_token.range,
                    ));
                    j += 1;
                }
                TokenKind::Newline => {
                    // Include newline as trailing trivia, then stop
                    trailing_trivia.push(Trivia::new(
                        next_token.kind.to_syntax_kind(),
                        next_token.range,
                    ));
                    j += 1;
                    break;
                }
                _ => break,
            }
        }

        let cst_token = CstToken::with_trivia(
            token.kind.to_syntax_kind(),
            token.range,
            leading_trivia,
            trailing_trivia,
        );
        result.push(cst_token);

        i = j;
    }

    // Any remaining leading trivia is attached to the last token as trailing
    // This handles cases like trailing whitespace at end of file
    if !pending_leading_trivia.is_empty() {
        if let Some(last_token) = result.last_mut() {
            for trivia in pending_leading_trivia {
                last_token.add_trailing_trivia(trivia);
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use chamber_syntax::SyntaxKind;
    use chamber_text_size::TextSize;

    #[test]
    fn test_simple_tokens() {
        let source = "CDE";
        let tokens = tokenize_cst(source);

        assert_eq!(tokens.len(), 3); // C, D, E (no EOF because no trailing content)
        assert_eq!(tokens[0].kind(), SyntaxKind::NOTE_NAME);
        assert_eq!(tokens[0].text(source), "C");
        assert!(!tokens[0].has_leading_trivia());
        assert!(!tokens[0].has_trailing_trivia());
    }

    #[test]
    fn test_whitespace_as_trailing() {
        let source = "C D";
        let tokens = tokenize_cst(source);

        // C with trailing space, D
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text(source), "C");
        assert!(tokens[0].has_trailing_trivia());
        assert_eq!(tokens[0].trailing_trivia().len(), 1);
        assert_eq!(tokens[0].trailing_trivia()[0].text(source), " ");

        assert_eq!(tokens[1].text(source), "D");
        assert!(!tokens[1].has_leading_trivia()); // Whitespace attached as trailing to previous
    }

    #[test]
    fn test_newline_as_trailing() {
        let source = "C\nD";
        let tokens = tokenize_cst(source);

        // C with trailing newline, D
        assert_eq!(tokens.len(), 2);
        assert!(tokens[0].has_trailing_trivia());
        assert_eq!(tokens[0].trailing_trivia().len(), 1);
        assert_eq!(tokens[0].trailing_trivia()[0].kind, SyntaxKind::NEWLINE);

        assert!(!tokens[1].has_leading_trivia());
    }

    #[test]
    fn test_comment_as_trailing() {
        let source = "C % comment\nD";
        let tokens = tokenize_cst(source);

        // C with trailing: space, comment, newline
        // D with no trivia
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].trailing_trivia().len(), 3);
        assert_eq!(tokens[0].trailing_trivia()[0].kind, SyntaxKind::WHITESPACE);
        assert_eq!(tokens[0].trailing_trivia()[1].kind, SyntaxKind::COMMENT);
        assert_eq!(tokens[0].trailing_trivia()[2].kind, SyntaxKind::NEWLINE);
    }

    #[test]
    fn test_leading_trivia_at_start() {
        let source = "  C";
        let tokens = tokenize_cst(source);

        // Whitespace at start becomes leading trivia of C
        assert_eq!(tokens.len(), 1);
        assert!(tokens[0].has_leading_trivia());
        assert_eq!(tokens[0].leading_trivia().len(), 1);
        assert_eq!(tokens[0].leading_trivia()[0].text(source), "  ");
    }

    #[test]
    fn test_full_range_includes_trivia() {
        let source = "  C  ";
        let tokens = tokenize_cst(source);

        assert_eq!(tokens.len(), 1);
        // Token range is just "C" (position 2-3)
        assert_eq!(tokens[0].range().start(), TextSize::from(2));
        assert_eq!(tokens[0].range().end(), TextSize::from(3));

        // Full range includes trivia
        assert_eq!(tokens[0].full_range().start(), TextSize::from(0));
        assert_eq!(tokens[0].full_range().end(), TextSize::from(5));
    }

    #[test]
    fn test_header_field() {
        let source = "X:1\n";
        let tokens = tokenize_cst(source);

        // X, :, 1, (newline as trailing of 1)
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].kind(), SyntaxKind::FIELD_LABEL);
        assert_eq!(tokens[1].kind(), SyntaxKind::COLON);
        assert_eq!(tokens[2].kind(), SyntaxKind::TEXT);
        assert!(tokens[2].has_trailing_trivia()); // newline
    }
}
