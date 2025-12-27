use chamber_diagnostics::{Diagnostic, DiagnosticBag, DiagnosticCode, DiagnosticSink};
use chamber_lexer::{token_text, Lexer, Token, TokenKind};
use chamber_text_size::{TextRange, TextSize};

use crate::ast::*;

/// Result of parsing, containing the AST and any diagnostics.
#[derive(Debug, Clone)]
pub struct ParseResult {
    /// The parsed tune (may be incomplete if errors occurred).
    pub tune: Tune,
    /// Diagnostics collected during parsing.
    pub diagnostics: Vec<Diagnostic>,
}

impl ParseResult {
    /// Returns true if there are any errors.
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.is_error())
    }

    /// Returns true if there are any warnings.
    pub fn has_warnings(&self) -> bool {
        self.diagnostics.iter().any(|d| d.is_warning())
    }
}

/// Parser for ABC notation.
pub struct Parser<'a, S: DiagnosticSink = DiagnosticBag> {
    source: &'a str,
    tokens: Vec<Token>,
    position: usize,
    diagnostics: Option<S>,
}

impl<'a> Parser<'a, DiagnosticBag> {
    /// Creates a new parser for the given source text (no diagnostics).
    pub fn new(source: &'a str) -> Self {
        let tokens = Lexer::new(source).tokenize();
        Self {
            source,
            tokens,
            position: 0,
            diagnostics: None,
        }
    }
}

impl<'a, S: DiagnosticSink> Parser<'a, S> {
    /// Creates a new parser with a diagnostic sink.
    pub fn with_diagnostics(source: &'a str, diagnostics: S) -> Self {
        let tokens = Lexer::new(source).tokenize();
        Self {
            source,
            tokens,
            position: 0,
            diagnostics: Some(diagnostics),
        }
    }

    /// Reports a diagnostic if a sink is available.
    fn report(&mut self, diagnostic: Diagnostic) {
        if let Some(ref mut sink) = self.diagnostics {
            sink.report(diagnostic);
        }
    }

    /// Parses the source into a Tune AST.
    pub fn parse(&mut self) -> Tune {
        let start = self.current_position();

        // Handle any initial error tokens
        self.handle_error_tokens();

        let header = self.parse_header();
        self.validate_header(&header);
        let body = self.parse_body();

        let end = self.current_position();

        // S001: EmptyTune - warn if tune has no content
        if header.fields.is_empty() && body.elements.is_empty() {
            self.report(Diagnostic::warning(
                DiagnosticCode::EmptyTune,
                TextRange::new(start, end),
                "empty tune (no header or body content)",
            ));
        }

        Tune {
            header,
            body,
            range: TextRange::new(start, end),
        }
    }

    /// Validates the header and reports any diagnostics.
    fn validate_header(&mut self, header: &Header) {
        let fields = &header.fields;

        // Check for required fields
        let x_fields: Vec<_> = fields
            .iter()
            .filter(|f| f.kind == HeaderFieldKind::ReferenceNumber)
            .collect();
        let k_field = fields.iter().find(|f| f.kind == HeaderFieldKind::Key);
        let t_field = fields.iter().find(|f| f.kind == HeaderFieldKind::Title);

        // H001: Missing X:
        if x_fields.is_empty() {
            self.report(Diagnostic::error(
                DiagnosticCode::MissingReferenceNumber,
                header.range,
                "missing reference number field (X:)",
            ));
        }

        // H003: Duplicate X:
        if x_fields.len() > 1 {
            for dup in &x_fields[1..] {
                self.report(
                    Diagnostic::error(
                        DiagnosticCode::DuplicateReferenceNumber,
                        dup.range,
                        "duplicate reference number field",
                    )
                    .with_label(x_fields[0].range, "first X: defined here"),
                );
            }
        }

        // H002: Missing K:
        if k_field.is_none() {
            self.report(Diagnostic::error(
                DiagnosticCode::MissingKeyField,
                header.range,
                "missing key field (K:)",
            ));
        }

        // H009: Missing T: (warning)
        if t_field.is_none() {
            self.report(Diagnostic::warning(
                DiagnosticCode::MissingTitle,
                header.range,
                "missing title field (T:)",
            ));
        }

        // Validate field values
        for field in fields {
            match field.kind {
                HeaderFieldKind::ReferenceNumber => {
                    self.validate_reference_number(field);
                }
                HeaderFieldKind::Title => {
                    if field.value.trim().is_empty() {
                        // H010: Empty T:
                        self.report(Diagnostic::warning(
                            DiagnosticCode::EmptyTitle,
                            field.range,
                            "empty title field",
                        ));
                    }
                }
                HeaderFieldKind::Meter => {
                    self.validate_meter(field);
                }
                HeaderFieldKind::Tempo => {
                    self.validate_tempo(field);
                }
                HeaderFieldKind::UnitNoteLength => {
                    self.validate_unit_note_length(field);
                }
                HeaderFieldKind::Key => {
                    self.validate_key(field);
                }
                _ => {}
            }
        }

        // H004: Check field order (X: should be first)
        if let Some(first) = fields.first() {
            if first.kind != HeaderFieldKind::ReferenceNumber {
                self.report(Diagnostic::warning(
                    DiagnosticCode::InvalidFieldOrder,
                    first.range,
                    "X: (reference number) should be the first field in the header",
                ));
            }
        }
    }

