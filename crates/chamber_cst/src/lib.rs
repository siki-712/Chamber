//! Concrete Syntax Tree for ABC notation.
//!
//! This crate provides lossless syntax tree types that preserve all
//! source information including whitespace and comments.
//!
//! # Key Types
//!
//! - [`CstNode`]: A composite syntax node containing children
//! - [`CstToken`]: A terminal token with optional trivia
//! - [`CstChild`]: Either a node or token child
//!
//! # Example
//!
//! ```ignore
//! use chamber_cst::{CstNode, CstToken, CstChild};
//! use chamber_syntax::SyntaxKind;
//!
//! // CST preserves everything for lossless round-trip
//! let cst = parse_cst("X:1\nK:C\nCDE");
//! let output = cst.print(source);
//! assert_eq!(source, output);
//! ```

mod node;
mod token;
mod print;

pub use node::{CstNode, CstChild};
pub use token::CstToken;
pub use print::print_cst;
