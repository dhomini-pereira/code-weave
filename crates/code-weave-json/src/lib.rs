mod chunker;
mod lexer;
mod syntax;
mod token;

use code_weave_core::{CodeChunk, Formatter, Language, ParseError, Source};

use chunker::JsonChunker;
use lexer::JsonLexer;
use syntax::JsonSyntaxParser;

pub struct JsonFormatter;

impl Formatter for JsonFormatter {
    fn language(&self) -> Language {
        Language::Json
    }

    fn parse(&self, source: &Source) -> Result<Vec<CodeChunk>, ParseError> {
        let lexer = JsonLexer;

        let tokens = lexer.tokenize(source.content());

        let parser = JsonSyntaxParser::new(&tokens, source.content());

        let tree = parser.parse()?;

        let chunker = JsonChunker;

        Ok(chunker.chunk(&tree, source))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use code_weave_core::{ChunkKind, ChunkPath, ParseErrorKind};

    #[test]
    fn should_implement_formatter() {
        let formatter = JsonFormatter;

        assert_eq!(formatter.language(), Language::Json);
    }

    #[test]
    fn should_parse_json_into_chunks() {
        let input = r#"{
  "database": {
    "host": "localhost",
    "port": 5432
  }
}"#;

        let source = Source::new(input);

        let formatter = JsonFormatter;

        let chunks = formatter.parse(&source).unwrap();

        assert_eq!(chunks.len(), 2);

        assert_eq!(chunks[0].kind, ChunkKind::File);

        assert_eq!(chunks[1].name.as_deref(), Some("database"));

        assert_eq!(
            chunks[1].path.as_ref().map(ChunkPath::as_str),
            Some("database")
        );
    }

    #[test]
    fn should_return_error_with_span() {
        let input = r#"{
      "name": "Dhomini"
      "age": 25
    }"#;

        let lexer = JsonLexer;
        let tokens = lexer.tokenize(input);

        let parser = JsonSyntaxParser::new(&tokens, input);

        let error = parser.parse().unwrap_err();

        assert_eq!(error.kind, ParseErrorKind::UnexpectedToken);

        assert!(error.span.is_some());

        let span = error.span.unwrap();

        assert_eq!(&input[span.start..span.end], "\"age\"");
    }
}
