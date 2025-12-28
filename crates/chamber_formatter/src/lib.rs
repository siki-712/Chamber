//! ABC notation formatter.
//!
//! This crate provides formatting capabilities for ABC notation files,
//! using the CST for lossless transformation.
//!
//! # Example
//!
//! ```
//! use chamber_formatter::{format, FormatterConfig};
//!
//! let source = "X:1\nT:My Tune\nK:C\nCDEF|G2A2|";
//! let config = FormatterConfig::default();
//! let formatted = format(source, &config);
//! ```

mod config;
mod formatter;

pub use config::FormatterConfig;
pub use formatter::format;
