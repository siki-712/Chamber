use std::env;
use std::fs;
use std::process::ExitCode;

use chamber_diagnostics::{Diagnostic, LineIndex, Severity};
use chamber_parser::parse_with_diagnostics;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage(&args[0]);
        return ExitCode::from(1);
    }

    match args[1].as_str() {
        "check" => {
            if args.len() < 3 {
                eprintln!("Error: missing file path");
                eprintln!("Usage: {} check <file.abc>", args[0]);
                return ExitCode::from(1);
            }
            cmd_check(&args[2])
        }
        "help" | "--help" | "-h" => {
            print_usage(&args[0]);
            ExitCode::SUCCESS
        }
        "version" | "--version" | "-V" => {
            println!("chamber {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        cmd => {
            eprintln!("Unknown command: {}", cmd);
            print_usage(&args[0]);
            ExitCode::from(1)
        }
    }
}

fn print_usage(program: &str) {
    eprintln!(
        r#"Chamber - ABC notation toolkit

Usage: {} <command> [options]

Commands:
  check <file>    Check an ABC file for errors
  help            Show this help message
  version         Show version information

Examples:
  {} check tune.abc
"#,
        program, program
    );
}

fn cmd_check(path: &str) -> ExitCode {
    // Read file
    let source = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", path, e);
            return ExitCode::from(1);
        }
    };

    // Parse with diagnostics
    let result = parse_with_diagnostics(&source);
    let line_index = LineIndex::new(&source);

    // Print diagnostics
    let mut error_count = 0;
    let mut warning_count = 0;

    for diag in &result.diagnostics {
        print_diagnostic(path, &source, &line_index, diag);

        match diag.severity {
            Severity::Error => error_count += 1,
            Severity::Warning => warning_count += 1,
            Severity::Info => {}
        }
    }

    // Print summary
    if error_count > 0 || warning_count > 0 {
        eprintln!();
        eprint!("{}: ", path);

        let mut parts = Vec::new();
        if error_count > 0 {
            parts.push(format!(
                "{} error{}",
                error_count,
                if error_count == 1 { "" } else { "s" }
            ));
        }
        if warning_count > 0 {
            parts.push(format!(
                "{} warning{}",
                warning_count,
                if warning_count == 1 { "" } else { "s" }
            ));
        }
        eprintln!("{}", parts.join(", "));
    } else {
        eprintln!("{}: OK", path);
    }

    // Print tune info
    let tune = &result.tune;
    if !tune.header.fields.is_empty() {
        eprintln!();
        eprintln!("Tune info:");
        for field in &tune.header.fields {
            eprintln!("  {:?}: {}", field.kind, field.value);
        }

        let note_count = tune
            .body
            .elements
            .iter()
            .filter(|e| matches!(e, chamber_parser::MusicElement::Note(_)))
            .count();
        let bar_count = tune
            .body
            .elements
            .iter()
            .filter(|e| matches!(e, chamber_parser::MusicElement::BarLine(_)))
            .count();

        eprintln!();
        eprintln!("Body: {} notes, {} bar lines", note_count, bar_count);
    }

    if error_count > 0 {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn print_diagnostic(path: &str, source: &str, line_index: &LineIndex, diag: &Diagnostic) {
    let start_pos = line_index.line_col(diag.range.start());
    let end_pos = line_index.line_col(diag.range.end());

    // Severity color/prefix
    let (severity_str, color_code) = match diag.severity {
        Severity::Error => ("error", "\x1b[31m"),   // Red
        Severity::Warning => ("warning", "\x1b[33m"), // Yellow
        Severity::Info => ("info", "\x1b[34m"),     // Blue
    };
    let reset = "\x1b[0m";
    let bold = "\x1b[1m";

    // Print main diagnostic line
    eprintln!(
        "{}{}{}[{}]{}: {}",
        bold, color_code, severity_str, diag.code, reset, diag.message
    );

    // Print location
    eprintln!(
        "  {}-->{} {}:{}:{}",
        "\x1b[36m", // Cyan
        reset,
        path,
        start_pos.line_display(),
        start_pos.col_display()
    );

    // Print source snippet
    if let Some(line_text) = line_index.line_text(start_pos.line, source) {
        let line_num = format!("{}", start_pos.line_display());
        let padding = " ".repeat(line_num.len());

        eprintln!("  {} {}|{}", padding, "\x1b[36m", reset);
        eprintln!(
            "  {}{} |{} {}",
            "\x1b[36m", line_num, reset, line_text
        );

        // Print underline
        let underline_start = start_pos.col as usize;
        let underline_len = if start_pos.line == end_pos.line {
            (end_pos.col as usize).saturating_sub(underline_start).max(1)
        } else {
            line_text.len().saturating_sub(underline_start).max(1)
        };

        let spaces = " ".repeat(underline_start);
        let carets = "^".repeat(underline_len);

        eprintln!(
            "  {} {}|{} {}{}{}{}",
            padding, "\x1b[36m", reset, spaces, color_code, carets, reset
        );
    }

    // Print labels
    for label in &diag.labels {
        let label_pos = line_index.line_col(label.range.start());
        eprintln!(
            "  {} = note: {} (at {}:{})",
            padding_for(3),
            label.message,
            label_pos.line_display(),
            label_pos.col_display()
        );
    }

    // Print notes
    for note in &diag.notes {
        eprintln!("  {} = note: {}", padding_for(3), note);
    }

    eprintln!();
}

fn padding_for(n: usize) -> String {
    " ".repeat(n)
}
