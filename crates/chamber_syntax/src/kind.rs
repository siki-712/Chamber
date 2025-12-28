//! Syntax element kinds for ABC notation.

#![allow(non_camel_case_types)]

/// Kind of a syntax element (token or node).
///
/// Values 0-127 are tokens, values 128+ are composite nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum SyntaxKind {
    // ==========================================================
    // Tokens (0-127)
    // ==========================================================

    // --- Trivia ---
    /// Whitespace (spaces, tabs)
    WHITESPACE = 0,
    /// Newline (\n, \r\n)
    NEWLINE,
    /// Comment (% ...)
    COMMENT,
    /// Line continuation (\)
    LINE_CONTINUATION,

    // --- Header ---
    /// Field label (X, T, M, K, L, Q, C, etc.)
    FIELD_LABEL,
    /// Colon separator
    COLON,

    // --- Notes ---
    /// Note name (C, D, E, F, G, A, B, c, d, e, f, g, a, b)
    NOTE_NAME,
    /// Rest (z or Z)
    REST,
    /// Octave up modifier (')
    OCTAVE_UP,
    /// Octave down modifier (,)
    OCTAVE_DOWN,

    // --- Accidentals ---
    /// Sharp (^)
    SHARP,
    /// Natural (=)
    NATURAL,
    /// Flat (_)
    FLAT,

    // --- Bar lines ---
    /// Single bar (|)
    BAR,
    /// Double bar (||)
    DOUBLE_BAR,
    /// Repeat start (|:)
    REPEAT_START,
    /// Repeat end (:|)
    REPEAT_END,
    /// Thin-thick bar (|])
    THIN_THICK_BAR,
    /// Thick-thin bar ([|)
    THICK_THIN_BAR,

    // --- Delimiters ---
    /// Left bracket ([)
    L_BRACKET,
    /// Right bracket (])
    R_BRACKET,
    /// Left parenthesis (()
    L_PAREN,
    /// Right parenthesis ())
    R_PAREN,
    /// Left brace ({)
    L_BRACE,
    /// Right brace (})
    R_BRACE,

    // --- Other symbols ---
    /// Tie (-)
    TIE,
    /// Broken rhythm (< or >)
    BROKEN_RHYTHM,
    /// Tuplet marker ((3, (2, etc.)
    TUPLET_MARKER,
    /// Decoration (!trill!, +fermata+, etc.)
    DECORATION,
    /// Number
    NUMBER,
    /// Slash (/)
    SLASH,
    /// Text content
    TEXT,

    // --- Special ---
    /// End of file
    EOF,
    /// Error/unknown token
    ERROR,

    // ==========================================================
    // Nodes (128+)
    // ==========================================================

    // --- Root ---
    /// Complete tune
    TUNE = 128,

    // --- Header structure ---
    /// Header section
    HEADER,
    /// Single header field (X:1, T:Title, etc.)
    HEADER_FIELD,

    // --- Body structure ---
    /// Music body
    BODY,

    // --- Music elements ---
    /// A note with optional accidentals, octave modifiers, duration
    NOTE,
    /// A rest element
    REST_NODE,
    /// A chord ([CEG])
    CHORD,
    /// A bar line
    BAR_LINE,
    /// A tuplet ((3CDE)
    TUPLET,
    /// A slur ((CDE))
    SLUR,
    /// Grace notes ({cde})
    GRACE_NOTES,
    /// Broken rhythm element
    BROKEN_RHYTHM_NODE,
    /// Tie element
    TIE_NODE,
    /// Inline field ([M:3/4])
    INLINE_FIELD,

    // --- Sub-elements ---
    /// Duration (2, /2, 3/4, etc.)
    DURATION,
    /// Accidental group (^, ^^, =, _, __)
    ACCIDENTAL,
    /// Decoration (!trill!)
    DECORATION_NODE,
}

impl SyntaxKind {
    /// Returns true if this is a token kind (not a composite node).
    #[inline]
    pub fn is_token(self) -> bool {
        (self as u16) < 128
    }

    /// Returns true if this is a composite node kind.
    #[inline]
    pub fn is_node(self) -> bool {
        (self as u16) >= 128
    }

    /// Returns true if this is trivia (whitespace, comments, etc.).
    #[inline]
    pub fn is_trivia(self) -> bool {
        matches!(
            self,
            Self::WHITESPACE | Self::NEWLINE | Self::COMMENT | Self::LINE_CONTINUATION
        )
    }

    /// Returns true if this is a bar line token.
    #[inline]
    pub fn is_bar_line(self) -> bool {
        matches!(
            self,
            Self::BAR
                | Self::DOUBLE_BAR
                | Self::REPEAT_START
                | Self::REPEAT_END
                | Self::THIN_THICK_BAR
                | Self::THICK_THIN_BAR
        )
    }

    /// Returns true if this is an accidental token.
    #[inline]
    pub fn is_accidental(self) -> bool {
        matches!(self, Self::SHARP | Self::NATURAL | Self::FLAT)
    }

    /// Returns true if this is a bracket/delimiter token.
    #[inline]
    pub fn is_bracket(self) -> bool {
        matches!(
            self,
            Self::L_BRACKET
                | Self::R_BRACKET
                | Self::L_PAREN
                | Self::R_PAREN
                | Self::L_BRACE
                | Self::R_BRACE
        )
    }
}

impl std::fmt::Display for SyntaxKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_token() {
        assert!(SyntaxKind::WHITESPACE.is_token());
        assert!(SyntaxKind::NOTE_NAME.is_token());
        assert!(SyntaxKind::EOF.is_token());
        assert!(!SyntaxKind::TUNE.is_token());
        assert!(!SyntaxKind::NOTE.is_token());
    }

    #[test]
    fn test_is_node() {
        assert!(!SyntaxKind::WHITESPACE.is_node());
        assert!(SyntaxKind::TUNE.is_node());
        assert!(SyntaxKind::NOTE.is_node());
    }

    #[test]
    fn test_is_trivia() {
        assert!(SyntaxKind::WHITESPACE.is_trivia());
        assert!(SyntaxKind::NEWLINE.is_trivia());
        assert!(SyntaxKind::COMMENT.is_trivia());
        assert!(!SyntaxKind::NOTE_NAME.is_trivia());
    }
}
