use chamber_text_size::{TextRange, TextSize};

use crate::{Token, TokenKind};

/// A lexer for ABC notation.
pub struct Lexer<'a> {
    source: &'a str,
    position: usize,
    /// Whether we're currently in a header context (field value parsing)
    in_header: bool,
}

impl<'a> Lexer<'a> {
    /// Creates a new lexer for the given source text.
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            position: 0,
            in_header: false,
        }
    }

    /// Tokenizes the entire source and returns all tokens.
    pub fn tokenize(mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token();
            let is_eof = token.kind == TokenKind::Eof;
            tokens.push(token);
            if is_eof {
                break;
            }
        }
        tokens
    }

    /// Returns the next token.
    pub fn next_token(&mut self) -> Token {
        if self.is_at_end() {
            return self.make_token(TokenKind::Eof, 0);
        }

        let start = self.position;
        let c = self.advance();

        let kind = match c {
            // Whitespace (not newline)
            ' ' | '\t' => self.whitespace(),

            // Newline
            '\n' => {
                self.in_header = false;
                TokenKind::Newline
            }
            '\r' => {
                if self.peek() == Some('\n') {
                    self.advance();
                }
                self.in_header = false;
                TokenKind::Newline
            }

            // Comment
            '%' => self.comment(),

            // Line continuation
            '\\' => TokenKind::LineContinuation,

            // Colon - context switch to header
            ':' => {
                // Check for repeat markers like :| or : |
                if self.has_bar_ahead() {
                    // Skip whitespace before bar
                    while self.peek().map(|c| c == ' ' || c == '\t').unwrap_or(false) {
                        self.advance();
                    }
                    // Consume the |
                    self.advance();
                    TokenKind::RepeatEnd
                } else {
                    self.in_header = true;
                    TokenKind::Colon
                }
            }

            // Bar lines
            '|' => self.bar_line(),

            // Brackets
            '[' => {
                // Check for thick-thin bar [|
                if self.peek() == Some('|') {
                    self.advance();
                    TokenKind::ThickThinBar
                } else {
                    TokenKind::LeftBracket
                }
            }
            ']' => TokenKind::RightBracket,

            // Parentheses
            '(' => {
                // Check for tuplet like (3 or ( 3
                if self.has_digit_ahead() {
                    // Skip whitespace before digit
                    while self.peek().map(|c| c == ' ' || c == '\t').unwrap_or(false) {
                        self.advance();
                    }
                    // Consume digits
                    while self.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                        self.advance();
                    }
                    TokenKind::Tuplet
                } else {
                    TokenKind::LeftParen
                }
            }
            ')' => TokenKind::RightParen,

            // Braces
            '{' => TokenKind::LeftBrace,
            '}' => TokenKind::RightBrace,

            // Accidentals
            '^' => TokenKind::Sharp,
            '=' => TokenKind::Natural,
            '_' => TokenKind::Flat,

            // Octave modifiers
            '\'' => TokenKind::OctaveUp,
            ',' => TokenKind::OctaveDown,

            // Tie
            '-' => TokenKind::Tie,

            // Broken rhythm
            '<' | '>' => {
                // Consume consecutive same characters
                while self.peek() == Some(c) {
                    self.advance();
                }
                TokenKind::BrokenRhythm
            }

            // Slash (for fractions)
            '/' => TokenKind::Slash,

            // Numbers
            '0'..='9' => {
                if self.in_header {
                    self.text()
                } else {
                    while self.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                        self.advance();
                    }
                    TokenKind::NoteLength
                }
            }

            // Notes (uppercase can also be field labels: A=Area, B=Book, C=Composer, etc.)
            'C' | 'D' | 'E' | 'F' | 'G' | 'A' | 'B' => {
                if self.in_header {
                    self.text()
                } else if self.has_colon_ahead() {
                    TokenKind::FieldLabel
                } else {
                    TokenKind::Note
                }
            }

            // Lowercase notes (never field labels)
            'c' | 'd' | 'e' | 'f' | 'g' | 'a' | 'b' => {
                if self.in_header {
                    self.text()
                } else {
                    TokenKind::Note
                }
            }

            // Rest
            'z' | 'Z' => {
                if self.in_header {
                    self.text()
                } else {
                    TokenKind::Rest
                }
            }

            // Field labels (uppercase letters that are not notes: H, I, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y)
            'H' | 'I' | 'K' | 'L' | 'M' | 'N' | 'O' | 'P' | 'Q' | 'R' | 'S' | 'T' | 'U' | 'V'
            | 'W' | 'X' | 'Y' => {
                if self.in_header {
                    self.text()
                } else if self.has_colon_ahead() {
                    // Field label: "X:", "T :", "K  :" etc.
                    TokenKind::FieldLabel
                } else {
                    // Treat as text if not followed by colon
                    self.text()
                }
            }

            // Decorations (!trill!, +fermata+)
            '!' | '+' => {
                if self.in_header {
                    self.text()
                } else {
                    self.decoration(c)
                }
            }

            // Annotations/chord symbols ("CM7", "Am", etc.)
            '"' => {
                if self.in_header {
                    self.text()
                } else {
                    self.annotation()
                }
            }

            // Everything else in header context is text
            _ if self.in_header => self.text(),

            // Unknown character
            _ => TokenKind::Error,
        };

        Token::new(
            kind,
            TextRange::new(
                TextSize::new(start as u32),
                TextSize::new(self.position as u32),
            ),
        )
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.source.len()
    }

    fn peek(&self) -> Option<char> {
        self.source[self.position..].chars().next()
    }

    /// Check if there's a colon ahead, possibly with whitespace in between.
    /// Used for detecting field labels like "T :" or "K  :".
    /// Returns false if the colon is part of ":| " (repeat end).
    fn has_colon_ahead(&self) -> bool {
        let remaining = &self.source[self.position..];
        let mut chars = remaining.chars().peekable();

        while let Some(c) = chars.next() {
            match c {
                ':' => {
                    // Check if this is ":| " (repeat end) - not a field label
                    if chars.peek() == Some(&'|') {
                        return false;
                    }
                    return true;
                }
                ' ' | '\t' => continue,
                _ => return false,
            }
        }
        false
    }

    /// Check if there's a digit ahead, possibly with whitespace in between.
    /// Used for detecting tuplets like "( 3" or "(3".
    fn has_digit_ahead(&self) -> bool {
        let remaining = &self.source[self.position..];
        for c in remaining.chars() {
            match c {
                '0'..='9' => return true,
                ' ' | '\t' => continue,
                _ => return false,
            }
        }
        false
    }

    /// Check if there's a bar `|` ahead, possibly with whitespace in between.
    /// Used for detecting repeat end like ": |" or ":|".
    fn has_bar_ahead(&self) -> bool {
        let remaining = &self.source[self.position..];
        for c in remaining.chars() {
            match c {
                '|' => return true,
                ' ' | '\t' => continue,
                _ => return false,
            }
        }
        false
    }

    /// Check if there's a colon `:` ahead, possibly with whitespace in between.
    /// Used for detecting repeat start like "| :" or "|:".
    fn has_colon_ahead_for_repeat(&self) -> bool {
        let remaining = &self.source[self.position..];
        for c in remaining.chars() {
            match c {
                ':' => return true,
                ' ' | '\t' => continue,
                _ => return false,
            }
        }
        false
    }

    fn advance(&mut self) -> char {
        let c = self.source[self.position..].chars().next().unwrap();
        self.position += c.len_utf8();
        c
    }

    fn make_token(&self, kind: TokenKind, len: usize) -> Token {
        Token::new(
            kind,
            TextRange::new(
                TextSize::new(self.position as u32),
                TextSize::new((self.position + len) as u32),
            ),
        )
    }

    fn whitespace(&mut self) -> TokenKind {
        while let Some(c) = self.peek() {
            if c == ' ' || c == '\t' {
                self.advance();
            } else {
                break;
            }
        }
        TokenKind::Whitespace
    }

    fn comment(&mut self) -> TokenKind {
        while let Some(c) = self.peek() {
            if c == '\n' || c == '\r' {
                break;
            }
            self.advance();
        }
        TokenKind::Comment
    }

    fn bar_line(&mut self) -> TokenKind {
        // Check for |: or | : (with space)
        if self.has_colon_ahead_for_repeat() {
            // Skip whitespace before colon
            while self.peek().map(|c| c == ' ' || c == '\t').unwrap_or(false) {
                self.advance();
            }
            // Consume the :
            self.advance();
            return TokenKind::RepeatStart;
        }

        match self.peek() {
            Some('|') => {
                self.advance();
                TokenKind::DoubleBar
            }
            Some(']') => {
                self.advance();
                TokenKind::ThinThickBar
            }
            _ => TokenKind::Bar,
        }
    }

    fn text(&mut self) -> TokenKind {
        // Consume text until we hit a delimiter
        while let Some(c) = self.peek() {
            match c {
                '\n' | '\r' | '%' => break,
                // Stop at ] for inline fields
                ']' => {
                    self.in_header = false;
                    break;
                }
                _ => {
                    self.advance();
                }
            }
        }
        TokenKind::Text
    }

    fn decoration(&mut self, delimiter: char) -> TokenKind {
        // Consume decoration name until matching delimiter
        while let Some(c) = self.peek() {
            if c == delimiter {
                self.advance(); // consume closing delimiter
                return TokenKind::Decoration;
            }
            if c == '\n' || c == '\r' {
                // Unterminated decoration
                return TokenKind::Error;
            }
            self.advance();
        }
        // Unterminated decoration (reached EOF)
        TokenKind::Error
    }

    fn annotation(&mut self) -> TokenKind {
        // Consume annotation text until closing double quote
        while let Some(c) = self.peek() {
            if c == '"' {
                self.advance(); // consume closing quote
                return TokenKind::Annotation;
            }
            if c == '\n' || c == '\r' {
                // Unterminated annotation
                return TokenKind::Error;
            }
            self.advance();
        }
        // Unterminated annotation (reached EOF)
        TokenKind::Error
    }
}

/// Returns the source text for a token.
pub fn token_text<'a>(source: &'a str, token: &Token) -> &'a str {
    let start = token.range.start().raw() as usize;
    let end = token.range.end().raw() as usize;
    &source[start..end]
}
