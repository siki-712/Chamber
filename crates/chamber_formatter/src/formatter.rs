//! Core formatter implementation.

use chamber_cst::{CstChild, CstNode, CstToken};
use chamber_parser::parse_cst;
use chamber_syntax::SyntaxKind;

use crate::FormatterConfig;

/// Formats ABC notation source code.
pub fn format(source: &str, config: &FormatterConfig) -> String {
    let cst = parse_cst(source);
    let mut formatter = Formatter::new(source, config);
    formatter.format_node(&cst);
    formatter.finish()
}

struct Formatter<'a> {
    source: &'a str,
    config: &'a FormatterConfig,
    output: String,
    /// Track if we're in the header section
    in_header: bool,
    /// Track if we just emitted a bar line
    after_bar: bool,
    /// Track if we just emitted a note/chord/rest
    after_note: bool,
    /// Current line content for width tracking
    current_line: String,
}

impl<'a> Formatter<'a> {
    fn new(source: &'a str, config: &'a FormatterConfig) -> Self {
        Self {
            source,
            config,
            output: String::new(),
            in_header: false,
            after_bar: false,
            after_note: false,
            current_line: String::new(),
        }
    }

    fn finish(mut self) -> String {
        if self.config.trim_trailing_whitespace {
            self.output = self
                .output
                .lines()
                .map(|line| line.trim_end())
                .collect::<Vec<_>>()
                .join("\n");
        }

        if self.config.ensure_final_newline && !self.output.ends_with('\n') && !self.output.is_empty() {
            self.output.push('\n');
        }

        self.output
    }

    fn format_node(&mut self, node: &CstNode) {
        match node.kind() {
            SyntaxKind::TUNE => self.format_tune(node),
            SyntaxKind::HEADER => self.format_header(node),
            SyntaxKind::HEADER_FIELD => self.format_header_field(node),
            SyntaxKind::BODY => self.format_body(node),
            SyntaxKind::NOTE => self.format_note(node),
            SyntaxKind::REST_NODE => self.format_rest(node),
            SyntaxKind::CHORD => self.format_chord(node),
            SyntaxKind::BAR_LINE => self.format_bar_line(node),
            SyntaxKind::TUPLET => self.format_tuplet(node),
            SyntaxKind::SLUR => self.format_slur(node),
            SyntaxKind::GRACE_NOTES => self.format_grace_notes(node),
            SyntaxKind::BROKEN_RHYTHM_NODE => self.format_broken_rhythm(node),
            SyntaxKind::TIE_NODE => self.format_tie(node),
            SyntaxKind::INLINE_FIELD => self.format_inline_field(node),
            SyntaxKind::DURATION => self.format_duration(node),
            SyntaxKind::ACCIDENTAL => self.format_accidental(node),
            SyntaxKind::DECORATION_NODE => self.format_decoration(node),
            _ => self.format_children(node),
        }
    }

    fn format_children(&mut self, node: &CstNode) {
        for child in node.children() {
            self.format_child(child);
        }
    }

    fn format_child(&mut self, child: &CstChild) {
        match child {
            CstChild::Node(node) => self.format_node(node),
            CstChild::Token(token) => self.format_token(token),
        }
    }

    fn format_token(&mut self, token: &CstToken) {
        self.emit_token(token);
    }

    fn emit_token(&mut self, token: &CstToken) {
        // Handle leading trivia
        for trivia in token.leading_trivia() {
            let text = trivia.text(self.source);
            self.emit_text(text);
        }

        // Emit token text
        let text = token.text(self.source);
        self.emit_text(text);

        // Handle trailing trivia
        for trivia in token.trailing_trivia() {
            let text = trivia.text(self.source);
            self.emit_text(text);
        }
    }

    fn emit_text(&mut self, text: &str) {
        self.output.push_str(text);
        if text.contains('\n') {
            self.current_line.clear();
        } else {
            self.current_line.push_str(text);
        }
    }

    fn emit(&mut self, s: &str) {
        self.output.push_str(s);
        if s.contains('\n') {
            self.current_line.clear();
        } else {
            self.current_line.push_str(s);
        }
    }

    // === Node-specific formatters ===

    fn format_tune(&mut self, node: &CstNode) {
        self.format_children(node);
    }

    fn format_header(&mut self, node: &CstNode) {
        self.in_header = true;

        if self.config.normalize_header_order {
            self.format_header_ordered(node);
        } else {
            self.format_children(node);
        }

        self.in_header = false;
    }