    /// H011, H012: Validate reference number field
    fn validate_reference_number(&mut self, field: &HeaderField) {
        let value = field.value.trim();
        if value.is_empty() {
            self.report(Diagnostic::error(
                DiagnosticCode::EmptyReferenceNumber,
                field.range,
                "empty reference number field",
            ));
        } else if value.parse::<u32>().is_err() {
            self.report(Diagnostic::error(
                DiagnosticCode::InvalidReferenceNumber,
                field.range,
                format!(
                    "invalid reference number '{}' (must be a positive integer)",
                    value
                ),
            ));
        }
    }

    /// H005: Validate meter field (M:)
    /// Valid formats: 4/4, 3/4, 6/8, C, C|, none
    fn validate_meter(&mut self, field: &HeaderField) {
        let value = field.value.trim();
        if value.is_empty() {
            return; // Empty is allowed (uses default)
        }

        // Special values
        if matches!(value, "C" | "C|" | "none") {
            return;
        }

        // Fraction format: num/denom
        if let Some((num, denom)) = value.split_once('/') {
            let num_ok = num.trim().parse::<u32>().is_ok();
            let denom_ok = denom.trim().parse::<u32>().map(|d| d > 0).unwrap_or(false);

            if !num_ok || !denom_ok {
                self.report(Diagnostic::error(
                    DiagnosticCode::InvalidMeterValue,
                    field.range,
                    format!("invalid meter value '{}' (expected format: 4/4, 3/4, C, C|)", value),
                ));
            }
        } else {
            self.report(Diagnostic::error(
                DiagnosticCode::InvalidMeterValue,
                field.range,
                format!("invalid meter value '{}' (expected format: 4/4, 3/4, C, C|)", value),
            ));
        }
    }

    /// H006: Validate tempo field (Q:)
    /// Valid formats: 120, 1/4=120, "Allegro" 1/4=120, 3/8=120
    fn validate_tempo(&mut self, field: &HeaderField) {
        let value = field.value.trim();
        if value.is_empty() {
            return; // Empty is allowed
        }

        // Remove quoted tempo name if present
        let value = if value.starts_with('"') {
            if let Some(end_quote) = value[1..].find('"') {
                value[end_quote + 2..].trim()
            } else {
                value // malformed, let it fail below
            }
        } else {
            value
        };

        // If empty after removing name, it's just a tempo name (valid)
        if value.is_empty() {
            return;
        }

        // Check for note=bpm format (1/4=120) or just bpm (120)
        if let Some((note_part, bpm_part)) = value.split_once('=') {
            // Validate note length
            let note_part = note_part.trim();
            if let Some((num, denom)) = note_part.split_once('/') {
                let num_ok = num.trim().parse::<u32>().is_ok();
                let denom_ok = denom.trim().parse::<u32>().map(|d| d > 0).unwrap_or(false);
                if !num_ok || !denom_ok {
                    self.report(Diagnostic::error(
                        DiagnosticCode::InvalidTempo,
                        field.range,
                        format!("invalid tempo note length '{}'", note_part),
                    ));
                    return;
                }
            } else if note_part.parse::<u32>().is_err() {
                self.report(Diagnostic::error(
                    DiagnosticCode::InvalidTempo,
                    field.range,
                    format!("invalid tempo note length '{}'", note_part),
                ));
                return;
            }

            // Validate BPM
            let bpm_part = bpm_part.trim();
            if bpm_part.parse::<u32>().is_err() {
                self.report(Diagnostic::error(
                    DiagnosticCode::InvalidTempo,
                    field.range,
                    format!("invalid tempo BPM '{}'", bpm_part),
                ));
            }
        } else {
            // Just BPM
            if value.parse::<u32>().is_err() {
                self.report(Diagnostic::error(
                    DiagnosticCode::InvalidTempo,
                    field.range,
                    format!("invalid tempo value '{}' (expected format: 120 or 1/4=120)", value),
                ));
            }
        }
    }

