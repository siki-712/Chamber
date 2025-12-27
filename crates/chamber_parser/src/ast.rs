use chamber_text_size::TextRange;

/// A complete ABC tune.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tune {
    /// Header fields (X:, T:, M:, K:, etc.)
    pub header: Header,
    /// Music body
    pub body: Body,
    /// Span of the entire tune
    pub range: TextRange,
}

/// Collection of header fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    pub fields: Vec<HeaderField>,
    pub range: TextRange,
}

/// A single header field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeaderField {
    /// Field type (X, T, M, K, L, Q, C, etc.)
    pub kind: HeaderFieldKind,
    /// Field value as text
    pub value: String,
    pub range: TextRange,
}

/// Types of header fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HeaderFieldKind {
    /// X: Reference number (required, must be first)
    ReferenceNumber,
    /// T: Title
    Title,
    /// C: Composer
    Composer,
    /// M: Meter (time signature)
    Meter,
    /// L: Default note length
    UnitNoteLength,
    /// Q: Tempo
    Tempo,
    /// K: Key (required, must be last in header)
    Key,
    /// Other fields
    Other(char),
}

impl HeaderFieldKind {
    pub fn from_char(c: char) -> Self {
        match c {
            'X' => Self::ReferenceNumber,
            'T' => Self::Title,
            'C' => Self::Composer,
            'M' => Self::Meter,
            'L' => Self::UnitNoteLength,
            'Q' => Self::Tempo,
            'K' => Self::Key,
            _ => Self::Other(c),
        }
    }
}

/// The music body of a tune.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Body {
    pub elements: Vec<MusicElement>,
    pub range: TextRange,
}

/// A music element in the body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MusicElement {
    Note(Note),
    Rest(Rest),
    Chord(Chord),
    BarLine(BarLine),
    Tuplet(Tuplet),
    Slur(Slur),
    GraceNotes(GraceNotes),
    BrokenRhythm(BrokenRhythm),
    Tie(Tie),
}

/// A single note.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note {
    /// Pitch class (C, D, E, F, G, A, B)
    pub pitch: Pitch,
    /// Octave modifier (positive = up, negative = down)
    pub octave: i8,
    /// Accidental (sharp, flat, natural)
    pub accidental: Option<Accidental>,
    /// Duration as a fraction (numerator, denominator)
    pub duration: Option<Duration>,
    pub range: TextRange,
}

/// Pitch class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Pitch {
    C,
    D,
    E,
    F,
    G,
    A,
    B,
}

impl Pitch {
    pub fn from_char(c: char) -> Option<(Self, i8)> {
        match c {
            'C' => Some((Self::C, 0)),
            'D' => Some((Self::D, 0)),
            'E' => Some((Self::E, 0)),
            'F' => Some((Self::F, 0)),
            'G' => Some((Self::G, 0)),
            'A' => Some((Self::A, 0)),
            'B' => Some((Self::B, 0)),
            'c' => Some((Self::C, 1)),
            'd' => Some((Self::D, 1)),
            'e' => Some((Self::E, 1)),
            'f' => Some((Self::F, 1)),
            'g' => Some((Self::G, 1)),
            'a' => Some((Self::A, 1)),
            'b' => Some((Self::B, 1)),
            _ => None,
        }
    }
}

/// Accidental.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Accidental {
    Sharp,
    DoubleSharp,
    Flat,
    DoubleFlat,
    Natural,
}

/// Note duration as a fraction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Duration {
    pub numerator: u32,
    pub denominator: u32,
}

impl Duration {
    pub fn new(numerator: u32, denominator: u32) -> Self {
        Self { numerator, denominator }
    }
}

impl Default for Duration {
    fn default() -> Self {
        Self { numerator: 1, denominator: 1 }
    }
}

/// A rest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rest {
    /// Whether this is a multi-measure rest (Z vs z)
    pub multi_measure: bool,
    pub duration: Option<Duration>,
    pub range: TextRange,
}

/// A chord (multiple notes played together).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chord {
    pub notes: Vec<Note>,
    pub duration: Option<Duration>,
    pub range: TextRange,
}

/// Bar line types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BarLineKind {
    Single,
    Double,
    RepeatStart,
    RepeatEnd,
    ThinThick,
    ThickThin,
}

/// A bar line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BarLine {
    pub kind: BarLineKind,
    pub range: TextRange,
}

/// A tuplet (e.g., triplet).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tuplet {
    /// Tuplet ratio (e.g., 3 for triplet)
    pub ratio: u32,
    /// Notes in the tuplet
    pub notes: Vec<Note>,
    pub range: TextRange,
}

/// A slur grouping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Slur {
    pub elements: Vec<MusicElement>,
    pub range: TextRange,
}

/// Grace notes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraceNotes {
    pub notes: Vec<Note>,
    pub range: TextRange,
}

/// Broken rhythm marker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrokenRhythm {
    /// Direction: true for >, false for <
    pub dotted_first: bool,
    /// Number of markers (> or >>)
    pub count: u32,
    pub range: TextRange,
}

/// A tie between notes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tie {
    pub range: TextRange,
}