    fn format_header_ordered(&mut self, node: &CstNode) {
        // Standard header field order
        let order = ['X', 'T', 'C', 'O', 'A', 'M', 'L', 'Q', 'P', 'Z', 'N', 'G', 'H', 'K'];

        let fields: Vec<_> = node
            .child_nodes()
            .filter(|n| n.kind() == SyntaxKind::HEADER_FIELD)
            .collect();

        // Collect fields by their label
        let mut field_map: std::collections::HashMap<char, Vec<&CstNode>> =
            std::collections::HashMap::new();
        let mut other_fields: Vec<&CstNode> = Vec::new();

        for field in &fields {
            if let Some(label) = self.get_field_label(field) {
                field_map.entry(label).or_default().push(field);
            } else {
                other_fields.push(field);
            }
        }

        // Emit in order
        for label in order {
            if let Some(matching_fields) = field_map.remove(&label) {
                for field in matching_fields {
                    self.format_header_field(field);
                }
            }
        }

        // Emit any remaining fields not in the standard order
        for (_label, remaining) in field_map {
            for field in remaining {
                self.format_header_field(field);
            }
        }

        // Emit other fields (no recognizable label)
        for field in other_fields {
            self.format_header_field(field);
        }
    }

    fn get_field_label(&self, node: &CstNode) -> Option<char> {
        node.find_child_token(SyntaxKind::FIELD_LABEL)
            .map(|t| t.text(self.source).chars().next().unwrap_or('?'))
    }

    fn format_header_field(&mut self, node: &CstNode) {
        // Format header field: Label:Value
        for child in node.children() {
            match child {
                CstChild::Token(token) => {
                    match token.kind() {
                        SyntaxKind::FIELD_LABEL => {
                            // Emit label without leading trivia modifications
                            self.emit_token(token);
                        }
                        SyntaxKind::COLON => {
                            // Emit colon
                            self.emit_token(token);
                        }
                        SyntaxKind::TEXT => {
                            // Emit value, preserving trivia
                            self.emit_token(token);
                        }
                        _ => {
                            self.emit_token(token);
                        }
                    }
                }
                CstChild::Node(n) => self.format_node(n),
            }
        }
    }

    fn format_body(&mut self, node: &CstNode) {
        self.after_bar = false;
        self.after_note = false;

        let children: Vec<_> = node.children().iter().collect();

        for (i, child) in children.iter().enumerate() {
            let prev = if i > 0 { Some(children[i - 1]) } else { None };
            let next = children.get(i + 1).copied();

            self.format_body_element(child, prev, next);
        }
    }

    fn format_body_element(
        &mut self,
        child: &CstChild,
        prev: Option<&CstChild>,
        next: Option<&CstChild>,
    ) {
        match child {
            CstChild::Node(node) => {
                match node.kind() {
                    SyntaxKind::BAR_LINE => {
                        if self.config.space_around_bars {
                            self.maybe_space_before_bar(prev);
                            self.format_bar_line_content(node);
                            self.maybe_space_after_bar(next);
                        } else {
                            // Passthrough: preserve original trivia
                            self.format_node(node);
                        }
                        self.after_bar = true;
                        self.after_note = false;
                    }
                    SyntaxKind::NOTE | SyntaxKind::REST_NODE | SyntaxKind::CHORD => {
                        self.maybe_space_between_notes(prev);
                        self.format_node(node);
                        self.after_note = true;
                        self.after_bar = false;
                    }
                    _ => {
                        self.format_node(node);
                        self.after_bar = false;
                        self.after_note = false;
                    }
                }
            }
            CstChild::Token(token) => {
                self.emit_token(token);
            }
        }
    }

    fn maybe_space_before_bar(&mut self, prev: Option<&CstChild>) {
        if !self.config.space_around_bars {
            return;
        }

        if let Some(prev_child) = prev {
            if self.is_note_like(prev_child) {
                // Check if there's already trailing space
                if !self.output.ends_with(' ') && !self.output.ends_with('\n') {
                    self.emit(" ");
                }
            }
        }
    }

    fn maybe_space_after_bar(&mut self, next: Option<&CstChild>) {
        if !self.config.space_around_bars {
            return;
        }

        if let Some(next_child) = next {
            if self.is_note_like(next_child) {
                // Space will be added when processing the next element
                // We just set a flag
            }
        }
    }

    fn maybe_space_between_notes(&mut self, prev: Option<&CstChild>) {
        if !self.config.normalize_note_spacing {
            return;
        }

        if prev.is_some() {
            // If previous was a bar and we want space around bars
            if self.after_bar && self.config.space_around_bars {
                if !self.output.ends_with(' ') && !self.output.ends_with('\n') {
                    self.emit(" ");
                }
            }
        }
    }

