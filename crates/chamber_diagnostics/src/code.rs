use crate::Severity;

/// Diagnostic codes organized by category.
///
/// - L: Lexer errors (L001-L099)
/// - H: Header errors (H001-H099)
/// - M: Music body errors (M001-M099)
/// - S: Structural errors (S001-S099)
/// - W: Warnings (W001-W099)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticCode {
    // =========================================
    // Lexer errors (L001-L099)
    // =========================================
    /// L001: Unexpected character in source.
    UnexpectedCharacter,

    // =========================================
    // Header errors (H001-H099)
    // =========================================
    /// H001: Missing reference number field (X:).
    MissingReferenceNumber,
    /// H002: Missing key field (K:).
    MissingKeyField,
    /// H003: Duplicate reference number field.
    DuplicateReferenceNumber,
    /// H004: Invalid field order (X: must be first, K: must be last in header).
    InvalidFieldOrder,
    /// H005: Invalid meter value.
    InvalidMeterValue,
    /// H006: Invalid tempo value.
    InvalidTempo,
    /// H007: Invalid unit note length.
    InvalidUnitNoteLength,
    /// H008: Invalid key signature.
    InvalidKeySignature,
    /// H009: Missing title field (T:).
    MissingTitle,
    /// H010: Empty title field.
    EmptyTitle,
    /// H011: Empty reference number field.
    EmptyReferenceNumber,
    /// H012: Invalid reference number (not a number).
    InvalidReferenceNumber,

    // =========================================
    // Music body errors (M001-M099)
    // =========================================
    /// M001: Unclosed chord ([ without ]).
    UnclosedChord,
    /// M002: Unclosed slur (( without )).
    UnclosedSlur,
    /// M003: Unclosed grace notes ({ without }).
    UnclosedGraceNotes,
    /// M004: Unexpected closing bracket (]).
    UnexpectedClosingBracket,
    /// M005: Unexpected closing parenthesis ()).
    UnexpectedClosingParen,
    /// M006: Unexpected closing brace (}).
    UnexpectedClosingBrace,
    /// M007: Invalid note name.
    InvalidNoteName,
    /// M008: Invalid accidental.
    InvalidAccidental,
    /// M009: Invalid duration.
    InvalidDuration,
    /// M010: Empty chord.
    EmptyChord,
    /// M011: Empty tuplet.
    EmptyTuplet,
    /// M012: Tuplet note count mismatch.
    TupletNoteMismatch,

    // =========================================
    // Structural errors (S001-S099)
    // =========================================
    /// S001: Empty tune.
    EmptyTune,
    /// S002: Unexpected token.
    UnexpectedToken,

    // =========================================
    // Warnings (W001-W099)
    // =========================================
    /// W001: Unusual octave (very high or very low).
    UnusualOctave,
    /// W002: Suspicious duration (very large).
    SuspiciousDuration,
}