    /// H007: Validate unit note length field (L:)
    /// Valid formats: 1/4, 1/8, 1/16
    fn validate_unit_note_length(&mut self, field: &HeaderField) {
        let value = field.value.trim();
        if value.is_empty() {
            return; // Empty uses default
        }

        if let Some((num, denom)) = value.split_once('/') {
            let num_ok = num.trim().parse::<u32>().is_ok();
            let denom_ok = denom.trim().parse::<u32>().map(|d| d > 0).unwrap_or(false);

            if !num_ok || !denom_ok {
                self.report(Diagnostic::error(
                    DiagnosticCode::InvalidUnitNoteLength,
                    field.range,
                    format!("invalid unit note length '{}' (expected format: 1/4, 1/8)", value),
                ));
            }
        } else {
            self.report(Diagnostic::error(
                DiagnosticCode::InvalidUnitNoteLength,
                field.range,
                format!("invalid unit note length '{}' (expected format: 1/4, 1/8)", value),
            ));
        }
    }

    /// H008: Validate key field (K:)
    /// Valid formats: C, G, Am, Dmix, HP, Hp, none, etc.
    fn validate_key(&mut self, field: &HeaderField) {
        let value = field.value.trim();
        if value.is_empty() {
            self.report(Diagnostic::error(
                DiagnosticCode::InvalidKeySignature,
                field.range,
                "empty key field (K:)",
            ));
            return;
        }

        // Special values
        if matches!(value, "none" | "HP" | "Hp") {
            return;
        }

        let mut chars = value.chars().peekable();

        // First char must be A-G
        let tonic = match chars.next() {
            Some(c) if c.is_ascii_uppercase() && ('A'..='G').contains(&c) => c,
            _ => {
                self.report(Diagnostic::error(
                    DiagnosticCode::InvalidKeySignature,
                    field.range,
                    format!(
                        "invalid key '{}' (must start with A-G)",
                        value
                    ),
                ));
                return;
            }
        };

        // Check for accidental (# or b)
        if let Some(&c) = chars.peek() {
            if c == '#' || c == 'b' {
                chars.next();
            }
        }

        // Rest is mode (optional): m, min, maj, mix, dor, phr, lyd, loc, exp
        let mode: String = chars.collect();
        let mode = mode.to_lowercase();

        if !mode.is_empty() {
            let valid_modes = [
                "m", "min", "minor",
                "maj", "major",
                "mix", "mixolydian",
                "dor", "dorian",
                "phr", "phrygian",
                "lyd", "lydian",
                "loc", "locrian",
                "exp", // explicit (no accidentals)
            ];

            // Check if mode starts with any valid mode
            let mode_valid = valid_modes.iter().any(|m| mode.starts_with(m));

            if !mode_valid && !mode.chars().all(|c| c.is_whitespace()) {
                self.report(Diagnostic::error(
                    DiagnosticCode::InvalidKeySignature,
                    field.range,
                    format!(
                        "invalid key mode '{}' for key {}",
                        mode, tonic
                    ),
                ));
            }
        }
    }

    /// Handles any error tokens at the current position.
    fn handle_error_tokens(&mut self) {
        while self.check(TokenKind::Error) {
            let token = self.advance().unwrap();
            let text = self.token_text(&token);
            self.report(Diagnostic::error(
                DiagnosticCode::UnexpectedCharacter,
                token.range,
                format!("unexpected character '{}'", text.chars().next().unwrap_or('?')),
            ));
        }
    }

    fn parse_header(&mut self) -> Header {
        let start = self.current_position();
        let mut fields = Vec::new();

        while !self.is_at_end() {
            self.skip_trivia();
            self.handle_error_tokens();

            // Check if we're still in header (field label followed by colon)
            if self.check(TokenKind::FieldLabel) {
                if let Some(field) = self.parse_header_field() {
                    let is_key = field.kind == HeaderFieldKind::Key;
                    fields.push(field);

                    // K: field marks end of header
                    if is_key {
                        self.skip_newlines();
                        break;
                    }
                }
            } else {
                break;
            }
        }

        let end = self.current_position();
        Header {
            fields,
            range: TextRange::new(start, end),
        }
    }

