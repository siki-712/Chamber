//! ABC notation formatter.
//!
//! This crate provides formatting capabilities for ABC notation files,
//! using the CST for lossless transformation.
//!
//! # Quick Start
//!
//! ```
//! use chamber_formatter::{format, FormatterConfig};
//!
//! // Format with default settings (cleanup whitespace, ensure newline)
//! let source = "X:1  \nK:C\nCDEF|";
//! let formatted = format(source, &FormatterConfig::default());
//! assert!(formatted.ends_with('\n'));
//! ```
//!
//! # Configuration Presets
//!
//! ## Default - Standard cleanup
//! ```
//! use chamber_formatter::{format, FormatterConfig};
//!
//! let config = FormatterConfig::default();
//! // - Removes trailing whitespace
//! // - Ensures final newline
//! // - Normalizes note spacing
//! // - Adds space around bar lines
//! ```
//!
//! ## Passthrough - Preserve everything
//! ```
//! use chamber_formatter::{format, FormatterConfig};
//!
//! let source = "X:1\nK:C\n C  D  E |\n";
//! let formatted = format(source, &FormatterConfig::passthrough());
//! assert_eq!(formatted, source); // Exact preservation
//! ```
//!
//! ## Minimal - Just cleanup
//! ```
//! use chamber_formatter::{format, FormatterConfig};
//!
//! let config = FormatterConfig::minimal();
//! // - Removes trailing whitespace
//! // - Ensures final newline
//! // - Preserves original spacing otherwise
//! ```
//!
//! # Custom Configuration
//!
//! ```
//! use chamber_formatter::{format, FormatterConfig};
//!
//! let config = FormatterConfig {
//!     normalize_note_spacing: false,
//!     space_around_bars: false,
//!     trim_trailing_whitespace: true,
//!     ensure_final_newline: true,
//!     normalize_header_order: true,  // Reorder headers: X, T, C, M, L, Q, K
//!     ..FormatterConfig::default()
//! };
//!
//! let source = "K:C\nT:My Tune\nX:1\nCDEF|";
//! let formatted = format(source, &config);
//! // Headers reordered: X:1, T:My Tune, K:C
//! ```
//!
//! # Real-World Examples
//!
//! ## Clean up a messy file
//! ```
//! use chamber_formatter::{format, FormatterConfig};
//!
//! let messy = "X:1  \nT:My Tune  \nK:C  \nCDEF|G2A2|  ";
//! let clean = format(messy, &FormatterConfig::default());
//! // No trailing whitespace, has final newline
//! ```
//!
//! ## Preserve formatting for version control
//! ```
//! use chamber_formatter::{format, FormatterConfig};
//!
//! let source = "X:1\nK:C\nC D E F | G2 A2 |\n";
//! let formatted = format(source, &FormatterConfig::passthrough());
//! assert_eq!(source, formatted); // No diff
//! ```
//!
//! ## Idempotent formatting
//! ```
//! use chamber_formatter::{format, FormatterConfig};
//!
//! let source = "X:1\nK:C\nCDEF|";
//! let config = FormatterConfig::default();
//!
//! let first = format(source, &config);
//! let second = format(&first, &config);
//! assert_eq!(first, second); // Formatting twice = same result
//! ```

mod config;
mod formatter;

pub use config::FormatterConfig;
pub use formatter::format;
