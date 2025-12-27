use chamber_analyzer::analyze;
use chamber_diagnostics::DiagnosticCode;
use chamber_parser::parse;

#[test]
fn test_unknown_decoration_error() {
    let tune = parse("X:1\nK:C\n!trillx!C");
    let diagnostics = analyze(&tune);

    let errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.code == DiagnosticCode::UnknownDecoration)
        .collect();

    assert_eq!(errors.len(), 1);
    assert!(errors[0].message.contains("trillx"));
    assert!(errors[0].message.contains("trill")); // Should suggest 'trill'
}

#[test]
fn test_unknown_decoration_no_suggestion() {
    let tune = parse("X:1\nK:C\n!xyzabc!C");
    let diagnostics = analyze(&tune);

    let errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.code == DiagnosticCode::UnknownDecoration)
        .collect();

    assert_eq!(errors.len(), 1);
    assert!(errors[0].message.contains("xyzabc"));
    // No suggestion for completely unknown decoration
    assert!(!errors[0].message.contains("did you mean"));
}

#[test]
fn test_valid_decorations_no_error() {
    let decorations = [
        "trill", "fermata", "accent", "staccato", "tenuto",
        "mordent", "turn", "roll", "upbow", "downbow",
        "p", "f", "ff", "pp", "mp", "mf", "sfz",
        "coda", "segno", "fine",
    ];

    for deco in decorations {
        let source = format!("X:1\nK:C\n!{}!C", deco);
        let tune = parse(&source);
        let diagnostics = analyze(&tune);

        let errors: Vec<_> = diagnostics
            .iter()
            .filter(|d| d.code == DiagnosticCode::UnknownDecoration)
            .collect();

        assert!(
            errors.is_empty(),
            "Expected no error for decoration '{}', got: {:?}",
            deco,
            errors
        );
    }
}

#[test]
fn test_decoration_case_insensitive() {
    let tune = parse("X:1\nK:C\n!TRILL!C !Fermata!D");
    let diagnostics = analyze(&tune);

    let errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.code == DiagnosticCode::UnknownDecoration)
        .collect();

    assert!(errors.is_empty(), "Decorations should be case-insensitive");
}

#[test]
fn test_decoration_in_chord() {
    let tune = parse("X:1\nK:C\n!trillx![CEG]");
    let diagnostics = analyze(&tune);

    let errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.code == DiagnosticCode::UnknownDecoration)
        .collect();

    assert_eq!(errors.len(), 1);
    assert!(errors[0].message.contains("trillx"));
}

#[test]
fn test_decoration_on_rest() {
    let tune = parse("X:1\nK:C\n!trillx!z");
    let diagnostics = analyze(&tune);

    let errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.code == DiagnosticCode::UnknownDecoration)
        .collect();

    assert_eq!(errors.len(), 1);
}

#[test]
fn test_multiple_unknown_decorations() {
    let tune = parse("X:1\nK:C\n!trillx!C !fermatta!D !akcent!E");
    let diagnostics = analyze(&tune);

    let errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.code == DiagnosticCode::UnknownDecoration)
        .collect();

    assert_eq!(errors.len(), 3);
}
