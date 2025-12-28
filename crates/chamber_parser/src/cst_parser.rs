//! CST parser for ABC notation.
//!
//! This parser produces a lossless Concrete Syntax Tree that preserves
//! all source information including whitespace and comments.

use chamber_cst::{CstChild, CstNode, CstToken};
use chamber_lexer::tokenize_cst;
use chamber_syntax::SyntaxKind;

/// Parses source into a CST.
pub fn parse_cst(source: &str) -> CstNode {
    let tokens = tokenize_cst(source);
    let mut parser = CstParser::new(tokens);
    parser.parse_tune()
}

/// CST parser state.
struct CstParser {
    tokens: Vec<CstToken>,
    position: usize,
}

impl CstParser {
    fn new(tokens: Vec<CstToken>) -> Self {
        Self { tokens, position: 0 }
    }

    // === Navigation ===

    fn is_at_end(&self) -> bool {
        self.position >= self.tokens.len()
    }

    fn current(&self) -> Option<&CstToken> {
        self.tokens.get(self.position)
    }

    fn current_kind(&self) -> Option<SyntaxKind> {
        self.current().map(|t| t.kind())
    }

    fn advance(&mut self) -> Option<CstToken> {
        if self.is_at_end() {
            None
        } else {
            let token = self.tokens[self.position].clone();
            self.position += 1;
            Some(token)
        }
    }

    fn check(&self, kind: SyntaxKind) -> bool {
        self.current_kind() == Some(kind)
    }

    fn check_any(&self, kinds: &[SyntaxKind]) -> bool {
        self.current_kind().map_or(false, |k| kinds.contains(&k))
    }

    fn eat(&mut self, kind: SyntaxKind) -> Option<CstToken> {
        if self.check(kind) {
            self.advance()
        } else {
            None
        }
    }

    // === Parsing ===

    fn parse_tune(&mut self) -> CstNode {
        let mut children = Vec::new();

        // Parse header
        let header = self.parse_header();
        children.push(CstChild::Node(header));

        // Parse body
        let body = self.parse_body();
        children.push(CstChild::Node(body));

        CstNode::with_children(SyntaxKind::TUNE, children)
    }

    fn parse_header(&mut self) -> CstNode {
        let mut children = Vec::new();

        // Parse header fields until we hit K: (key) or body content
        while !self.is_at_end() {
            if self.check(SyntaxKind::FIELD_LABEL) {
                let field = self.parse_header_field();
                children.push(CstChild::Node(field));
                // Note: In a full implementation, we'd check if this was K: field
                // and stop parsing header. For now, we continue until non-header content.
            } else {
                // Not a header field - we've reached the body
                break;
            }
        }

        CstNode::with_children(SyntaxKind::HEADER, children)
    }

    fn parse_header_field(&mut self) -> CstNode {
        let mut children = Vec::new();

        // Field label (X, T, M, K, etc.)
        if let Some(label) = self.eat(SyntaxKind::FIELD_LABEL) {
            children.push(CstChild::Token(label));
        }

        // Colon
        if let Some(colon) = self.eat(SyntaxKind::COLON) {
            children.push(CstChild::Token(colon));
        }

        // Field value (TEXT tokens until newline)
        while !self.is_at_end() {
            // TEXT token contains the value
            if let Some(text) = self.eat(SyntaxKind::TEXT) {
                children.push(CstChild::Token(text));
                break; // TEXT consumes until newline
            } else {
                break;
            }
        }

        CstNode::with_children(SyntaxKind::HEADER_FIELD, children)
    }

    fn parse_body(&mut self) -> CstNode {
        let mut children = Vec::new();

        while !self.is_at_end() {
            if let Some(element) = self.parse_music_element() {
                children.push(element);
            } else {
                // Skip unknown token
                if let Some(token) = self.advance() {
                    children.push(CstChild::Token(token));
                }
            }
        }

        CstNode::with_children(SyntaxKind::BODY, children)
    }

