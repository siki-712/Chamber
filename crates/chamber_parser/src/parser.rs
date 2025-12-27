use chamber_lexer::{token_text, Lexer, Token, TokenKind};
use chamber_text_size::{TextRange, TextSize};

use crate::ast::*;

/// Parser for ABC notation.
pub struct Parser<'a> {
    source: &'a str,
    tokens: Vec<Token>,
    position: usize,
}

impl<'a> Parser<'a> {
    /// Creates a new parser for the given source text.
    pub fn new(source: &'a str) -> Self {
        let tokens = Lexer::new(source).tokenize();
        Self {
            source,
            tokens,
            position: 0,
        }
    }

    /// Parses the source into a Tune AST.
    pub fn parse(mut self) -> Tune {
        let start = self.current_position();

        let header = self.parse_header();
        let body = self.parse_body();

        let end = self.current_position();

        Tune {
            header,
            body,
            range: TextRange::new(start, end),
        }
    }

    fn parse_header(&mut self) -> Header {
        let start = self.current_position();
        let mut fields = Vec::new();

        while !self.is_at_end() {
            self.skip_trivia();

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
            TokenKind::Note => {
                self.parse_note().map(MusicElement::Note)
            }
            TokenKind::Rest => {
                self.parse_rest().map(MusicElement::Rest)
            }
            TokenKind::Bar | TokenKind::DoubleBar | TokenKind::RepeatStart
            | TokenKind::RepeatEnd | TokenKind::ThinThickBar | TokenKind::ThickThinBar => {
                self.parse_bar_line().map(MusicElement::BarLine)
            }
            TokenKind::LeftBracket => {
                self.parse_chord().map(MusicElement::Chord)
            }
            TokenKind::Tuplet => {
                self.parse_tuplet().map(MusicElement::Tuplet)
            }
            TokenKind::LeftParen => {
                self.parse_slur().map(MusicElement::Slur)
            }
            TokenKind::LeftBrace => {
                self.parse_grace_notes().map(MusicElement::GraceNotes)
            }
            TokenKind::BrokenRhythm => {
                self.parse_broken_rhythm().map(MusicElement::BrokenRhythm)
            }
            TokenKind::Tie => {
                self.parse_tie().map(MusicElement::Tie)
            }
            TokenKind::Newline => {
                self.advance();
                None
            }
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
        let (pitch, base_octave) = Pitch::from_char(note_text.chars().next()?)?;

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

        // Parse duration
        let duration = self.parse_duration();

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
                    denominator = d;
                }
            } else {
                // Just "/" means /2
                denominator = 2;
            }
        }

        if has_duration {
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

        // Consume [
        if !self.check(TokenKind::LeftBracket) {
            return None;
        }
        self.advance();

        // Parse notes until ]
        let mut notes = Vec::new();
        while !self.is_at_end() && !self.check(TokenKind::RightBracket) {
            if let Some(note) = self.parse_note() {
                notes.push(note);
            } else {
                break;
            }
        }

        // Consume ]
        if self.check(TokenKind::RightBracket) {
            self.advance();
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
            if let Some(note) = self.parse_note() {
                notes.push(note);
            } else {
                break;
            }
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

        // Consume (
        if !self.check(TokenKind::LeftParen) {
            return None;
        }
        self.advance();

        // Parse elements until )
        let mut elements = Vec::new();
        while !self.is_at_end() && !self.check(TokenKind::RightParen) {
            if let Some(element) = self.parse_music_element() {
                elements.push(element);
            } else if self.check(TokenKind::Eof) {
                break;
            } else {
                self.advance();
            }
        }

        // Consume )
        if self.check(TokenKind::RightParen) {
            self.advance();
        }

        let end = self.current_position();
        Some(Slur {
            elements,
            range: TextRange::new(start, end),
        })
    }

    fn parse_grace_notes(&mut self) -> Option<GraceNotes> {
        let start = self.current_position();

        // Consume {
        if !self.check(TokenKind::LeftBrace) {
            return None;
        }
        self.advance();

        // Parse notes until }
        let mut notes = Vec::new();
        while !self.is_at_end() && !self.check(TokenKind::RightBrace) {
            if let Some(note) = self.parse_note() {
                notes.push(note);
            } else {
                break;
            }
        }

        // Consume }
        if self.check(TokenKind::RightBrace) {
            self.advance();
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