    fn parse_header_field(&mut self) -> Option<HeaderField> {
        let start = self.current_position();

        // Expect field label
        let label_token = self.advance()?;
        if label_token.kind != TokenKind::FieldLabel {
            return None;
        }
        let label_text = self.token_text(&label_token);
        let kind = HeaderFieldKind::from_char(label_text.chars().next()?);

        // Expect colon
        self.skip_trivia();
        if !self.check(TokenKind::Colon) {
            return None;
        }
        self.advance();

        // Collect value until newline
        self.skip_whitespace_only();
        let value = self.collect_until_newline();

        // Skip the newline
        self.skip_newlines();

        let end = self.current_position();
        Some(HeaderField {
            kind,
            value,
            range: TextRange::new(start, end),
        })
    }

    fn parse_body(&mut self) -> Body {
        let start = self.current_position();
        let mut elements = Vec::new();

        while !self.is_at_end() {
            self.skip_trivia();
            self.handle_error_tokens();

            // S002: UnexpectedToken - field label in body (should be in header)
            if self.check(TokenKind::FieldLabel) {
                let token = self.advance().unwrap();
                let label = self.token_text(&token);
                self.report(Diagnostic::warning(
                    DiagnosticCode::UnexpectedToken,
                    token.range,
                    format!(
                        "field '{}:' found in music body (should be in header before K:)",
                        label
                    ),
                ));
                // Skip the rest of this field line
                self.skip_trivia();
                if self.check(TokenKind::Colon) {
                    self.advance();
                }
                self.collect_until_newline();
                continue;
            }

            // Handle unexpected closing brackets
            if self.check(TokenKind::RightBracket) {
                let token = self.advance().unwrap();
                self.report(Diagnostic::error(
                    DiagnosticCode::UnexpectedClosingBracket,
                    token.range,
                    "unexpected ']' without matching '['",
                ));
                continue;
            }

            if self.check(TokenKind::RightParen) {
                let token = self.advance().unwrap();
                self.report(Diagnostic::error(
                    DiagnosticCode::UnexpectedClosingParen,
                    token.range,
                    "unexpected ')' without matching '('",
                ));
                continue;
            }

            if self.check(TokenKind::RightBrace) {
                let token = self.advance().unwrap();
                self.report(Diagnostic::error(
                    DiagnosticCode::UnexpectedClosingBrace,
                    token.range,
                    "unexpected '}' without matching '{'",
                ));
                continue;
            }

            // Skip newlines
            if self.check(TokenKind::Newline) {
                self.advance();
                continue;
            }

            if let Some(element) = self.parse_music_element() {
                elements.push(element);
            } else {
                // Skip unknown tokens
                if !self.is_at_end() && !self.check(TokenKind::Eof) {
                    self.advance();
                }
            }
        }

        let end = self.current_position();
        Body {
            elements,
            range: TextRange::new(start, end),
        }
    }

    fn parse_music_element(&mut self) -> Option<MusicElement> {
        self.skip_trivia();

        let token = self.peek()?;

        match token.kind {
            TokenKind::Sharp | TokenKind::Flat | TokenKind::Natural => {
                self.parse_note().map(MusicElement::Note)
            }
            TokenKind::Note => self.parse_note().map(MusicElement::Note),
            TokenKind::Rest => self.parse_rest().map(MusicElement::Rest),
            TokenKind::Bar
            | TokenKind::DoubleBar
            | TokenKind::RepeatStart
            | TokenKind::RepeatEnd
            | TokenKind::ThinThickBar
            | TokenKind::ThickThinBar => self.parse_bar_line().map(MusicElement::BarLine),
            TokenKind::LeftBracket => {
                if self.is_inline_field() {
                    self.parse_inline_field().map(MusicElement::InlineField)
                } else {
                    self.parse_chord().map(MusicElement::Chord)
                }
            }
            TokenKind::Tuplet => self.parse_tuplet().map(MusicElement::Tuplet),
            TokenKind::LeftParen => self.parse_slur().map(MusicElement::Slur),
            TokenKind::LeftBrace => self.parse_grace_notes().map(MusicElement::GraceNotes),
            TokenKind::BrokenRhythm => self.parse_broken_rhythm().map(MusicElement::BrokenRhythm),
            TokenKind::Tie => self.parse_tie().map(MusicElement::Tie),
            _ => None,
        }
    }

