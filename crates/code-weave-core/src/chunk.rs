use crate::{ChunkPath, Language};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeChunk {
    pub kind: ChunkKind,
    pub name: Option<String>,
    pub parent: Option<ChunkPath>,
    pub path: Option<ChunkPath>,
    pub language: Language,
    pub start_line: usize,
    pub end_line: usize,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChunkKind {
    File,
    Module,
    Namespace,
    Class,
    Interface,
    Struct,
    Enum,
    Function,
    Method,
    Object,
    Array,
    Property,
    Field,
    Variable,
    Constant,
    Section,
    Block,
}
