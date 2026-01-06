//! Conversion from CST to AST.
//!
//! This module provides functions to convert a lossless CST
//! into a semantic AST for analysis.

use chamber_ast::{
    Accidental, Annotation, BarLine, BarLineKind, Body, BrokenRhythm, Chord, Decoration, Duration,
    GraceNotes, Header, HeaderField, HeaderFieldKind, InlineField, MusicElement, Note, Pitch, Rest,
    Slur, Tie, Tune, Tuplet,
};
use chamber_cst::{CstChild, CstNode, CstToken};
use chamber_syntax::SyntaxKind;
use chamber_text_size::TextRange;

/// Converts a CST tune to an AST tune.
pub fn cst_to_ast(cst: &CstNode, source: &str) -> Tune {
    debug_assert_eq!(cst.kind(), SyntaxKind::TUNE);

    let header = cst
        .find_child_node(SyntaxKind::HEADER)
        .map(|h| convert_header(h, source))
        .unwrap_or_else(|| Header {
            fields: vec![],
            range: TextRange::default(),
        });

    let body = cst
        .find_child_node(SyntaxKind::BODY)
        .map(|b| convert_body(b, source))
        .unwrap_or_else(|| Body {
            elements: vec![],
            range: TextRange::default(),
        });

    Tune {
        header,
        body,
        range: cst.range(),
    }
}

fn convert_header(cst: &CstNode, source: &str) -> Header {
    let fields: Vec<HeaderField> = cst
        .child_nodes()
        .filter(|n| n.kind() == SyntaxKind::HEADER_FIELD)
        .map(|n| convert_header_field(n, source))
        .collect();

    Header {
        fields,
        range: cst.range(),
    }
}

fn convert_header_field(cst: &CstNode, source: &str) -> HeaderField {
    let label_token = cst.find_child_token(SyntaxKind::FIELD_LABEL);
    let text_token = cst.find_child_token(SyntaxKind::TEXT);

    let label_char = label_token
        .map(|t| t.text(source).chars().next().unwrap_or('?'))
        .unwrap_or('?');

    let value = text_token.map(|t| t.text(source).trim().to_string()).unwrap_or_default();

    HeaderField {
        kind: HeaderFieldKind::from_char(label_char),
        value,
        range: cst.range(),
    }
}

fn convert_body(cst: &CstNode, source: &str) -> Body {
    let elements: Vec<MusicElement> = cst
        .children()
        .iter()
        .filter_map(|child| convert_music_element(child, source))
        .collect();

    Body {
        elements,
        range: cst.range(),
    }
}

fn convert_music_element(child: &CstChild, source: &str) -> Option<MusicElement> {
    match child {
        CstChild::Node(node) => match node.kind() {
            SyntaxKind::NOTE => Some(MusicElement::Note(convert_note(node, source))),
            SyntaxKind::REST_NODE => Some(MusicElement::Rest(convert_rest(node, source))),
            SyntaxKind::CHORD => Some(MusicElement::Chord(convert_chord(node, source))),
            SyntaxKind::BAR_LINE => Some(MusicElement::BarLine(convert_bar_line(node, source))),
            SyntaxKind::TUPLET => Some(MusicElement::Tuplet(convert_tuplet(node, source))),
            SyntaxKind::SLUR => Some(MusicElement::Slur(convert_slur(node, source))),
            SyntaxKind::GRACE_NOTES => {
                Some(MusicElement::GraceNotes(convert_grace_notes(node, source)))
            }
            SyntaxKind::BROKEN_RHYTHM_NODE => {
                Some(MusicElement::BrokenRhythm(convert_broken_rhythm(node, source)))
            }
            SyntaxKind::TIE_NODE => Some(MusicElement::Tie(convert_tie(node))),
            SyntaxKind::INLINE_FIELD => {
                Some(MusicElement::InlineField(convert_inline_field(node, source)))
            }
            SyntaxKind::ANNOTATION_NODE => {
                Some(MusicElement::Annotation(convert_annotation(node, source)))
            }
            _ => None,
        },
        CstChild::Token(_) => None, // Skip raw tokens in body
    }
}

