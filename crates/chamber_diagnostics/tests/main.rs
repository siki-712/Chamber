use chamber_diagnostics::*;
use chamber_text_size::{TextRange, TextSize};

#[test]
fn test_severity_display() {
    assert_eq!(Severity::Error.as_str(), "error");
    assert_eq!(Severity::Warning.as_str(), "warning");
    assert_eq!(Severity::Info.as_str(), "info");
}

#[test]
fn test_severity_ordering() {
    assert!(Severity::Error > Severity::Warning);
    assert!(Severity::Warning > Severity::Info);
}

#[test]
fn test_diagnostic_code_display() {
    assert_eq!(DiagnosticCode::UnexpectedCharacter.code(), "L001");
    assert_eq!(DiagnosticCode::MissingReferenceNumber.code(), "H001");
    assert_eq!(DiagnosticCode::UnclosedChord.code(), "M001");
}

#[test]
fn test_diagnostic_code_severity() {
    assert_eq!(
        DiagnosticCode::UnexpectedCharacter.default_severity(),
        Severity::Error
    );
    assert_eq!(
        DiagnosticCode::UnusualOctave.default_severity(),
        Severity::Warning
    );
}

#[test]
fn test_diagnostic_creation() {
    let range = TextRange::new(TextSize::new(0), TextSize::new(5));
    let diag = Diagnostic::error(DiagnosticCode::UnclosedChord, range, "test message");

    assert_eq!(diag.code, DiagnosticCode::UnclosedChord);
    assert_eq!(diag.severity, Severity::Error);
    assert_eq!(diag.message, "test message");
    assert_eq!(diag.range, range);
}

#[test]
fn test_diagnostic_with_label() {
    let range = TextRange::new(TextSize::new(0), TextSize::new(5));
    let label_range = TextRange::new(TextSize::new(0), TextSize::new(1));

    let diag = Diagnostic::error(DiagnosticCode::UnclosedChord, range, "test")
        .with_label(label_range, "opening bracket here");

    assert_eq!(diag.labels.len(), 1);
    assert_eq!(diag.labels[0].range, label_range);
    assert_eq!(diag.labels[0].message, "opening bracket here");
}

#[test]
fn test_diagnostic_with_note() {
    let range = TextRange::new(TextSize::new(0), TextSize::new(5));
    let diag = Diagnostic::error(DiagnosticCode::UnclosedChord, range, "test")
        .with_note("consider adding a closing bracket");

    assert_eq!(diag.notes.len(), 1);
    assert_eq!(diag.notes[0], "consider adding a closing bracket");
}

#[test]
fn test_diagnostic_bag() {
    let mut bag = DiagnosticBag::new();
    let range = TextRange::new(TextSize::new(0), TextSize::new(5));

    bag.report(Diagnostic::error(
        DiagnosticCode::UnclosedChord,
        range,
        "error 1",
    ));
    bag.report(Diagnostic::warning(
        DiagnosticCode::EmptyChord,
        range,
        "warning 1",
    ));
    bag.report(Diagnostic::error(
        DiagnosticCode::UnclosedSlur,
        range,
        "error 2",
    ));

    assert_eq!(bag.len(), 3);
    assert_eq!(bag.error_count(), 2);
    assert_eq!(bag.warning_count(), 1);
    assert!(bag.has_errors());
    assert!(bag.has_warnings());
}

#[test]
fn test_diagnostic_bag_empty() {
    let bag = DiagnosticBag::new();

    assert!(bag.is_empty());
    assert!(!bag.has_errors());
    assert!(!bag.has_warnings());
    assert_eq!(bag.error_count(), 0);
    assert_eq!(bag.warning_count(), 0);
}

#[test]
fn test_line_index_single_line() {
    let source = "CDEFGAB";
    let index = LineIndex::new(source);

    assert_eq!(index.line_count(), 1);
    assert_eq!(
        index.line_col(TextSize::new(0)),
        LineCol { line: 0, col: 0 }
    );
    assert_eq!(
        index.line_col(TextSize::new(3)),
        LineCol { line: 0, col: 3 }
    );
}

#[test]
fn test_line_index_multiple_lines() {
    let source = "X:1\nT:Test\nK:C";
    let index = LineIndex::new(source);

    assert_eq!(index.line_count(), 3);

    // First line: X:1
    assert_eq!(
        index.line_col(TextSize::new(0)),
        LineCol { line: 0, col: 0 }
    );
    assert_eq!(
        index.line_col(TextSize::new(2)),
        LineCol { line: 0, col: 2 }
    );

    // Second line: T:Test (starts at byte 4)
    assert_eq!(
        index.line_col(TextSize::new(4)),
        LineCol { line: 1, col: 0 }
    );
    assert_eq!(
        index.line_col(TextSize::new(6)),
        LineCol { line: 1, col: 2 }
    );

    // Third line: K:C (starts at byte 11)
    assert_eq!(
        index.line_col(TextSize::new(11)),
        LineCol { line: 2, col: 0 }
    );
}

#[test]
fn test_line_index_line_start() {
    let source = "X:1\nT:Test\nK:C";
    let index = LineIndex::new(source);

    assert_eq!(index.line_start(0), Some(TextSize::new(0)));
    assert_eq!(index.line_start(1), Some(TextSize::new(4)));
    assert_eq!(index.line_start(2), Some(TextSize::new(11)));
    assert_eq!(index.line_start(3), None);
}

#[test]
fn test_line_index_line_text() {
    let source = "X:1\nT:Test\nK:C";
    let index = LineIndex::new(source);

    assert_eq!(index.line_text(0, source), Some("X:1"));
    assert_eq!(index.line_text(1, source), Some("T:Test"));
    assert_eq!(index.line_text(2, source), Some("K:C"));
    assert_eq!(index.line_text(3, source), None);
}

#[test]
fn test_line_col_display() {
    let lc = LineCol { line: 0, col: 5 };
    assert_eq!(format!("{}", lc), "1:6");

    let lc2 = LineCol { line: 2, col: 10 };
    assert_eq!(format!("{}", lc2), "3:11");
}
