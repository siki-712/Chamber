//! Syntax primitives for ABC notation.
//!
//! This crate defines the fundamental syntax types used by the CST:
//! - `SyntaxKind`: Enumeration of all token and node kinds
//! - `Trivia`: Whitespace, comments, and other non-semantic tokens

mod kind;
mod trivia;

pub use kind::SyntaxKind;
pub use trivia::Trivia;