fn convert_note(cst: &CstNode, source: &str) -> Note {
    // Extract decorations
    let decorations: Vec<Decoration> = cst
        .child_nodes()
        .filter(|n| n.kind() == SyntaxKind::DECORATION_NODE)
        .filter_map(|n| n.first_token())
        .map(|t| {
            let text = t.text(source);
            // Remove the delimiters (! or +)
            let name = text
                .trim_start_matches(|c| c == '!' || c == '+')
                .trim_end_matches(|c| c == '!' || c == '+')
                .to_string();
            Decoration::new(name, t.range())
        })
        .collect();

    // Extract accidental
    let accidental = cst.find_child_node(SyntaxKind::ACCIDENTAL).map(|acc| {
        let sharp_count = acc
            .child_tokens()
            .filter(|t| t.kind() == SyntaxKind::SHARP)
            .count();
        let flat_count = acc
            .child_tokens()
            .filter(|t| t.kind() == SyntaxKind::FLAT)
            .count();
        let natural_count = acc
            .child_tokens()
            .filter(|t| t.kind() == SyntaxKind::NATURAL)
            .count();

        if natural_count > 0 {
            Accidental::Natural
        } else if sharp_count >= 2 {
            Accidental::DoubleSharp
        } else if sharp_count == 1 {
            Accidental::Sharp
        } else if flat_count >= 2 {
            Accidental::DoubleFlat
        } else {
            Accidental::Flat
        }
    });

    // Extract pitch and base octave
    let note_token = cst.find_child_token(SyntaxKind::NOTE_NAME);
    let (pitch, base_octave) = note_token
        .and_then(|t| {
            let c = t.text(source).chars().next()?;
            Pitch::from_char(c)
        })
        .unwrap_or((Pitch::C, 0));

    // Count octave modifiers
    let octave_up = cst
        .child_tokens()
        .filter(|t| t.kind() == SyntaxKind::OCTAVE_UP)
        .count() as i8;
    let octave_down = cst
        .child_tokens()
        .filter(|t| t.kind() == SyntaxKind::OCTAVE_DOWN)
        .count() as i8;
    let octave = base_octave + octave_up - octave_down;

    // Extract duration
    let duration = cst
        .find_child_node(SyntaxKind::DURATION)
        .map(|d| convert_duration(d, source));

    Note {
        pitch,
        octave,
        accidental,
        duration,
        decorations,
        range: cst.range(),
    }
}

fn convert_duration(cst: &CstNode, source: &str) -> Duration {
    let tokens: Vec<&CstToken> = cst.child_tokens().collect();

    let mut numerator = 1u32;
    let mut denominator = 1u32;
    let mut has_slash = false;

    for token in &tokens {
        match token.kind() {
            SyntaxKind::NUMBER => {
                let val: u32 = token.text(source).parse().unwrap_or(1);
                if has_slash {
                    denominator = val;
                } else {
                    numerator = val;
                }
            }
            SyntaxKind::SLASH => {
                has_slash = true;
                // If no denominator follows, default is 2
                denominator = 2;
            }
            _ => {}
        }
    }

    Duration::new(numerator, denominator)
}

fn convert_rest(cst: &CstNode, source: &str) -> Rest {
    let decorations: Vec<Decoration> = cst
        .child_nodes()
        .filter(|n| n.kind() == SyntaxKind::DECORATION_NODE)
        .filter_map(|n| n.first_token())
        .map(|t| {
            let text = t.text(source);
            let name = text
                .trim_start_matches(|c| c == '!' || c == '+')
                .trim_end_matches(|c| c == '!' || c == '+')
                .to_string();
            Decoration::new(name, t.range())
        })
        .collect();

    let rest_token = cst.find_child_token(SyntaxKind::REST);
    let multi_measure = rest_token
        .map(|t| t.text(source) == "Z")
        .unwrap_or(false);

    let duration = cst
        .find_child_node(SyntaxKind::DURATION)
        .map(|d| convert_duration(d, source));

    Rest {
        multi_measure,
        duration,
        decorations,
        range: cst.range(),
    }
}

