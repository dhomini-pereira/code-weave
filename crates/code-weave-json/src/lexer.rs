use crate::token::{Token, TokenKind};
use code_weave_core::Span;

pub struct JsonLexer;

impl JsonLexer {
    pub fn tokenize(&self, input: &str) -> Vec<Token> {
        let bytes = input.as_bytes();
        let mut tokens = Vec::new();

        let mut index = 0;

        while index < bytes.len() {
            let byte = bytes[index];

            match byte {
                b'{' => {
                    tokens.push(Token {
                        kind: TokenKind::LeftBrace,
                        span: Span::new(index, index + 1),
                    });

                    index += 1;
                }

                b'}' => {
                    tokens.push(Token {
                        kind: TokenKind::RightBrace,
                        span: Span::new(index, index + 1),
                    });

                    index += 1;
                }

                b'[' => {
                    tokens.push(Token {
                        kind: TokenKind::LeftBracket,
                        span: Span::new(index, index + 1),
                    });

                    index += 1;
                }

                b']' => {
                    tokens.push(Token {
                        kind: TokenKind::RightBracket,
                        span: Span::new(index, index + 1),
                    });

                    index += 1;
                }

                b':' => {
                    tokens.push(Token {
                        kind: TokenKind::Colon,
                        span: Span::new(index, index + 1),
                    });

                    index += 1;
                }

                b',' => {
                    tokens.push(Token {
                        kind: TokenKind::Comma,
                        span: Span::new(index, index + 1),
                    });

                    index += 1;
                }

                b'"' => {
                    let start = index;

                    index += 1;

                    while index < bytes.len() {
                        if bytes[index] == b'\\' {
                            index += 2;
                            continue;
                        }

                        if bytes[index] == b'"' {
                            index += 1;
                            break;
                        }

                        index += 1;
                    }

                    tokens.push(Token {
                        kind: TokenKind::String,
                        span: Span::new(start, index),
                    });
                }

                b'-' | b'0'..=b'9' => {
                    let start = index;

                    while index < bytes.len()
                        && !matches!(
                            bytes[index],
                            b',' | b'}' | b']' | b' ' | b'\n' | b'\r' | b'\t'
                        )
                    {
                        index += 1;
                    }

                    tokens.push(Token {
                        kind: TokenKind::Number,
                        span: Span::new(start, index),
                    });
                }

                b't' if input[index..].starts_with("true") => {
                    tokens.push(Token {
                        kind: TokenKind::True,
                        span: Span::new(index, index + 4),
                    });

                    index += 4;
                }

                b'f' if input[index..].starts_with("false") => {
                    tokens.push(Token {
                        kind: TokenKind::False,
                        span: Span::new(index, index + 5),
                    });

                    index += 5;
                }

                b'n' if input[index..].starts_with("null") => {
                    tokens.push(Token {
                        kind: TokenKind::Null,
                        span: Span::new(index, index + 4),
                    });

                    index += 4;
                }

                b' ' | b'\n' | b'\r' | b'\t' => {
                    index += 1;
                }

                _ => {
                    tokens.push(Token {
                        kind: TokenKind::Unknown,
                        span: Span::new(index, index + 1),
                    });

                    index += 1;
                }
            }
        }

        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::TokenKind;

    #[test]
    fn should_tokenize_object() {
        let input = r#"{
    "name": "Dhomini",
    "age": 25
}"#;

        let lexer = JsonLexer;

        let tokens = lexer.tokenize(input);

        assert_eq!(
            tokens.iter().map(|token| &token.kind).collect::<Vec<_>>(),
            vec![
                &TokenKind::LeftBrace,
                &TokenKind::String,
                &TokenKind::Colon,
                &TokenKind::String,
                &TokenKind::Comma,
                &TokenKind::String,
                &TokenKind::Colon,
                &TokenKind::Number,
                &TokenKind::RightBrace,
            ]
        );
    }

    #[test]
    fn should_preserve_spans() {
        let input = r#"{"name":"Dhomini"}"#;

        let lexer = JsonLexer;

        let tokens = lexer.tokenize(input);

        let name = &tokens[1];

        assert_eq!(&input[name.span.start..name.span.end], "\"name\"");
    }
}
