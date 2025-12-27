//! Diagnostic types for Chamber's ABC notation parser.
//!
//! This crate provides error and warning reporting infrastructure.
//!
//! # Diagnostic Codes
//!
//! All diagnostics have a code in the format `XNNN`:
//! - `L`: Lexer errors
//! - `H`: Header validation errors
//! - `M`: Music body errors
//! - `S`: Structural errors
//! - `W`: Warnings
//!
//! See [DIAGNOSTICS.md](https://github.com/user/chamber/blob/main/crates/chamber_diagnostics/DIAGNOSTICS.md)
//! for a complete list of diagnostic codes.
//!
//! # Example
//!
//! ```
//! use chamber_diagnostics::{Diagnostic, DiagnosticCode, Severity};
//! use chamber_text_size::{TextRange, TextSize};
//!
//! let range = TextRange::new(TextSize::new(0), TextSize::new(5));
//! let diag = Diagnostic::error(
//!     DiagnosticCode::UnclosedChord,
//!     range,
//!     "unclosed chord, missing ']'",
//! );
//!
//! assert_eq!(diag.code, DiagnosticCode::UnclosedChord);
//! assert_eq!(diag.severity, Severity::Error);
//! ```

mod code;
mod diagnostic;
mod line_index;
mod severity;
mod sink;

pub use code::DiagnosticCode;
pub use diagnostic::{Diagnostic, Label};
pub use line_index::{LineCol, LineIndex};
pub use severity::Severity;
pub use sink::{DiagnosticBag, DiagnosticSink};