fn convert_chord(cst: &CstNode, source: &str) -> Chord {
    let decorations: Vec<Decoration> = cst
        .child_nodes()
        .filter(|n| n.kind() == SyntaxKind::DECORATION_NODE)
        .filter_map(|n| n.first_token())
        .map(|t| {
            let text = t.text(source);
            let name = text
                .trim_start_matches(|c| c == '!' || c == '+')
                .trim_end_matches(|c| c == '!' || c == '+')
                .to_string();
            Decoration::new(name, t.range())
        })
        .collect();

    let notes: Vec<Note> = cst
        .child_nodes()
        .filter(|n| n.kind() == SyntaxKind::NOTE)
        .map(|n| convert_note(n, source))
        .collect();

    let duration = cst
        .find_child_node(SyntaxKind::DURATION)
        .map(|d| convert_duration(d, source));

    Chord {
        notes,
        duration,
        decorations,
        range: cst.range(),
    }
}

fn convert_bar_line(cst: &CstNode, _source: &str) -> BarLine {
    let kind = cst
        .first_token()
        .map(|t| match t.kind() {
            SyntaxKind::BAR => BarLineKind::Single,
            SyntaxKind::DOUBLE_BAR => BarLineKind::Double,
            SyntaxKind::REPEAT_START => BarLineKind::RepeatStart,
            SyntaxKind::REPEAT_END => BarLineKind::RepeatEnd,
            SyntaxKind::THIN_THICK_BAR => BarLineKind::ThinThick,
            SyntaxKind::THICK_THIN_BAR => BarLineKind::ThickThin,
            _ => BarLineKind::Single,
        })
        .unwrap_or(BarLineKind::Single);

    BarLine {
        kind,
        range: cst.range(),
    }
}

fn convert_tuplet(cst: &CstNode, source: &str) -> Tuplet {
    let marker_token = cst.find_child_token(SyntaxKind::TUPLET_MARKER);
    let ratio = marker_token
        .map(|t| {
            let text = t.text(source);
            // Extract number from (3 -> 3
            text.trim_start_matches('(').parse().unwrap_or(3)
        })
        .unwrap_or(3);

    let notes: Vec<Note> = cst
        .child_nodes()
        .filter(|n| n.kind() == SyntaxKind::NOTE)
        .map(|n| convert_note(n, source))
        .collect();

    Tuplet {
        ratio,
        notes,
        range: cst.range(),
    }
}

fn convert_slur(cst: &CstNode, source: &str) -> Slur {
    let elements: Vec<MusicElement> = cst
        .children()
        .iter()
        .filter_map(|child| convert_music_element(child, source))
        .collect();

    Slur {
        elements,
        range: cst.range(),
    }
}

fn convert_grace_notes(cst: &CstNode, source: &str) -> GraceNotes {
    let notes: Vec<Note> = cst
        .child_nodes()
        .filter(|n| n.kind() == SyntaxKind::NOTE)
        .map(|n| convert_note(n, source))
        .collect();

    GraceNotes {
        notes,
        range: cst.range(),
    }
}

fn convert_broken_rhythm(cst: &CstNode, source: &str) -> BrokenRhythm {
    let token = cst.first_token();
    let text = token.map(|t| t.text(source)).unwrap_or("");

    let dotted_first = text.contains('>');
    let count = text.len() as u32;

    BrokenRhythm {
        dotted_first,
        count,
        range: cst.range(),
    }
}

fn convert_tie(cst: &CstNode) -> Tie {
    Tie { range: cst.range() }
}