    fn parse_note(&mut self) -> Option<Note> {
        let start = self.current_position();

        // Parse optional accidental
        let accidental = self.parse_accidental();

        // Parse pitch
        let token = self.advance()?;
        if token.kind != TokenKind::Note {
            return None;
        }

        let note_text = self.token_text(&token);
        let note_char = note_text.chars().next()?;
        let (pitch, base_octave) = match Pitch::from_char(note_char) {
            Some(result) => result,
            None => {
                // M007: InvalidNoteName
                self.report(Diagnostic::error(
                    DiagnosticCode::InvalidNoteName,
                    token.range,
                    format!("invalid note name '{}'", note_char),
                ));
                return None;
            }
        };

        // Parse octave modifiers
        let mut octave = base_octave;
        while let Some(token) = self.peek() {
            match token.kind {
                TokenKind::OctaveUp => {
                    octave += 1;
                    self.advance();
                }
                TokenKind::OctaveDown => {
                    octave -= 1;
                    self.advance();
                }
                _ => break,
            }
        }

        // W001: UnusualOctave (very high or very low)
        if octave > 3 || octave < -2 {
            self.report(Diagnostic::warning(
                DiagnosticCode::UnusualOctave,
                TextRange::new(start, self.current_position()),
                format!(
                    "unusual octave {} (notes this {} are rare)",
                    octave,
                    if octave > 3 { "high" } else { "low" }
                ),
            ));
        }

        // Parse duration
        let duration = self.parse_duration_with_validation(start);

        let end = self.current_position();
        Some(Note {
            pitch,
            octave,
            accidental,
            duration,
            range: TextRange::new(start, end),
        })
    }

    fn parse_accidental(&mut self) -> Option<Accidental> {
        let token = self.peek()?;

        match token.kind {
            TokenKind::Sharp => {
                self.advance();
                // Check for double sharp
                if self.check(TokenKind::Sharp) {
                    self.advance();
                    Some(Accidental::DoubleSharp)
                } else {
                    Some(Accidental::Sharp)
                }
            }
            TokenKind::Flat => {
                self.advance();
                // Check for double flat
                if self.check(TokenKind::Flat) {
                    self.advance();
                    Some(Accidental::DoubleFlat)
                } else {
                    Some(Accidental::Flat)
                }
            }
            TokenKind::Natural => {
                self.advance();
                Some(Accidental::Natural)
            }
            _ => None,
        }
    }

    fn parse_duration(&mut self) -> Option<Duration> {
        self.parse_duration_internal(None)
    }

    fn parse_duration_with_validation(&mut self, note_start: TextSize) -> Option<Duration> {
        self.parse_duration_internal(Some(note_start))
    }

    fn parse_duration_internal(&mut self, note_start: Option<TextSize>) -> Option<Duration> {
        let dur_start = self.current_position();
        let mut numerator = 1u32;
        let mut denominator = 1u32;
        let mut has_duration = false;

        // Parse numerator (note length number)
        if self.check(TokenKind::NoteLength) {
            let token = self.advance()?;
            let text = self.token_text(&token);
            if let Ok(n) = text.parse::<u32>() {
                numerator = n;
                has_duration = true;
            }
        }

        // Parse slash and denominator
        if self.check(TokenKind::Slash) {
            self.advance();
            has_duration = true;

            if self.check(TokenKind::NoteLength) {
                let token = self.advance()?;
                let text = self.token_text(&token);
                if let Ok(d) = text.parse::<u32>() {
                    if d == 0 {
                        // M009: InvalidDuration - zero denominator
                        self.report(Diagnostic::error(
                            DiagnosticCode::InvalidDuration,
                            TextRange::new(dur_start, self.current_position()),
                            "invalid duration: denominator cannot be zero",
                        ));
                        denominator = 1; // Use 1 as fallback
                    } else {
                        denominator = d;
                    }
                }
            } else {
                // Just "/" means /2
                denominator = 2;
            }
        }

        if has_duration {
            // W002: SuspiciousDuration - very large duration
            let effective_duration = numerator as f64 / denominator as f64;
            if effective_duration >= 16.0 {
                let range = if let Some(start) = note_start {
                    TextRange::new(start, self.current_position())
                } else {
                    TextRange::new(dur_start, self.current_position())
                };
                self.report(Diagnostic::warning(
                    DiagnosticCode::SuspiciousDuration,
                    range,
                    format!(
                        "suspicious duration {}/{} (very long note)",
                        numerator, denominator
                    ),
                ));
            }

            Some(Duration::new(numerator, denominator))
        } else {
            None
        }
    }

