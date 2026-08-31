use crate::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxNode {
    pub kind: SyntaxKind,
    pub span: Span,
    pub name: Option<String>,
    pub children: Vec<SyntaxNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyntaxKind {
    Root,
    Object,
    Array,
    Property,
    String,
    Number,
    Boolean,
    Null,
}