impl DiagnosticCode {
    /// Returns the string code (e.g., "L001", "H002").
    pub fn code(&self) -> &'static str {
        match self {
            // Lexer
            DiagnosticCode::UnexpectedCharacter => "L001",

            // Header
            DiagnosticCode::MissingReferenceNumber => "H001",
            DiagnosticCode::MissingKeyField => "H002",
            DiagnosticCode::DuplicateReferenceNumber => "H003",
            DiagnosticCode::InvalidFieldOrder => "H004",
            DiagnosticCode::InvalidMeterValue => "H005",
            DiagnosticCode::InvalidTempo => "H006",
            DiagnosticCode::InvalidUnitNoteLength => "H007",
            DiagnosticCode::InvalidKeySignature => "H008",
            DiagnosticCode::MissingTitle => "H009",
            DiagnosticCode::EmptyTitle => "H010",
            DiagnosticCode::EmptyReferenceNumber => "H011",
            DiagnosticCode::InvalidReferenceNumber => "H012",

            // Music
            DiagnosticCode::UnclosedChord => "M001",
            DiagnosticCode::UnclosedSlur => "M002",
            DiagnosticCode::UnclosedGraceNotes => "M003",
            DiagnosticCode::UnexpectedClosingBracket => "M004",
            DiagnosticCode::UnexpectedClosingParen => "M005",
            DiagnosticCode::UnexpectedClosingBrace => "M006",
            DiagnosticCode::InvalidNoteName => "M007",
            DiagnosticCode::InvalidAccidental => "M008",
            DiagnosticCode::InvalidDuration => "M009",
            DiagnosticCode::EmptyChord => "M010",
            DiagnosticCode::EmptyTuplet => "M011",
            DiagnosticCode::TupletNoteMismatch => "M012",

            // Structural
            DiagnosticCode::EmptyTune => "S001",
            DiagnosticCode::UnexpectedToken => "S002",

            // Warnings
            DiagnosticCode::UnusualOctave => "W001",
            DiagnosticCode::SuspiciousDuration => "W002",
        }
    }

    /// Returns the default severity for this diagnostic.
    pub fn default_severity(&self) -> Severity {
        match self {
            DiagnosticCode::UnusualOctave
            | DiagnosticCode::SuspiciousDuration
            | DiagnosticCode::MissingTitle
            | DiagnosticCode::EmptyTitle
            | DiagnosticCode::EmptyChord
            | DiagnosticCode::EmptyTuplet
            | DiagnosticCode::TupletNoteMismatch
            | DiagnosticCode::InvalidFieldOrder => Severity::Warning,
            _ => Severity::Error,
        }
    }

    /// Returns the default message template for this diagnostic.
    pub fn message_template(&self) -> &'static str {
        match self {
            // Lexer
            DiagnosticCode::UnexpectedCharacter => "unexpected character",

            // Header
            DiagnosticCode::MissingReferenceNumber => "missing reference number field (X:)",
            DiagnosticCode::MissingKeyField => "missing key field (K:)",
            DiagnosticCode::DuplicateReferenceNumber => "duplicate reference number field",
            DiagnosticCode::InvalidFieldOrder => "invalid field order",
            DiagnosticCode::InvalidMeterValue => "invalid meter value",
            DiagnosticCode::InvalidTempo => "invalid tempo value",
            DiagnosticCode::InvalidUnitNoteLength => "invalid unit note length",
            DiagnosticCode::InvalidKeySignature => "invalid key signature",
            DiagnosticCode::MissingTitle => "missing title field (T:)",
            DiagnosticCode::EmptyTitle => "empty title field",
            DiagnosticCode::EmptyReferenceNumber => "empty reference number field",
            DiagnosticCode::InvalidReferenceNumber => "invalid reference number (must be a positive integer)",

            // Music
            DiagnosticCode::UnclosedChord => "unclosed chord, missing ']'",
            DiagnosticCode::UnclosedSlur => "unclosed slur, missing ')'",
            DiagnosticCode::UnclosedGraceNotes => "unclosed grace notes, missing '}'",
            DiagnosticCode::UnexpectedClosingBracket => "unexpected ']' without matching '['",
            DiagnosticCode::UnexpectedClosingParen => "unexpected ')' without matching '('",
            DiagnosticCode::UnexpectedClosingBrace => "unexpected '}' without matching '{'",
            DiagnosticCode::InvalidNoteName => "invalid note name",
            DiagnosticCode::InvalidAccidental => "invalid accidental",
            DiagnosticCode::InvalidDuration => "invalid duration",
            DiagnosticCode::EmptyChord => "empty chord",
            DiagnosticCode::EmptyTuplet => "empty tuplet",
            DiagnosticCode::TupletNoteMismatch => "tuplet note count does not match ratio",

            // Structural
            DiagnosticCode::EmptyTune => "empty tune",
            DiagnosticCode::UnexpectedToken => "unexpected token",

            // Warnings
            DiagnosticCode::UnusualOctave => "unusual octave (very high or very low)",
            DiagnosticCode::SuspiciousDuration => "suspicious duration (very large)",
        }
    }
}

impl std::fmt::Display for DiagnosticCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code())
    }
}