    fn parse_rest(&mut self) -> Option<Rest> {
        let start = self.current_position();

        let token = self.advance()?;
        if token.kind != TokenKind::Rest {
            return None;
        }

        let rest_text = self.token_text(&token);
        let multi_measure = rest_text == "Z";

        let duration = self.parse_duration();

        let end = self.current_position();
        Some(Rest {
            multi_measure,
            duration,
            range: TextRange::new(start, end),
        })
    }

    fn parse_bar_line(&mut self) -> Option<BarLine> {
        let token = self.advance()?;

        let kind = match token.kind {
            TokenKind::Bar => BarLineKind::Single,
            TokenKind::DoubleBar => BarLineKind::Double,
            TokenKind::RepeatStart => BarLineKind::RepeatStart,
            TokenKind::RepeatEnd => BarLineKind::RepeatEnd,
            TokenKind::ThinThickBar => BarLineKind::ThinThick,
            TokenKind::ThickThinBar => BarLineKind::ThickThin,
            _ => return None,
        };

        Some(BarLine {
            kind,
            range: token.range,
        })
    }

    fn parse_chord(&mut self) -> Option<Chord> {
        let start = self.current_position();
        let open_bracket_range = self.peek()?.range;

        // Consume [
        if !self.check(TokenKind::LeftBracket) {
            return None;
        }
        self.advance();

        // Parse notes until ]
        let mut notes = Vec::new();
        while !self.is_at_end() && !self.check(TokenKind::RightBracket) {
            // Handle error tokens inside chord
            if self.check(TokenKind::Error) {
                self.handle_error_tokens();
                continue;
            }

            // Check recovery point BEFORE attempting to parse (parse_note advances)
            if self.is_recovery_point() {
                break;
            }

            // Opening brackets of other structures end this chord
            if self.check(TokenKind::LeftParen) || self.check(TokenKind::LeftBrace) {
                break;
            }

            if let Some(note) = self.parse_note() {
                notes.push(note);
            } else {
                self.advance();
            }
        }

        // Consume ] or report error
        let closed = if self.check(TokenKind::RightBracket) {
            self.advance();
            true
        } else {
            let end = self.current_position();
            self.report(
                Diagnostic::error(
                    DiagnosticCode::UnclosedChord,
                    TextRange::new(start, end),
                    "unclosed chord, missing ']'",
                )
                .with_label(open_bracket_range, "opening '[' here"),
            );
            false
        };

        // Warn about empty chord
        if notes.is_empty() && closed {
            self.report(Diagnostic::warning(
                DiagnosticCode::EmptyChord,
                TextRange::new(start, self.current_position()),
                "empty chord",
            ));
        }

        // Parse chord duration
        let duration = self.parse_duration();

        let end = self.current_position();
        Some(Chord {
            notes,
            duration,
            range: TextRange::new(start, end),
        })
    }

    /// Checks if the current position is an inline field [X:...].
    fn is_inline_field(&self) -> bool {
        // Look ahead: [ followed by FieldLabel
        let mut i = self.position;

        // Skip [
        if self.tokens.get(i).map(|t| t.kind) != Some(TokenKind::LeftBracket) {
            return false;
        }
        i += 1;

        // Skip whitespace
        while self.tokens.get(i).map(|t| t.kind) == Some(TokenKind::Whitespace) {
            i += 1;
        }

        // Check for FieldLabel
        self.tokens.get(i).map(|t| t.kind) == Some(TokenKind::FieldLabel)
    }

