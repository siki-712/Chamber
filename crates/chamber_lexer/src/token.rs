use chamber_syntax::SyntaxKind;
use chamber_text_size::TextRange;
use serde::{Deserialize, Serialize};

/// A token produced by the lexer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Token {
    /// The kind of token.
    pub kind: TokenKind,
    /// The range of the token in the source text.
    pub range: TextRange,
}

impl Token {
    /// Creates a new token.
    pub fn new(kind: TokenKind, range: TextRange) -> Self {
        Self { kind, range }
    }
}

/// The kind of token in ABC notation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TokenKind {
    // Header fields
    /// Field label (X, T, M, K, L, Q, C, etc.)
    FieldLabel,
    /// Colon after field label
    Colon,

    // Notes
    /// Note name (C, D, E, F, G, A, B, c, d, e, f, g, a, b)
    Note,
    /// Rest (z or Z)
    Rest,
    /// Octave modifier up (')
    OctaveUp,
    /// Octave modifier down (,)
    OctaveDown,
    /// Note length number
    NoteLength,

    // Accidentals
    /// Sharp (^)
    Sharp,
    /// Natural (=)
    Natural,
    /// Flat (_)
    Flat,

    // Bar lines
    /// Simple bar line (|)
    Bar,
    /// Double bar line (||)
    DoubleBar,
    /// Repeat start (|:)
    RepeatStart,
    /// Repeat end (:|)
    RepeatEnd,
    /// Thin-thick double bar (|])
    ThinThickBar,
    /// Thick-thin double bar ([|)
    ThickThinBar,

    // Grouping
    /// Left bracket ([) for chords
    LeftBracket,
    /// Right bracket (])
    RightBracket,
    /// Left paren (() for slurs
    LeftParen,
    /// Right paren ())
    RightParen,
    /// Left brace ({) for grace notes
    LeftBrace,
    /// Right brace (})
    RightBrace,

    // Other symbols
    /// Tie (-)
    Tie,
    /// Broken rhythm markers (< or >)
    BrokenRhythm,
    /// Tuplet marker (e.g., (3)
    Tuplet,
    /// Decoration (!trill!, +fermata+, etc.)
    Decoration,

    // Text and numbers
    /// Plain text content
    Text,
    /// Number
    Number,
    /// Fraction slash (/)
    Slash,

    // Whitespace and structure
    /// Whitespace
    Whitespace,
    /// Newline
    Newline,
    /// Comment (% ...)
    Comment,
    /// Line continuation (\)
    LineContinuation,

    // Special
    /// End of file
    Eof,
    /// Unknown/error token
    Error,
}

impl TokenKind {
    /// Returns true if this token is trivia (whitespace or comment).
    pub fn is_trivia(self) -> bool {
        matches!(
            self,
            TokenKind::Whitespace | TokenKind::Comment | TokenKind::Newline
        )
    }

    /// Converts this TokenKind to a SyntaxKind.
    pub fn to_syntax_kind(self) -> SyntaxKind {
        match self {
            TokenKind::Whitespace => SyntaxKind::WHITESPACE,
            TokenKind::Newline => SyntaxKind::NEWLINE,
            TokenKind::Comment => SyntaxKind::COMMENT,
            TokenKind::LineContinuation => SyntaxKind::LINE_CONTINUATION,
            TokenKind::FieldLabel => SyntaxKind::FIELD_LABEL,
            TokenKind::Colon => SyntaxKind::COLON,
            TokenKind::Note => SyntaxKind::NOTE_NAME,
            TokenKind::Rest => SyntaxKind::REST,
            TokenKind::OctaveUp => SyntaxKind::OCTAVE_UP,
            TokenKind::OctaveDown => SyntaxKind::OCTAVE_DOWN,
            TokenKind::NoteLength => SyntaxKind::NUMBER,
            TokenKind::Sharp => SyntaxKind::SHARP,
            TokenKind::Natural => SyntaxKind::NATURAL,
            TokenKind::Flat => SyntaxKind::FLAT,
            TokenKind::Bar => SyntaxKind::BAR,
            TokenKind::DoubleBar => SyntaxKind::DOUBLE_BAR,
            TokenKind::RepeatStart => SyntaxKind::REPEAT_START,
            TokenKind::RepeatEnd => SyntaxKind::REPEAT_END,
            TokenKind::ThinThickBar => SyntaxKind::THIN_THICK_BAR,
            TokenKind::ThickThinBar => SyntaxKind::THICK_THIN_BAR,
            TokenKind::LeftBracket => SyntaxKind::L_BRACKET,
            TokenKind::RightBracket => SyntaxKind::R_BRACKET,
            TokenKind::LeftParen => SyntaxKind::L_PAREN,
            TokenKind::RightParen => SyntaxKind::R_PAREN,
            TokenKind::LeftBrace => SyntaxKind::L_BRACE,
            TokenKind::RightBrace => SyntaxKind::R_BRACE,
            TokenKind::Tie => SyntaxKind::TIE,
            TokenKind::BrokenRhythm => SyntaxKind::BROKEN_RHYTHM,
            TokenKind::Tuplet => SyntaxKind::TUPLET_MARKER,
            TokenKind::Decoration => SyntaxKind::DECORATION,
            TokenKind::Text => SyntaxKind::TEXT,
            TokenKind::Number => SyntaxKind::NUMBER,
            TokenKind::Slash => SyntaxKind::SLASH,
            TokenKind::Eof => SyntaxKind::EOF,
            TokenKind::Error => SyntaxKind::ERROR,
        }
    }
}
