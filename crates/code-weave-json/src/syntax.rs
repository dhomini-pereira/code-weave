use crate::token::{Token, TokenKind};
use code_weave_core::{ParseError, Span, SyntaxKind, SyntaxNode};

pub struct JsonSyntaxParser<'a> {
    tokens: &'a [Token],
    source: &'a str,
    current: usize,
}

impl<'a> JsonSyntaxParser<'a> {
    pub fn new(tokens: &'a [Token], source: &'a str) -> Self {
        Self {
            tokens,
            source,
            current: 0,
        }
    }

    pub fn parse(mut self) -> Result<SyntaxNode, ParseError> {
        let root = self.parse_value(None)?;

        if let Some(token) = self.current_token() {
            return Err(ParseError::unexpected_token(
                format!("unexpected token after root value: {:?}", token.kind),
                token.span,
            ));
        }

        Ok(root)
    }

    fn parse_value(&mut self, name: Option<String>) -> Result<SyntaxNode, ParseError> {
        let token = self.current_token().cloned().ok_or_else(|| {
            ParseError::unexpected_end_of_input("expected value, found end of input")
        })?;

        match token.kind {
            TokenKind::LeftBrace => self.parse_object(name),

            TokenKind::LeftBracket => self.parse_array(name),

            TokenKind::String => {
                self.advance();

                Ok(SyntaxNode {
                    kind: SyntaxKind::String,
                    span: token.span,
                    name,
                    children: vec![],
                })
            }

            TokenKind::Number => {
                self.advance();

                Ok(SyntaxNode {
                    kind: SyntaxKind::Number,
                    span: token.span,
                    name,
                    children: vec![],
                })
            }

            TokenKind::True | TokenKind::False => {
                self.advance();

                Ok(SyntaxNode {
                    kind: SyntaxKind::Boolean,
                    span: token.span,
                    name,
                    children: vec![],
                })
            }

            TokenKind::Null => {
                self.advance();

                Ok(SyntaxNode {
                    kind: SyntaxKind::Null,
                    span: token.span,
                    name,
                    children: vec![],
                })
            }

            _ => Err(ParseError::unexpected_token(
                format!("expected value, found {:?}", token.kind),
                token.span,
            )),
        }
    }

    fn parse_object(&mut self, name: Option<String>) -> Result<SyntaxNode, ParseError> {
        let start = self.expect(TokenKind::LeftBrace)?.span.start;

        let mut children = Vec::new();

        if self.check(TokenKind::RightBrace) {
            let end = self.advance().span.end;

            return Ok(SyntaxNode {
                kind: SyntaxKind::Object,
                span: Span::new(start, end),
                name,
                children,
            });
        }

        loop {
            let key_token = self.expect(TokenKind::String)?;

            let key = self
                .string_value(key_token.span)
                .ok_or_else(|| ParseError::invalid_syntax("invalid object key"))?;

            self.expect(TokenKind::Colon)?;

            let value = self.parse_value(Some(key))?;

            children.push(value);

            if self.check(TokenKind::RightBrace) {
                break;
            }

            self.expect(TokenKind::Comma)?;
        }

        let end = self.advance().span.end;

        Ok(SyntaxNode {
            kind: SyntaxKind::Object,
            span: Span::new(start, end),
            name,
            children,
        })
    }

    fn parse_array(&mut self, name: Option<String>) -> Result<SyntaxNode, ParseError> {
        let start = self.expect(TokenKind::LeftBracket)?.span.start;

        let mut children = Vec::new();

        if self.check(TokenKind::RightBracket) {
            let end = self.advance().span.end;

            return Ok(SyntaxNode {
                kind: SyntaxKind::Array,
                span: Span::new(start, end),
                name,
                children,
            });
        }

        loop {
            let value = self.parse_value(None)?;

            children.push(value);

            if self.check(TokenKind::RightBracket) {
                break;
            }

            self.expect(TokenKind::Comma)?;
        }

        let end = self.advance().span.end;

        Ok(SyntaxNode {
            kind: SyntaxKind::Array,
            span: Span::new(start, end),
            name,
            children,
        })
    }

    fn current_token(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    fn advance(&mut self) -> Token {
        let token = self.tokens[self.current].clone();

        self.current += 1;

        token
    }

    fn check(&self, kind: TokenKind) -> bool {
        self.current_token().is_some_and(|token| token.kind == kind)
    }

    fn expect(&mut self, kind: TokenKind) -> Result<Token, ParseError> {
        let token = self.current_token().cloned().ok_or_else(|| {
            ParseError::unexpected_end_of_input(format!("expected {:?}, found end of input", kind))
        })?;

        if token.kind != kind {
            return Err(ParseError::unexpected_token(
                format!("expected {:?}, found {:?}", kind, token.kind),
                token.span,
            ));
        }

        Ok(self.advance())
    }

    fn string_value(&self, span: Span) -> Option<String> {
        let value = &self.source[span.start..span.end];

        serde_json::from_str(value).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::JsonLexer;
    use code_weave_core::SyntaxKind;

    #[test]
    fn should_parse_nested_object() {
        let input = r#"{
  "database": {
    "host": "localhost",
    "port": 5432
  }
}"#;

        let lexer = JsonLexer;
        let tokens = lexer.tokenize(input);

        let parser = JsonSyntaxParser::new(&tokens, input);

        let root = parser.parse().unwrap();

        assert_eq!(root.kind, SyntaxKind::Object);
        assert_eq!(root.name, None);

        assert_eq!(root.children.len(), 1);

        let database = &root.children[0];

        assert_eq!(database.kind, SyntaxKind::Object);

        assert_eq!(database.name.as_deref(), Some("database"));

        assert_eq!(database.children.len(), 2);
    }

    #[test]
    fn should_preserve_object_span() {
        let input = r#"{
  "database": {
    "host": "localhost"
  }
}"#;

        let lexer = JsonLexer;
        let tokens = lexer.tokenize(input);

        let parser = JsonSyntaxParser::new(&tokens, input);

        let root = parser.parse().unwrap();

        let database = &root.children[0];

        let content = &input[database.span.start..database.span.end];

        assert_eq!(
            content,
            r#"{
    "host": "localhost"
  }"#
        );
    }
}
