//! CST node types.

use chamber_syntax::SyntaxKind;
use chamber_text_size::TextRange;

use crate::CstToken;

/// A child element of a CST node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CstChild {
    /// A child node.
    Node(CstNode),
    /// A child token.
    Token(CstToken),
}

impl CstChild {
    /// Returns this child as a node, if it is one.
    pub fn as_node(&self) -> Option<&CstNode> {
        match self {
            CstChild::Node(node) => Some(node),
            CstChild::Token(_) => None,
        }
    }

    /// Returns this child as a token, if it is one.
    pub fn as_token(&self) -> Option<&CstToken> {
        match self {
            CstChild::Node(_) => None,
            CstChild::Token(token) => Some(token),
        }
    }

    /// Returns the range of this child (including trivia for tokens).
    pub fn range(&self) -> TextRange {
        match self {
            CstChild::Node(node) => node.range(),
            CstChild::Token(token) => token.range(),
        }
    }

    /// Returns the full range of this child (including trivia).
    pub fn full_range(&self) -> TextRange {
        match self {
            CstChild::Node(node) => node.full_range(),
            CstChild::Token(token) => token.full_range(),
        }
    }

    /// Returns the kind of this child.
    pub fn kind(&self) -> SyntaxKind {
        match self {
            CstChild::Node(node) => node.kind(),
            CstChild::Token(token) => token.kind(),
        }
    }
}

/// A composite node in the CST.
///
/// Nodes contain children, which can be other nodes or tokens.
/// The range of a node spans from its first child to its last child.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CstNode {
    /// The kind of this node.
    kind: SyntaxKind,
    /// Child elements (nodes and tokens).
    children: Vec<CstChild>,
}

impl CstNode {
    /// Creates a new empty node.
    pub fn new(kind: SyntaxKind) -> Self {
        debug_assert!(kind.is_node(), "CstNode must have a node kind");
        Self {
            kind,
            children: Vec::new(),
        }
    }

    /// Creates a new node with children.
    pub fn with_children(kind: SyntaxKind, children: Vec<CstChild>) -> Self {
        debug_assert!(kind.is_node(), "CstNode must have a node kind");
        Self { kind, children }
    }

    /// Returns the kind of this node.
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        self.kind
    }

    /// Returns the source range of this node (from first to last child, excluding trivia).
    pub fn range(&self) -> TextRange {
        if self.children.is_empty() {
            return TextRange::default();
        }

        let start = self.children.first().unwrap().range().start();
        let end = self.children.last().unwrap().range().end();
        TextRange::new(start, end)
    }

    /// Returns the full range including all trivia.
    pub fn full_range(&self) -> TextRange {
        if self.children.is_empty() {
            return TextRange::default();
        }

        let start = self.children.first().unwrap().full_range().start();
        let end = self.children.last().unwrap().full_range().end();
        TextRange::new(start, end)
    }

    /// Returns the children of this node.
    #[inline]
    pub fn children(&self) -> &[CstChild] {
        &self.children
    }

    /// Returns an iterator over child nodes.
    pub fn child_nodes(&self) -> impl Iterator<Item = &CstNode> {
        self.children.iter().filter_map(|c| c.as_node())
    }

    /// Returns an iterator over child tokens.
    pub fn child_tokens(&self) -> impl Iterator<Item = &CstToken> {
        self.children.iter().filter_map(|c| c.as_token())
    }

    /// Finds the first child node with the given kind.
    pub fn find_child_node(&self, kind: SyntaxKind) -> Option<&CstNode> {
        self.child_nodes().find(|n| n.kind() == kind)
    }

    /// Finds the first child token with the given kind.
    pub fn find_child_token(&self, kind: SyntaxKind) -> Option<&CstToken> {
        self.child_tokens().find(|t| t.kind() == kind)
    }

    /// Returns the first token in this subtree.
    pub fn first_token(&self) -> Option<&CstToken> {
        for child in &self.children {
            match child {
                CstChild::Token(token) => return Some(token),
                CstChild::Node(node) => {
                    if let Some(token) = node.first_token() {
                        return Some(token);
                    }
                }
            }
        }
        None
    }

    /// Returns the last token in this subtree.
    pub fn last_token(&self) -> Option<&CstToken> {
        for child in self.children.iter().rev() {
            match child {
                CstChild::Token(token) => return Some(token),
                CstChild::Node(node) => {
                    if let Some(token) = node.last_token() {
                        return Some(token);
                    }
                }
            }
        }
        None
    }

    /// Adds a child to this node.
    pub fn add_child(&mut self, child: CstChild) {
        self.children.push(child);
    }

    /// Adds a token child to this node.
    pub fn add_token(&mut self, token: CstToken) {
        self.children.push(CstChild::Token(token));
    }

    /// Adds a node child to this node.
    pub fn add_node(&mut self, node: CstNode) {
        self.children.push(CstChild::Node(node));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chamber_text_size::TextSize;

    fn make_token(kind: SyntaxKind, start: u32, end: u32) -> CstToken {
        CstToken::new(kind, TextRange::new(TextSize::from(start), TextSize::from(end)))
    }

    #[test]
    fn test_node_creation() {
        let node = CstNode::new(SyntaxKind::NOTE);
        assert_eq!(node.kind(), SyntaxKind::NOTE);
        assert!(node.children().is_empty());
    }

    #[test]
    fn test_node_with_children() {
        let token = make_token(SyntaxKind::NOTE_NAME, 0, 1);
        let node = CstNode::with_children(
            SyntaxKind::NOTE,
            vec![CstChild::Token(token)],
        );

        assert_eq!(node.children().len(), 1);
        assert_eq!(
            node.range(),
            TextRange::new(TextSize::from(0), TextSize::from(1))
        );
    }

    #[test]
    fn test_node_child_iteration() {
        let token1 = make_token(SyntaxKind::SHARP, 0, 1);
        let token2 = make_token(SyntaxKind::NOTE_NAME, 1, 2);
        let inner_node = CstNode::with_children(
            SyntaxKind::ACCIDENTAL,
            vec![CstChild::Token(token1)],
        );

        let outer_node = CstNode::with_children(
            SyntaxKind::NOTE,
            vec![
                CstChild::Node(inner_node),
                CstChild::Token(token2),
            ],
        );

        assert_eq!(outer_node.child_nodes().count(), 1);
        assert_eq!(outer_node.child_tokens().count(), 1);
    }

    #[test]
    fn test_first_last_token() {
        let token1 = make_token(SyntaxKind::SHARP, 0, 1);
        let token2 = make_token(SyntaxKind::NOTE_NAME, 1, 2);
        let token3 = make_token(SyntaxKind::NUMBER, 2, 3);

        let inner = CstNode::with_children(
            SyntaxKind::ACCIDENTAL,
            vec![CstChild::Token(token1)],
        );

        let node = CstNode::with_children(
            SyntaxKind::NOTE,
            vec![
                CstChild::Node(inner),
                CstChild::Token(token2),
                CstChild::Token(token3),
            ],
        );

        let first = node.first_token().unwrap();
        assert_eq!(first.kind(), SyntaxKind::SHARP);

        let last = node.last_token().unwrap();
        assert_eq!(last.kind(), SyntaxKind::NUMBER);
    }
}
