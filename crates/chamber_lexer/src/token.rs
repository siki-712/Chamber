use chamber_text_size::TextRange;

/// A token produced by the lexer.
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
        matches!(self, TokenKind::Whitespace | TokenKind::Comment)
    }
}