    fn parse_inline_field(&mut self) -> Option<InlineField> {
        let start = self.current_position();
        let open_bracket_range = self.peek()?.range;

        // Consume [
        if !self.check(TokenKind::LeftBracket) {
            return None;
        }
        self.advance();

        // Skip whitespace
        self.skip_whitespace_only();

        // Parse field label
        let label_token = self.advance()?;
        if label_token.kind != TokenKind::FieldLabel {
            return None;
        }
        let label_text = self.token_text(&label_token);
        let label = label_text.chars().next()?;

        // Skip whitespace
        self.skip_whitespace_only();

        // Expect colon
        if !self.check(TokenKind::Colon) {
            return None;
        }
        self.advance();

        // Collect value until ]
        let value_start = self.current_position();
        while !self.is_at_end() && !self.check(TokenKind::RightBracket) {
            // Stop at recovery points
            if self.is_recovery_point() {
                break;
            }
            self.advance();
        }
        let value_end = self.current_position();
        let value = self.source[value_start.raw() as usize..value_end.raw() as usize].to_string();

        // Consume ] or report error
        if self.check(TokenKind::RightBracket) {
            self.advance();
        } else {
            self.report(
                Diagnostic::error(
                    DiagnosticCode::UnclosedInlineField,
                    TextRange::new(start, self.current_position()),
                    "unclosed inline field, missing ']'",
                )
                .with_label(open_bracket_range, "opening '[' here"),
            );
        }

        let end = self.current_position();
        Some(InlineField {
            label,
            value: value.trim().to_string(),
            range: TextRange::new(start, end),
        })
    }

    fn parse_tuplet(&mut self) -> Option<Tuplet> {
        let start = self.current_position();

        // Consume tuplet marker (e.g., (3)
        let token = self.advance()?;
        if token.kind != TokenKind::Tuplet {
            return None;
        }

        let text = self.token_text(&token);
        // Extract the ratio from (3 -> 3
        let ratio: u32 = text[1..].parse().unwrap_or(3);

        // Parse notes in the tuplet
        let mut notes = Vec::new();
        for _ in 0..ratio {
            self.skip_trivia();
            self.handle_error_tokens();
            if let Some(note) = self.parse_note() {
                notes.push(note);
            } else {
                break;
            }
        }

        // Check if we got enough notes
        if (notes.len() as u32) < ratio && !notes.is_empty() {
            let end = self.current_position();
            self.report(Diagnostic::warning(
                DiagnosticCode::TupletNoteMismatch,
                TextRange::new(start, end),
                format!(
                    "tuplet expects {} notes but found {}",
                    ratio,
                    notes.len()
                ),
            ));
        }

        if notes.is_empty() {
            self.report(Diagnostic::warning(
                DiagnosticCode::EmptyTuplet,
                TextRange::new(start, self.current_position()),
                "empty tuplet",
            ));
        }

        let end = self.current_position();
        Some(Tuplet {
            ratio,
            notes,
            range: TextRange::new(start, end),
        })
    }

    fn parse_slur(&mut self) -> Option<Slur> {
        let start = self.current_position();
        let open_paren_range = self.peek()?.range;

        // Consume (
        if !self.check(TokenKind::LeftParen) {
            return None;
        }
        self.advance();

        // Parse elements until )
        let mut elements = Vec::new();
        while !self.is_at_end() && !self.check(TokenKind::RightParen) {
            self.handle_error_tokens();

            // Check recovery point BEFORE attempting to parse
            if self.is_recovery_point() {
                break;
            }

            if let Some(element) = self.parse_music_element() {
                elements.push(element);
            } else {
                // Check for unexpected closing brackets before skipping
                if self.check(TokenKind::RightBracket) {
                    let token = self.advance().unwrap();
                    self.report(Diagnostic::error(
                        DiagnosticCode::UnexpectedClosingBracket,
                        token.range,
                        "unexpected ']' without matching '['",
                    ));
                } else if self.check(TokenKind::RightBrace) {
                    let token = self.advance().unwrap();
                    self.report(Diagnostic::error(
                        DiagnosticCode::UnexpectedClosingBrace,
                        token.range,
                        "unexpected '}' without matching '{'",
                    ));
                } else {
                    self.advance();
                }
            }
        }

        // Consume ) or report error
        if self.check(TokenKind::RightParen) {
            self.advance();
        } else {
            let end = self.current_position();
            self.report(
                Diagnostic::error(
                    DiagnosticCode::UnclosedSlur,
                    TextRange::new(start, end),
                    "unclosed slur, missing ')'",
                )
                .with_label(open_paren_range, "opening '(' here"),
            );
        }

        let end = self.current_position();
        Some(Slur {
            elements,
            range: TextRange::new(start, end),
        })
    }

