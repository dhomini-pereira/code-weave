use crate::{CodeChunk, Language, ParseError, Source};

pub trait Parser {
    fn language(&self) -> Language;

    fn parse(&self, source: &Source) -> Result<Vec<CodeChunk>, ParseError>;
}
