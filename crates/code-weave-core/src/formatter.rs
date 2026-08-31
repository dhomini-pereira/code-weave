use crate::{CodeChunk, Language, ParseError, Source};

pub trait Formatter {
    fn language(&self) -> Language;

    fn parse(&self, source: &Source) -> Result<Vec<CodeChunk>, ParseError>;
}