    fn is_note_like(&self, child: &CstChild) -> bool {
        match child {
            CstChild::Node(node) => matches!(
                node.kind(),
                SyntaxKind::NOTE | SyntaxKind::REST_NODE | SyntaxKind::CHORD | SyntaxKind::TUPLET
            ),
            CstChild::Token(_) => false,
        }
    }

    fn format_bar_line_content(&mut self, node: &CstNode) {
        // Just emit the bar line tokens
        for child in node.children() {
            match child {
                CstChild::Token(token) => {
                    // Emit bar token, skip trivia handling for cleaner output
                    self.emit(token.text(self.source));
                }
                CstChild::Node(n) => self.format_node(n),
            }
        }
    }

    fn format_note(&mut self, node: &CstNode) {
        for child in node.children() {
            self.format_child(child);
        }
    }

    fn format_rest(&mut self, node: &CstNode) {
        self.format_children(node);
    }

    fn format_chord(&mut self, node: &CstNode) {
        self.format_children(node);
    }

    fn format_bar_line(&mut self, node: &CstNode) {
        self.format_children(node);
    }

    fn format_tuplet(&mut self, node: &CstNode) {
        self.format_children(node);
    }

    fn format_slur(&mut self, node: &CstNode) {
        self.format_children(node);
    }

    fn format_grace_notes(&mut self, node: &CstNode) {
        self.format_children(node);
    }

    fn format_broken_rhythm(&mut self, node: &CstNode) {
        self.format_children(node);
    }

    fn format_tie(&mut self, node: &CstNode) {
        self.format_children(node);
    }

    fn format_inline_field(&mut self, node: &CstNode) {
        self.format_children(node);
    }

    fn format_duration(&mut self, node: &CstNode) {
        self.format_children(node);
    }

    fn format_accidental(&mut self, node: &CstNode) {
        self.format_children(node);
    }

    fn format_decoration(&mut self, node: &CstNode) {
        self.format_children(node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_preserves_simple_tune() {
        let source = "X:1\nT:Test\nK:C\nCDEF|\n";
        let config = FormatterConfig::passthrough();
        let formatted = format(source, &config);
        // Passthrough should preserve exactly
        assert_eq!(formatted, source);
    }

    #[test]
    fn test_format_ensures_final_newline() {
        let source = "X:1\nK:C\nCDEF|";
        let config = FormatterConfig {
            ensure_final_newline: true,
            ..FormatterConfig::passthrough()
        };
        let formatted = format(source, &config);
        assert!(formatted.ends_with('\n'));
    }

    #[test]
    fn test_format_trims_trailing_whitespace() {
        let source = "X:1  \nK:C  \nCDEF|  \n";
        let config = FormatterConfig {
            trim_trailing_whitespace: true,
            ..FormatterConfig::passthrough()
        };
        let formatted = format(source, &config);
        for line in formatted.lines() {
            assert!(!line.ends_with(' '), "Line has trailing space: {:?}", line);
        }
    }

    #[test]
    fn test_format_default_config() {
        let source = "X:1\nK:C\nCDEF|";
        let config = FormatterConfig::default();
        let formatted = format(source, &config);
        // Should have final newline and no trailing whitespace
        assert!(formatted.ends_with('\n'));
    }

    #[test]
    fn test_format_with_chords() {
        let source = "X:1\nK:C\n[CEG]2[FAC]2|\n";
        let config = FormatterConfig::passthrough();
        let formatted = format(source, &config);
        assert_eq!(formatted, source);
    }

    #[test]
    fn test_format_with_accidentals() {
        let source = "X:1\nK:C\n^C_D=E|\n";
        let config = FormatterConfig::passthrough();
        let formatted = format(source, &config);
        assert_eq!(formatted, source);
    }

    #[test]
    fn test_format_with_tuplets() {
        let source = "X:1\nK:C\n(3CDE|\n";
        let config = FormatterConfig::passthrough();
        let formatted = format(source, &config);
        assert_eq!(formatted, source);
    }

    #[test]
    fn test_format_with_grace_notes() {
        let source = "X:1\nK:C\n{cde}C|\n";
        let config = FormatterConfig::passthrough();
        let formatted = format(source, &config);
        assert_eq!(formatted, source);
    }

    #[test]
    fn test_format_preserves_comments() {
        let source = "X:1 % tune number\nK:C\nCDEF| % bar 1\n";
        let config = FormatterConfig::passthrough();
        let formatted = format(source, &config);
        assert!(formatted.contains("% tune number"));
        assert!(formatted.contains("% bar 1"));
    }
}