fn convert_inline_field(cst: &CstNode, source: &str) -> InlineField {
    // Find field label token inside brackets
    let label = cst
        .child_tokens()
        .find(|t| t.kind() == SyntaxKind::FIELD_LABEL)
        .map(|t| t.text(source).chars().next().unwrap_or('?'))
        .unwrap_or('?');

    // Find text content
    let value = cst
        .child_tokens()
        .find(|t| t.kind() == SyntaxKind::TEXT)
        .map(|t| t.text(source).trim().to_string())
        .unwrap_or_default();

    InlineField {
        label,
        value,
        range: cst.range(),
    }
}

fn convert_annotation(cst: &CstNode, source: &str) -> Annotation {
    let text = cst
        .first_token()
        .map(|t| {
            let raw = t.text(source);
            // Remove surrounding double quotes
            raw.trim_start_matches('"')
                .trim_end_matches('"')
                .to_string()
        })
        .unwrap_or_default();

    Annotation {
        text,
        range: cst.range(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_cst;

    #[test]
    fn test_convert_simple_tune() {
        let source = "X:1\nK:C\nCDE";
        let cst = parse_cst(source);
        let ast = cst_to_ast(&cst, source);

        assert!(!ast.header.fields.is_empty());
        assert!(!ast.body.elements.is_empty());
    }

    #[test]
    fn test_convert_annotation() {
        let source = "X:1\nK:C\n\"CM7\"C";
        let cst = parse_cst(source);
        let ast = cst_to_ast(&cst, source);

        eprintln!("Body elements: {:?}", ast.body.elements);
        assert_eq!(ast.body.elements.len(), 2, "Expected 2 elements (annotation + note)");

        match &ast.body.elements[0] {
            MusicElement::Annotation(ann) => {
                assert_eq!(ann.text, "CM7");
            }
            other => panic!("Expected Annotation at 0, got {:?}", other),
        }

        match &ast.body.elements[1] {
            MusicElement::Note(note) => {
                assert_eq!(note.pitch, Pitch::C);
            }
            other => panic!("Expected Note at 1, got {:?}", other),
        }
    }

    #[test]
    fn test_convert_note_pitch() {
        let source = "X:1\nK:C\nC";
        let cst = parse_cst(source);
        let ast = cst_to_ast(&cst, source);

        if let Some(MusicElement::Note(note)) = ast.body.elements.first() {
            assert_eq!(note.pitch, Pitch::C);
            assert_eq!(note.octave, 0);
        } else {
            panic!("Expected note");
        }
    }

    #[test]
    fn test_convert_note_lowercase() {
        let source = "X:1\nK:C\nc";
        let cst = parse_cst(source);
        let ast = cst_to_ast(&cst, source);

        if let Some(MusicElement::Note(note)) = ast.body.elements.first() {
            assert_eq!(note.pitch, Pitch::C);
            assert_eq!(note.octave, 1); // lowercase = octave 1
        } else {
            panic!("Expected note");
        }
    }

    #[test]
    fn test_convert_note_with_accidental() {
        let source = "X:1\nK:C\n^C";
        let cst = parse_cst(source);
        let ast = cst_to_ast(&cst, source);

        if let Some(MusicElement::Note(note)) = ast.body.elements.first() {
            assert_eq!(note.accidental, Some(Accidental::Sharp));
        } else {
            panic!("Expected note");
        }
    }

    #[test]
    fn test_convert_chord() {
        let source = "X:1\nK:C\n[CEG]";
        let cst = parse_cst(source);
        let ast = cst_to_ast(&cst, source);

        if let Some(MusicElement::Chord(chord)) = ast.body.elements.first() {
            assert_eq!(chord.notes.len(), 3);
        } else {
            panic!("Expected chord");
        }
    }

    #[test]
    fn test_convert_bar_line() {
        let source = "X:1\nK:C\nC|D";
        let cst = parse_cst(source);
        let ast = cst_to_ast(&cst, source);

        let bars: Vec<_> = ast
            .body
            .elements
            .iter()
            .filter(|e| matches!(e, MusicElement::BarLine(_)))
            .collect();
        assert_eq!(bars.len(), 1);
    }
}