    fn parse_grace_notes(&mut self) -> Option<GraceNotes> {
        let start = self.current_position();
        let open_brace_range = self.peek()?.range;

        // Consume {
        if !self.check(TokenKind::LeftBrace) {
            return None;
        }
        self.advance();

        // Parse notes until }
        let mut notes = Vec::new();
        while !self.is_at_end() && !self.check(TokenKind::RightBrace) {
            self.handle_error_tokens();

            // Check recovery point BEFORE attempting to parse (parse_note advances)
            if self.is_recovery_point() {
                break;
            }

            // Opening brackets of other structures end this grace notes
            if self.check(TokenKind::LeftBracket)
                || self.check(TokenKind::LeftParen)
                || self.check(TokenKind::LeftBrace)
            {
                break;
            }

            if let Some(note) = self.parse_note() {
                notes.push(note);
            } else {
                self.advance();
            }
        }

        // Consume } or report error
        if self.check(TokenKind::RightBrace) {
            self.advance();
        } else {
            let end = self.current_position();
            self.report(
                Diagnostic::error(
                    DiagnosticCode::UnclosedGraceNotes,
                    TextRange::new(start, end),
                    "unclosed grace notes, missing '}'",
                )
                .with_label(open_brace_range, "opening '{' here"),
            );
        }

        let end = self.current_position();
        Some(GraceNotes {
            notes,
            range: TextRange::new(start, end),
        })
    }

    fn parse_broken_rhythm(&mut self) -> Option<BrokenRhythm> {
        let token = self.advance()?;
        if token.kind != TokenKind::BrokenRhythm {
            return None;
        }

        let text = self.token_text(&token);
        let dotted_first = text.starts_with('>');
        let count = text.len() as u32;

        Some(BrokenRhythm {
            dotted_first,
            count,
            range: token.range,
        })
    }

    fn parse_tie(&mut self) -> Option<Tie> {
        let token = self.advance()?;
        if token.kind != TokenKind::Tie {
            return None;
        }

        Some(Tie { range: token.range })
    }

    // Helper methods

    /// Checks if we're at a recovery point (bar line or newline).
    fn is_recovery_point(&self) -> bool {
        matches!(
            self.peek().map(|t| t.kind),
            Some(
                TokenKind::Bar
                    | TokenKind::DoubleBar
                    | TokenKind::RepeatStart
                    | TokenKind::RepeatEnd
                    | TokenKind::Newline
                    | TokenKind::Eof
            )
        )
    }

    fn current_position(&self) -> TextSize {
        self.tokens
            .get(self.position)
            .map(|t| t.range.start())
            .unwrap_or_else(|| {
                self.tokens
                    .last()
                    .map(|t| t.range.end())
                    .unwrap_or(TextSize::new(0))
            })
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.tokens.len() || self.check(TokenKind::Eof)
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    fn advance(&mut self) -> Option<Token> {
        if self.position < self.tokens.len() {
            let token = self.tokens[self.position].clone();
            self.position += 1;
            Some(token)
        } else {
            None
        }
    }

    fn check(&self, kind: TokenKind) -> bool {
        self.peek().map(|t| t.kind == kind).unwrap_or(false)
    }

    fn token_text(&self, token: &Token) -> &str {
        token_text(self.source, token)
    }

    fn skip_trivia(&mut self) {
        while let Some(token) = self.peek() {
            if token.kind.is_trivia() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_whitespace_only(&mut self) {
        while self.check(TokenKind::Whitespace) {
            self.advance();
        }
    }

    fn skip_newlines(&mut self) {
        while self.check(TokenKind::Newline) {
            self.advance();
        }
    }

    fn collect_until_newline(&mut self) -> String {
        let mut value = String::new();
        while let Some(token) = self.peek() {
            match token.kind {
                TokenKind::Newline | TokenKind::Eof => break,
                TokenKind::Comment => break,
                _ => {
                    value.push_str(self.token_text(token));
                    self.advance();
                }
            }
        }
        value.trim().to_string()
    }
}

/// Parses ABC notation source into a Tune AST.
pub fn parse(source: &str) -> Tune {
    Parser::new(source).parse()
}

impl Parser<'_, DiagnosticBag> {
    /// Consumes the parser and returns the diagnostics.
    pub fn into_diagnostics(self) -> Vec<Diagnostic> {
        self.diagnostics
            .map(|bag| bag.into_diagnostics())
            .unwrap_or_default()
    }
}

/// Parses ABC notation source with diagnostics.
pub fn parse_with_diagnostics(source: &str) -> ParseResult {
    let bag = DiagnosticBag::new();
    let mut parser = Parser::with_diagnostics(source, bag);
    let tune = parser.parse();
    let diagnostics = parser.into_diagnostics();

    ParseResult { tune, diagnostics }
}
