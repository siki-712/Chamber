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
        } else if self.config.remove_empty_header_lines || self.config.normalize_header_spacing {
            // Process header fields individually to handle formatting
            for child in node.children() {
                match child {
                    CstChild::Node(n) if n.kind() == SyntaxKind::HEADER_FIELD => {
                        self.format_header_field(n);
                    }
                    CstChild::Token(t) => {
                        // Skip empty lines (newlines with only whitespace before)
                        if self.config.remove_empty_header_lines {
                            let is_empty_line = t.kind() == SyntaxKind::NEWLINE
                                || (t.kind().is_trivia() && self.is_empty_line_trivia(t));
                            if is_empty_line {
                                continue;
                            }
                        }
                        self.emit_token(t);
                    }
                    CstChild::Node(n) => {
                        self.format_node(n);
                    }
                }
            }
        } else {
            self.format_children(node);
        }

        self.in_header = false;
    }

    fn is_empty_line_trivia(&self, token: &CstToken) -> bool {
        let text = token.text(self.source);
        // Check if this is just whitespace followed by newline
        text.chars().all(|c| c.is_whitespace())
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
        if self.config.normalize_header_spacing {
            self.format_header_field_normalized(node);
        } else {
            // Passthrough: preserve original formatting
            for child in node.children() {
                match child {
                    CstChild::Token(token) => self.emit_token(token),
                    CstChild::Node(n) => self.format_node(n),
                }
            }
        }
    }

    fn format_header_field_normalized(&mut self, node: &CstNode) {
        // Format header field with normalized spacing: "Label:Value"
        // Removes spaces around colon, trims value
        let mut label: Option<&CstToken> = None;
        let mut colon: Option<&CstToken> = None;
        let mut text: Option<&CstToken> = None;

        for child in node.children() {
            if let CstChild::Token(token) = child {
                match token.kind() {
                    SyntaxKind::FIELD_LABEL => label = Some(token),
                    SyntaxKind::COLON => colon = Some(token),
                    SyntaxKind::TEXT => text = Some(token),
                    _ => {}
                }
            }
        }

        // Emit leading trivia from label (preserves comments, skips empty lines)
        if let Some(l) = label {
            let leading = l.leading_trivia();
            let mut i = 0;
            while i < leading.len() {
                let trivia = &leading[i];
                match trivia.kind {
                    SyntaxKind::NEWLINE => {
                        // Check if this is an empty line (newline followed by another newline or just whitespace)
                        if self.config.remove_empty_header_lines {
                            // Skip this newline if it's just creating an empty line
                            // (empty lines in leading trivia means line before this field was blank)
                            i += 1;
                            continue;
                        }
                        self.emit_text(trivia.text(self.source));
                    }
                    SyntaxKind::WHITESPACE => {
                        // Skip whitespace at start of field (indentation will be normalized)
                        // Don't emit whitespace-only leading trivia
                    }
                    SyntaxKind::COMMENT => {
                        // Preserve comments
                        self.emit_text(trivia.text(self.source));
                    }
                    _ => {
                        self.emit_text(trivia.text(self.source));
                    }
                }
                i += 1;
            }
            // Emit label text only (no trailing trivia - spaces before colon)
            self.emit(l.text(self.source));
        }

        // Emit colon (no spaces around it)
        if colon.is_some() {
            self.emit(":");
        }

        // Emit value (trimmed)
        if let Some(t) = text {
            let value = t.text(self.source).trim();
            self.emit(value);

            // Emit trailing trivia (newline, comments after value)
            self.emit_trailing_trivia(t.trailing_trivia());
        } else if let Some(c) = colon {
            // No text, but emit trailing trivia from colon
            self.emit_trailing_trivia(c.trailing_trivia());
        }
    }

    /// Emit trailing trivia, filtering out empty lines if configured.
    fn emit_trailing_trivia(&mut self, trivia: &[chamber_syntax::Trivia]) {
        let mut newline_count = 0;

        for t in trivia {
            match t.kind {
                SyntaxKind::NEWLINE => {
                    newline_count += 1;
                    // If removing empty lines, only emit first newline
                    if self.config.remove_empty_header_lines && self.in_header {
                        if newline_count == 1 {
                            self.emit("\n");
                        }
                        // Skip additional newlines (empty lines)
                    } else {
                        self.emit_text(t.text(self.source));
                    }
                }
                SyntaxKind::WHITESPACE => {
                    // Skip whitespace after newline if we're removing empty lines
                    if self.config.remove_empty_header_lines && self.in_header && newline_count > 0 {
                        continue;
                    }
                    self.emit_text(t.text(self.source));
                }
                _ => {
                    // Comments, etc.
                    self.emit_text(t.text(self.source));
                }
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

    // =========================================================================
    // Basic functionality tests
    // =========================================================================

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

    // =========================================================================
    // Music element preservation tests
    // =========================================================================

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

    // =========================================================================
    // Real-world usage examples
    // =========================================================================

    /// Example: Format a messy tune with default settings
    #[test]
    fn example_cleanup_messy_tune() {
        let messy = "X:1  \nT:My Tune  \nK:C  \nCDEF|G2A2|  \n";

        let formatted = format(messy, &FormatterConfig::default());

        // Trailing whitespace removed, final newline ensured
        // Default config adds spaces around bars
        assert!(formatted.ends_with('\n'));
        assert!(!formatted.contains("  \n")); // No trailing whitespace
        assert!(formatted.contains("CDEF"));
        assert!(formatted.contains("G2A2"));
    }

    /// Example: Preserve exact formatting (for diff-friendly output)
    #[test]
    fn example_preserve_exact_formatting() {
        let source = "X:1\nT:My Tune\nK:C\n C  D  E  F | G2 A2 |\n";

        let formatted = format(source, &FormatterConfig::passthrough());

        // Passthrough mode preserves everything exactly
        assert_eq!(formatted, source);
    }

    /// Example: Minimal cleanup (only trailing whitespace)
    #[test]
    fn example_minimal_cleanup() {
        let source = "X:1  \nK:C\nC D E F|  ";

        let formatted = format(source, &FormatterConfig::minimal());

        // Only trailing whitespace removed, final newline added
        assert!(formatted.ends_with('\n'));
        assert!(!formatted.contains("  \n")); // No lines with trailing spaces
    }

    /// Example: Complex tune with all features
    #[test]
    fn example_complex_tune() {
        let source = r#"X:1
T:The Kesh Jig
M:6/8
L:1/8
K:G
|:GAG GAB|ABA ABd|edd gdd|edB dBA|
GAG GAB|ABA ABd|edd gdB|AGF G3:|
"#;

        // Use passthrough to preserve original spacing
        let config = FormatterConfig::passthrough();
        let formatted = format(source, &config);

        // Should preserve structure exactly
        assert!(formatted.contains("T:The Kesh Jig"));
        assert!(formatted.contains("M:6/8"));
        assert!(formatted.contains("|:GAG GAB|"));
        assert!(formatted.ends_with('\n'));
    }

    /// Example: Tune with decorations
    #[test]
    fn example_decorations() {
        let source = "X:1\nK:C\n!trill!C2 !fermata!D2|\n";

        let formatted = format(source, &FormatterConfig::passthrough());

        assert!(formatted.contains("!trill!C2"));
        assert!(formatted.contains("!fermata!D2"));
    }

    /// Example: Tune with inline fields
    #[test]
    fn example_inline_fields() {
        let source = "X:1\nK:C\nCDEF|[M:3/4]GAB|\n";

        let formatted = format(source, &FormatterConfig::passthrough());

        assert!(formatted.contains("[M:3/4]"));
    }

    /// Example: Multiple voices/parts
    #[test]
    fn example_multiline_tune() {
        let source = r#"X:1
T:Scale Exercise
K:C
CDEF|GABC'|
c'BAG|FEDC|
"#;

        // Use passthrough to preserve original bar spacing
        let formatted = format(source, &FormatterConfig::passthrough());

        assert!(formatted.contains("CDEF|GABC'|"));
        assert!(formatted.contains("c'BAG|FEDC|"));
    }

    /// Example: Repeat bars
    #[test]
    fn example_repeat_bars() {
        let source = "X:1\nK:C\n|:CDEF:|GABC||\n";

        let formatted = format(source, &FormatterConfig::passthrough());

        assert!(formatted.contains("|:CDEF:|"));
        assert!(formatted.contains("||"));
    }

    /// Example: Slurs and ties
    #[test]
    fn example_slurs_and_ties() {
        let source = "X:1\nK:C\n(CDE)F|G-G A2|\n";

        let formatted = format(source, &FormatterConfig::passthrough());

        assert!(formatted.contains("(CDE)"));
        assert!(formatted.contains("G-G"));
    }

    /// Example: Check that formatting is idempotent
    #[test]
    fn example_idempotent_formatting() {
        let source = "X:1  \nT:Test  \nK:C\nCDEF|";

        let config = FormatterConfig::default();
        let first = format(source, &config);
        let second = format(&first, &config);

        // Formatting twice should give the same result
        assert_eq!(first, second);
    }

    // =========================================================================
    // Header ordering tests
    // =========================================================================

    #[test]
    fn test_header_order_normalization() {
        // Fields in wrong order
        let source = "K:C\nT:My Tune\nX:1\n";

        let config = FormatterConfig {
            normalize_header_order: true,
            ..FormatterConfig::passthrough()
        };
        let formatted = format(source, &config);

        // X should come before T, which should come before K
        let x_pos = formatted.find("X:").unwrap();
        let t_pos = formatted.find("T:").unwrap();
        let k_pos = formatted.find("K:").unwrap();

        assert!(x_pos < t_pos, "X: should come before T:");
        assert!(t_pos < k_pos, "T: should come before K:");
    }

    // =========================================================================
    // Header spacing normalization tests
    // =========================================================================

    #[test]
    fn test_header_spacing_normalization() {
        // Spaces around colon
        let source = "X:1\nT : My Tune\nK: C\nCDEF|\n";

        let formatted = format(source, &FormatterConfig::default());

        // Spaces should be removed around colons
        assert!(formatted.contains("T:My Tune"), "T: should have no space around colon");
        assert!(formatted.contains("K:C"), "K: should have no space after colon");
    }

    #[test]
    fn test_remove_empty_header_lines() {
        // Empty line between header fields
        let source = "X:1\nT:Test\n\nM:4/4\nK:C\nCDEF|\n";

        let formatted = format(source, &FormatterConfig::default());

        // Should not have consecutive newlines in header
        let header_end = formatted.find("K:C").unwrap();
        let header = &formatted[..header_end];
        assert!(
            !header.contains("\n\n"),
            "Header should not contain empty lines: {:?}",
            header
        );
    }

    #[test]
    fn test_preserve_header_comments() {
        // Comment in header
        let source = "X:1 % reference number\nT:Test\nK:C\nCDEF|\n";

        let formatted = format(source, &FormatterConfig::default());

        // Comment should be preserved
        assert!(formatted.contains("% reference number"));
    }
}
