//! CST printing for lossless round-trip.

use crate::{CstChild, CstNode, CstToken};

/// Prints a CST back to source code.
///
/// This function reconstructs the original source by traversing the CST
/// and outputting all tokens including their trivia.
pub fn print_cst(node: &CstNode, source: &str) -> String {
    let mut output = String::new();
    print_node(node, source, &mut output);
    output
}

fn print_node(node: &CstNode, source: &str, output: &mut String) {
    for child in node.children() {
        print_child(child, source, output);
    }
}

fn print_child(child: &CstChild, source: &str, output: &mut String) {
    match child {
        CstChild::Node(node) => print_node(node, source, output),
        CstChild::Token(token) => print_token(token, source, output),
    }
}

fn print_token(token: &CstToken, source: &str, output: &mut String) {
    // Print leading trivia
    for trivia in token.leading_trivia() {
        output.push_str(trivia.text(source));
    }

    // Print the token itself
    output.push_str(token.text(source));

    // Print trailing trivia
    for trivia in token.trailing_trivia() {
        output.push_str(trivia.text(source));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chamber_syntax::{SyntaxKind, Trivia};
    use chamber_text_size::{TextRange, TextSize};

    fn make_range(start: u32, end: u32) -> TextRange {
        TextRange::new(TextSize::from(start), TextSize::from(end))
    }

    #[test]
    fn test_print_simple_token() {
        let source = "C";
        let token = CstToken::new(SyntaxKind::NOTE_NAME, make_range(0, 1));
        let node = CstNode::with_children(SyntaxKind::NOTE, vec![CstChild::Token(token)]);

        let output = print_cst(&node, source);
        assert_eq!(output, "C");
    }

    #[test]
    fn test_print_token_with_trivia() {
        let source = "  C  ";
        let leading = vec![Trivia::new(SyntaxKind::WHITESPACE, make_range(0, 2))];
        let trailing = vec![Trivia::new(SyntaxKind::WHITESPACE, make_range(3, 5))];
        let token = CstToken::with_trivia(
            SyntaxKind::NOTE_NAME,
            make_range(2, 3),
            leading,
            trailing,
        );
        let node = CstNode::with_children(SyntaxKind::NOTE, vec![CstChild::Token(token)]);

        let output = print_cst(&node, source);
        assert_eq!(output, "  C  ");
    }

    #[test]
    fn test_print_nested_nodes() {
        let source = "^C";
        let sharp = CstToken::new(SyntaxKind::SHARP, make_range(0, 1));
        let note = CstToken::new(SyntaxKind::NOTE_NAME, make_range(1, 2));

        let accidental_node = CstNode::with_children(
            SyntaxKind::ACCIDENTAL,
            vec![CstChild::Token(sharp)],
        );

        let note_node = CstNode::with_children(
            SyntaxKind::NOTE,
            vec![
                CstChild::Node(accidental_node),
                CstChild::Token(note),
            ],
        );

        let output = print_cst(&note_node, source);
        assert_eq!(output, "^C");
    }

    #[test]
    fn test_roundtrip_with_whitespace() {
        let source = "C D E";
        // Build CST matching this source
        let c_token = CstToken::new(SyntaxKind::NOTE_NAME, make_range(0, 1));
        let d_token = CstToken::with_trivia(
            SyntaxKind::NOTE_NAME,
            make_range(2, 3),
            vec![Trivia::new(SyntaxKind::WHITESPACE, make_range(1, 2))],
            vec![],
        );
        let e_token = CstToken::with_trivia(
            SyntaxKind::NOTE_NAME,
            make_range(4, 5),
            vec![Trivia::new(SyntaxKind::WHITESPACE, make_range(3, 4))],
            vec![],
        );

        let body = CstNode::with_children(
            SyntaxKind::BODY,
            vec![
                CstChild::Token(c_token),
                CstChild::Token(d_token),
                CstChild::Token(e_token),
            ],
        );

        let output = print_cst(&body, source);
        assert_eq!(output, source);
    }
}
