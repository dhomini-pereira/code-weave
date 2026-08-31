mod chunker;
mod lexer;
mod syntax;
use code_weave_core::{CodeChunk, Formatter, Language, ParseError, Source};

use crate::{chunker::YamlChunker, lexer::YamlLexer, syntax::YamlSyntaxParser};

pub struct YamlFormatter;

impl Formatter for YamlFormatter {
    fn language(&self) -> Language {
        Language::Yaml
    }

    fn parse(&self, source: &Source) -> Result<Vec<CodeChunk>, ParseError> {
        let lexer = YamlLexer;

        let tokens = lexer.tokenize(source.content());

        let parser = YamlSyntaxParser::new(&tokens, source.content());

        let tree = parser.parse();

        let chunker = YamlChunker;

        Ok(chunker.chunk(&tree, source))
    }
}
