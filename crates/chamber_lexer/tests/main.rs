use chamber_lexer::{token_text, Lexer, TokenKind};

fn tokenize(source: &str) -> Vec<TokenKind> {
    Lexer::new(source)
        .tokenize()
        .into_iter()
        .map(|t| t.kind)
        .collect()
}

fn tokenize_with_text(source: &str) -> Vec<(TokenKind, &str)> {
    let tokens = Lexer::new(source).tokenize();
    tokens
        .iter()
        .map(|t| (t.kind, token_text(source, t)))
        .collect()
}

#[test]
fn test_empty_input() {
    let tokens = tokenize("");
    assert_eq!(tokens, vec![TokenKind::Eof]);
}

#[test]
fn test_whitespace() {
    let tokens = tokenize("   \t  ");
    assert_eq!(tokens, vec![TokenKind::Whitespace, TokenKind::Eof]);
}

#[test]
fn test_newlines() {
    let tokens = tokenize("\n\r\n");
    assert_eq!(
        tokens,
        vec![TokenKind::Newline, TokenKind::Newline, TokenKind::Eof]
    );
}

#[test]
fn test_comment() {
    let tokens = tokenize_with_text("% this is a comment");
    assert_eq!(
        tokens,
        vec![
            (TokenKind::Comment, "% this is a comment"),
            (TokenKind::Eof, "")
        ]
    );
}

#[test]
fn test_header_field() {
    let tokens = tokenize_with_text("X:1");
    assert_eq!(
        tokens,
        vec![
            (TokenKind::FieldLabel, "X"),
            (TokenKind::Colon, ":"),
            (TokenKind::Text, "1"),
            (TokenKind::Eof, "")
        ]
    );
}

#[test]
fn test_title_field() {
    let tokens = tokenize_with_text("T:My Song");
    assert_eq!(
        tokens,
        vec![
            (TokenKind::FieldLabel, "T"),
            (TokenKind::Colon, ":"),
            (TokenKind::Text, "My Song"),
            (TokenKind::Eof, "")
        ]
    );
}