    fn parse_music_element(&mut self) -> Option<CstChild> {
        let kind = self.current_kind()?;

        match kind {
            // Notes with optional accidentals
            SyntaxKind::SHARP | SyntaxKind::NATURAL | SyntaxKind::FLAT | SyntaxKind::NOTE_NAME => {
                Some(CstChild::Node(self.parse_note()))
            }

            // Rest
            SyntaxKind::REST => Some(CstChild::Node(self.parse_rest())),

            // Bar lines
            SyntaxKind::BAR
            | SyntaxKind::DOUBLE_BAR
            | SyntaxKind::REPEAT_START
            | SyntaxKind::REPEAT_END
            | SyntaxKind::THIN_THICK_BAR
            | SyntaxKind::THICK_THIN_BAR => Some(CstChild::Node(self.parse_bar_line())),

            // Chord
            SyntaxKind::L_BRACKET => Some(CstChild::Node(self.parse_chord_or_inline_field())),

            // Slur
            SyntaxKind::L_PAREN => Some(CstChild::Node(self.parse_slur())),

            // Grace notes
            SyntaxKind::L_BRACE => Some(CstChild::Node(self.parse_grace_notes())),

            // Tuplet
            SyntaxKind::TUPLET_MARKER => Some(CstChild::Node(self.parse_tuplet())),

            // Decoration
            SyntaxKind::DECORATION => {
                let token = self.advance()?;
                Some(CstChild::Node(CstNode::with_children(
                    SyntaxKind::DECORATION_NODE,
                    vec![CstChild::Token(token)],
                )))
            }

            // Tie
            SyntaxKind::TIE => {
                let token = self.advance()?;
                Some(CstChild::Node(CstNode::with_children(
                    SyntaxKind::TIE_NODE,
                    vec![CstChild::Token(token)],
                )))
            }

            // Broken rhythm
            SyntaxKind::BROKEN_RHYTHM => {
                let token = self.advance()?;
                Some(CstChild::Node(CstNode::with_children(
                    SyntaxKind::BROKEN_RHYTHM_NODE,
                    vec![CstChild::Token(token)],
                )))
            }

            // Skip other tokens (they'll be handled by the caller)
            _ => None,
        }
    }

    fn parse_note(&mut self) -> CstNode {
        let mut children = Vec::new();

        // Decorations before note
        while self.check(SyntaxKind::DECORATION) {
            if let Some(dec) = self.advance() {
                let dec_node =
                    CstNode::with_children(SyntaxKind::DECORATION_NODE, vec![CstChild::Token(dec)]);
                children.push(CstChild::Node(dec_node));
            }
        }

        // Accidentals (^, ^^, =, _, __)
        if self.check_any(&[SyntaxKind::SHARP, SyntaxKind::NATURAL, SyntaxKind::FLAT]) {
            let acc_node = self.parse_accidental();
            children.push(CstChild::Node(acc_node));
        }

        // Note name
        if let Some(note) = self.eat(SyntaxKind::NOTE_NAME) {
            children.push(CstChild::Token(note));
        }

        // Octave modifiers
        while self.check_any(&[SyntaxKind::OCTAVE_UP, SyntaxKind::OCTAVE_DOWN]) {
            if let Some(oct) = self.advance() {
                children.push(CstChild::Token(oct));
            }
        }

        // Duration
        if self.check_any(&[SyntaxKind::NUMBER, SyntaxKind::SLASH]) {
            let dur = self.parse_duration();
            children.push(CstChild::Node(dur));
        }

        CstNode::with_children(SyntaxKind::NOTE, children)
    }

    fn parse_accidental(&mut self) -> CstNode {
        let mut children = Vec::new();

        // Consume accidental tokens (can be double: ^^ or __)
        while self.check_any(&[SyntaxKind::SHARP, SyntaxKind::NATURAL, SyntaxKind::FLAT]) {
            if let Some(acc) = self.advance() {
                children.push(CstChild::Token(acc));
            }
        }

        CstNode::with_children(SyntaxKind::ACCIDENTAL, children)
    }

    fn parse_duration(&mut self) -> CstNode {
        let mut children = Vec::new();

        // Numerator (optional)
        if let Some(num) = self.eat(SyntaxKind::NUMBER) {
            children.push(CstChild::Token(num));
        }

        // Slash and denominator (optional)
        if let Some(slash) = self.eat(SyntaxKind::SLASH) {
            children.push(CstChild::Token(slash));
            if let Some(denom) = self.eat(SyntaxKind::NUMBER) {
                children.push(CstChild::Token(denom));
            }
        }

        CstNode::with_children(SyntaxKind::DURATION, children)
    }

