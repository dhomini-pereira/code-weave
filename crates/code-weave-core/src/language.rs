#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    Json,
    Yaml,
    TypeScript,
    JavaScript,
    Rust,
    Python,
    Java,
}

impl Language {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Yaml => "yaml",
            Self::TypeScript => "typescript",
            Self::JavaScript => "javascript",
            Self::Rust => "rust",
            Self::Python => "python",
            Self::Java => "java",
        }
    }
}