#[test]
fn test_notes() {
    let tokens = tokenize("CDEF");
    assert_eq!(
        tokens,
        vec![
            TokenKind::Note,
            TokenKind::Note,
            TokenKind::Note,
            TokenKind::Note,
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_notes_with_text() {
    let source = "CDEFGABc";
    let tokens = tokenize_with_text(source);
    assert_eq!(
        tokens,
        vec![
            (TokenKind::Note, "C"),
            (TokenKind::Note, "D"),
            (TokenKind::Note, "E"),
            (TokenKind::Note, "F"),
            (TokenKind::Note, "G"),
            (TokenKind::Note, "A"),
            (TokenKind::Note, "B"),
            (TokenKind::Note, "c"),
            (TokenKind::Eof, "")
        ]
    );
}

#[test]
fn test_rest() {
    let tokens = tokenize_with_text("z2Z4");
    assert_eq!(
        tokens,
        vec![
            (TokenKind::Rest, "z"),
            (TokenKind::NoteLength, "2"),
            (TokenKind::Rest, "Z"),
            (TokenKind::NoteLength, "4"),
            (TokenKind::Eof, "")
        ]
    );
}

#[test]
fn test_octave_modifiers() {
    let tokens = tokenize("C'D,");
    assert_eq!(
        tokens,
        vec![
            TokenKind::Note,
            TokenKind::OctaveUp,
            TokenKind::Note,
            TokenKind::OctaveDown,
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_accidentals() {
    let tokens = tokenize("^C=D_E");
    assert_eq!(
        tokens,
        vec![
            TokenKind::Sharp,
            TokenKind::Note,
            TokenKind::Natural,
            TokenKind::Note,
            TokenKind::Flat,
            TokenKind::Note,
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_bar_lines() {
    // |: is RepeatStart, :| is RepeatEnd, || is DoubleBar
    let tokens = tokenize("|::|||");
    assert_eq!(
        tokens,
        vec![
            TokenKind::RepeatStart,
            TokenKind::RepeatEnd,
            TokenKind::DoubleBar,
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_simple_bar() {
    let tokens = tokenize("|C|D|");
    assert_eq!(
        tokens,
        vec![
            TokenKind::Bar,
            TokenKind::Note,
            TokenKind::Bar,
            TokenKind::Note,
            TokenKind::Bar,
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_thin_thick_bar() {
    let tokens = tokenize("|]");
    assert_eq!(tokens, vec![TokenKind::ThinThickBar, TokenKind::Eof]);
}

#[test]
fn test_thick_thin_bar() {
    let tokens = tokenize("[|");
    assert_eq!(tokens, vec![TokenKind::ThickThinBar, TokenKind::Eof]);
}

#[test]
fn test_chord() {
    let tokens = tokenize("[CEG]");
    assert_eq!(
        tokens,
        vec![
            TokenKind::LeftBracket,
            TokenKind::Note,
            TokenKind::Note,
            TokenKind::Note,
            TokenKind::RightBracket,
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_slur() {
    let tokens = tokenize("(CDE)");
    assert_eq!(
        tokens,
        vec![
            TokenKind::LeftParen,
            TokenKind::Note,
            TokenKind::Note,
            TokenKind::Note,
            TokenKind::RightParen,
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_tuplet() {
    let tokens = tokenize("(3CDE");
    assert_eq!(
        tokens,
        vec![
            TokenKind::Tuplet,
            TokenKind::Note,
            TokenKind::Note,
            TokenKind::Note,
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_grace_notes() {
    let tokens = tokenize("{g}C");
    assert_eq!(
        tokens,
        vec![
            TokenKind::LeftBrace,
            TokenKind::Note,
            TokenKind::RightBrace,
            TokenKind::Note,
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_tie() {
    let tokens = tokenize("C-C");
    assert_eq!(
        tokens,
        vec![
            TokenKind::Note,
            TokenKind::Tie,
            TokenKind::Note,
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_broken_rhythm() {
    let tokens = tokenize("C>DC<D");
    assert_eq!(
        tokens,
        vec![
            TokenKind::Note,
            TokenKind::BrokenRhythm,
            TokenKind::Note,
            TokenKind::Note,
            TokenKind::BrokenRhythm,
            TokenKind::Note,
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_note_length() {
    let tokens = tokenize_with_text("C2D/2E3/4");
    assert_eq!(
        tokens,
        vec![
            (TokenKind::Note, "C"),
            (TokenKind::NoteLength, "2"),
            (TokenKind::Note, "D"),
            (TokenKind::Slash, "/"),
            (TokenKind::NoteLength, "2"),
            (TokenKind::Note, "E"),
            (TokenKind::NoteLength, "3"),
            (TokenKind::Slash, "/"),
            (TokenKind::NoteLength, "4"),
            (TokenKind::Eof, "")
        ]
    );
}

#[test]
fn test_meter_field() {
    let tokens = tokenize_with_text("M:4/4");
    assert_eq!(
        tokens,
        vec![
            (TokenKind::FieldLabel, "M"),
            (TokenKind::Colon, ":"),
            (TokenKind::Text, "4/4"),
            (TokenKind::Eof, "")
        ]
    );
}

#[test]
fn test_full_tune_header() {
    let source = "X:1\nT:Test\nM:4/4\nK:C";
    let tokens = tokenize(source);
    assert_eq!(
        tokens,
        vec![
            TokenKind::FieldLabel, // X
            TokenKind::Colon,
            TokenKind::Text, // 1
            TokenKind::Newline,
            TokenKind::FieldLabel, // T
            TokenKind::Colon,
            TokenKind::Text, // Test
            TokenKind::Newline,
            TokenKind::FieldLabel, // M
            TokenKind::Colon,
            TokenKind::Text, // 4/4
            TokenKind::Newline,
            TokenKind::FieldLabel, // K
            TokenKind::Colon,
            TokenKind::Text, // C
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_line_continuation() {
    let tokens = tokenize("C\\\nD");
    assert_eq!(
        tokens,
        vec![
            TokenKind::Note,
            TokenKind::LineContinuation,
            TokenKind::Newline,
            TokenKind::Note,
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_mixed_content() {
    let source = "X:1\n|:CDEF|GABc:|";
    let tokens = tokenize(source);
    assert_eq!(
        tokens,
        vec![
            TokenKind::FieldLabel,
            TokenKind::Colon,
            TokenKind::Text,
            TokenKind::Newline,
            TokenKind::RepeatStart,
            TokenKind::Note,
            TokenKind::Note,
            TokenKind::Note,
            TokenKind::Note,
            TokenKind::Bar,
            TokenKind::Note,
            TokenKind::Note,
            TokenKind::Note,
            TokenKind::Note,
            TokenKind::RepeatEnd,
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_is_trivia() {
    assert!(TokenKind::Whitespace.is_trivia());
    assert!(TokenKind::Comment.is_trivia());
    assert!(TokenKind::Newline.is_trivia()); // Newline is now trivia for CST
    assert!(!TokenKind::Note.is_trivia());
}