    fn parse_rest(&mut self) -> CstNode {
        let mut children = Vec::new();

        // Decorations before rest
        while self.check(SyntaxKind::DECORATION) {
            if let Some(dec) = self.advance() {
                let dec_node =
                    CstNode::with_children(SyntaxKind::DECORATION_NODE, vec![CstChild::Token(dec)]);
                children.push(CstChild::Node(dec_node));
            }
        }

        // Rest token
        if let Some(rest) = self.eat(SyntaxKind::REST) {
            children.push(CstChild::Token(rest));
        }

        // Duration
        if self.check_any(&[SyntaxKind::NUMBER, SyntaxKind::SLASH]) {
            let dur = self.parse_duration();
            children.push(CstChild::Node(dur));
        }

        CstNode::with_children(SyntaxKind::REST_NODE, children)
    }

    fn parse_bar_line(&mut self) -> CstNode {
        let mut children = Vec::new();

        if let Some(bar) = self.advance() {
            children.push(CstChild::Token(bar));
        }

        CstNode::with_children(SyntaxKind::BAR_LINE, children)
    }

    fn parse_chord_or_inline_field(&mut self) -> CstNode {
        // Peek ahead to determine if this is an inline field [M:3/4] or chord [CEG]
        // For now, simplified: check if second token is FIELD_LABEL
        let is_inline_field = self.tokens.get(self.position + 1).map_or(false, |t| {
            t.kind() == SyntaxKind::FIELD_LABEL || t.kind() == SyntaxKind::TEXT
        });

        if is_inline_field {
            self.parse_inline_field()
        } else {
            self.parse_chord()
        }
    }

    fn parse_chord(&mut self) -> CstNode {
        let mut children = Vec::new();

        // Decorations before chord
        while self.check(SyntaxKind::DECORATION) {
            if let Some(dec) = self.advance() {
                let dec_node =
                    CstNode::with_children(SyntaxKind::DECORATION_NODE, vec![CstChild::Token(dec)]);
                children.push(CstChild::Node(dec_node));
            }
        }

        // Opening bracket
        if let Some(open) = self.eat(SyntaxKind::L_BRACKET) {
            children.push(CstChild::Token(open));
        }

        // Notes inside chord
        while !self.is_at_end() && !self.check(SyntaxKind::R_BRACKET) {
            if self.check_any(&[
                SyntaxKind::SHARP,
                SyntaxKind::NATURAL,
                SyntaxKind::FLAT,
                SyntaxKind::NOTE_NAME,
            ]) {
                let note = self.parse_note();
                children.push(CstChild::Node(note));
            } else {
                break;
            }
        }

        // Closing bracket
        if let Some(close) = self.eat(SyntaxKind::R_BRACKET) {
            children.push(CstChild::Token(close));
        }

        // Duration after chord
        if self.check_any(&[SyntaxKind::NUMBER, SyntaxKind::SLASH]) {
            let dur = self.parse_duration();
            children.push(CstChild::Node(dur));
        }

        CstNode::with_children(SyntaxKind::CHORD, children)
    }

    fn parse_inline_field(&mut self) -> CstNode {
        let mut children = Vec::new();

        // Opening bracket
        if let Some(open) = self.eat(SyntaxKind::L_BRACKET) {
            children.push(CstChild::Token(open));
        }

        // Field label or content until ]
        while !self.is_at_end() && !self.check(SyntaxKind::R_BRACKET) {
            if let Some(token) = self.advance() {
                children.push(CstChild::Token(token));
            }
        }

        // Closing bracket
        if let Some(close) = self.eat(SyntaxKind::R_BRACKET) {
            children.push(CstChild::Token(close));
        }

        CstNode::with_children(SyntaxKind::INLINE_FIELD, children)
    }

    fn parse_slur(&mut self) -> CstNode {
        let mut children = Vec::new();

        // Opening paren
        if let Some(open) = self.eat(SyntaxKind::L_PAREN) {
            children.push(CstChild::Token(open));
        }

        // Elements inside slur
        while !self.is_at_end() && !self.check(SyntaxKind::R_PAREN) {
            if let Some(element) = self.parse_music_element() {
                children.push(element);
            } else if self.check(SyntaxKind::R_PAREN) {
                break;
            } else {
                // Skip unknown token inside slur
                if let Some(token) = self.advance() {
                    children.push(CstChild::Token(token));
                }
            }
        }

        // Closing paren
        if let Some(close) = self.eat(SyntaxKind::R_PAREN) {
            children.push(CstChild::Token(close));
        }

        CstNode::with_children(SyntaxKind::SLUR, children)
    }

