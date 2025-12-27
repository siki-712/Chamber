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