    fn parse_grace_notes(&mut self) -> CstNode {
        let mut children = Vec::new();

        // Opening brace
        if let Some(open) = self.eat(SyntaxKind::L_BRACE) {
            children.push(CstChild::Token(open));
        }

        // Notes inside grace notes
        while !self.is_at_end() && !self.check(SyntaxKind::R_BRACE) {
            if self.check_any(&[
                SyntaxKind::SHARP,
                SyntaxKind::NATURAL,
                SyntaxKind::FLAT,
                SyntaxKind::NOTE_NAME,
            ]) {
                let note = self.parse_note();
                children.push(CstChild::Node(note));
            } else {
                break;
            }
        }

        // Closing brace
        if let Some(close) = self.eat(SyntaxKind::R_BRACE) {
            children.push(CstChild::Token(close));
        }

        CstNode::with_children(SyntaxKind::GRACE_NOTES, children)
    }

    fn parse_tuplet(&mut self) -> CstNode {
        let mut children = Vec::new();

        // Tuplet marker (e.g., (3)
        if let Some(marker) = self.eat(SyntaxKind::TUPLET_MARKER) {
            children.push(CstChild::Token(marker));
        }

        // Notes in tuplet (simplified: just collect following notes)
        // In ABC, tuplet applies to the next N notes based on the marker
        let count = 3; // Default for (3
        for _ in 0..count {
            if self.check_any(&[
                SyntaxKind::SHARP,
                SyntaxKind::NATURAL,
                SyntaxKind::FLAT,
                SyntaxKind::NOTE_NAME,
            ]) {
                let note = self.parse_note();
                children.push(CstChild::Node(note));
            } else {
                break;
            }
        }

        CstNode::with_children(SyntaxKind::TUPLET, children)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chamber_cst::print_cst;

    #[test]
    fn test_parse_simple_notes() {
        let source = "X:1\nK:C\nCDE";
        let cst = parse_cst(source);

        assert_eq!(cst.kind(), SyntaxKind::TUNE);
        assert_eq!(cst.child_nodes().count(), 2); // HEADER, BODY
    }

    #[test]
    fn test_roundtrip_simple() {
        let source = "X:1\nK:C\nCDE";
        let cst = parse_cst(source);
        let output = print_cst(&cst, source);

        assert_eq!(source, output);
    }

    #[test]
    fn test_roundtrip_with_whitespace() {
        let source = "X:1\nK:C\nC D E";
        let cst = parse_cst(source);
        let output = print_cst(&cst, source);

        assert_eq!(source, output);
    }

    #[test]
    fn test_roundtrip_with_comment() {
        let source = "X:1 % reference\nK:C\nCDE";
        let cst = parse_cst(source);
        let output = print_cst(&cst, source);

        assert_eq!(source, output);
    }

    #[test]
    fn test_parse_note_with_accidental() {
        let source = "X:1\nK:C\n^C";
        let cst = parse_cst(source);

        let body = cst.find_child_node(SyntaxKind::BODY).unwrap();
        let note = body.find_child_node(SyntaxKind::NOTE).unwrap();

        // Note should have accidental child
        assert!(note.find_child_node(SyntaxKind::ACCIDENTAL).is_some());
    }

    #[test]
    fn test_parse_chord() {
        let source = "X:1\nK:C\n[CEG]";
        let cst = parse_cst(source);

        let body = cst.find_child_node(SyntaxKind::BODY).unwrap();
        let chord = body.find_child_node(SyntaxKind::CHORD).unwrap();

        // Chord should have notes inside
        let notes: Vec<_> = chord.child_nodes().filter(|n| n.kind() == SyntaxKind::NOTE).collect();
        assert_eq!(notes.len(), 3);
    }

    #[test]
    fn test_parse_bar_line() {
        let source = "X:1\nK:C\nC|D";
        let cst = parse_cst(source);

        let body = cst.find_child_node(SyntaxKind::BODY).unwrap();
        let bar = body.find_child_node(SyntaxKind::BAR_LINE).unwrap();

        assert_eq!(bar.kind(), SyntaxKind::BAR_LINE);
    }
}
