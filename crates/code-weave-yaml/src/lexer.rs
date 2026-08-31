use code_weave_core::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YamlToken {
    pub kind: YamlTokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum YamlTokenKind {
    Indent,
    Dedent,
    Newline,
    Key,
    Scalar,
    Dash,
    Colon,
    Comment,
    Unknown,
}

pub struct YamlLexer;

impl YamlLexer {
    pub fn tokenize(&self, input: &str) -> Vec<YamlToken> {
        let mut tokens = Vec::new();
        let mut indent_stack = vec![0usize];
        let mut offset = 0usize;

        for line in input.split_inclusive('\n') {
            self.tokenize_line(line, offset, &mut indent_stack, &mut tokens);

            offset += line.len();
        }

        while indent_stack.len() > 1 {
            indent_stack.pop();

            tokens.push(YamlToken {
                kind: YamlTokenKind::Dedent,
                span: Span::new(input.len(), input.len()),
            });
        }

        tokens
    }

    fn tokenize_line(
        &self,
        line: &str,
        offset: usize,
        indent_stack: &mut Vec<usize>,
        tokens: &mut Vec<YamlToken>,
    ) {
        let content = line.strip_suffix('\n').unwrap_or(line);

        if content.trim().is_empty() {
            if line.ends_with('\n') {
                let newline_start = offset + line.len() - 1;

                tokens.push(YamlToken {
                    kind: YamlTokenKind::Newline,
                    span: Span::new(newline_start, newline_start + 1),
                });
            }

            return;
        }

        let indentation = content
            .chars()
            .take_while(|character| *character == ' ')
            .count();

        let content = &content[indentation..];

        let current_indent = *indent_stack.last().unwrap();

        if indentation > current_indent {
            indent_stack.push(indentation);

            tokens.push(YamlToken {
                kind: YamlTokenKind::Indent,
                span: Span::new(offset + current_indent, offset + indentation),
            });
        } else if indentation < current_indent {
            while indentation < *indent_stack.last().unwrap() {
                indent_stack.pop();

                tokens.push(YamlToken {
                    kind: YamlTokenKind::Dedent,
                    span: Span::new(offset + indentation, offset + indentation),
                });
            }

            if indentation != *indent_stack.last().unwrap() {
                tokens.push(YamlToken {
                    kind: YamlTokenKind::Unknown,
                    span: Span::new(offset, offset + indentation),
                });

                return;
            }
        }

        if content.starts_with('#') {
            let start = offset + indentation;
            let end = start + content.len();

            tokens.push(YamlToken {
                kind: YamlTokenKind::Comment,
                span: Span::new(start, end),
            });

            self.push_newline(line, offset, tokens);

            return;
        }

        if content.starts_with("- ") || content == "-" {
            let dash_start = offset + indentation;

            tokens.push(YamlToken {
                kind: YamlTokenKind::Dash,
                span: Span::new(dash_start, dash_start + 1),
            });

            let value = content.strip_prefix('-').unwrap().trim_start();

            if !value.is_empty() {
                let value_start = offset + indentation + content.find(value).unwrap();

                if let Some(colon) = value.find(':') {
                    let key = &value[..colon].trim_end();

                    let key_start = value_start;
                    let key_end = key_start + key.len();

                    tokens.push(YamlToken {
                        kind: YamlTokenKind::Key,
                        span: Span::new(key_start, key_end),
                    });

                    let colon_start = key_end;

                    tokens.push(YamlToken {
                        kind: YamlTokenKind::Colon,
                        span: Span::new(colon_start, colon_start + 1),
                    });

                    let scalar = value[colon + 1..].trim_start();

                    if !scalar.is_empty() {
                        let scalar_start =
                            colon_start + 1 + value[colon + 1..].len() - scalar.len();

                        tokens.push(YamlToken {
                            kind: YamlTokenKind::Scalar,
                            span: Span::new(scalar_start, scalar_start + scalar.len()),
                        });
                    }
                } else {
                    tokens.push(YamlToken {
                        kind: YamlTokenKind::Scalar,
                        span: Span::new(value_start, value_start + value.len()),
                    });
                }
            }
        } else if let Some(colon) = content.find(':') {
            let key = &content[..colon];

            let key_start = offset + indentation;
            let key_end = key_start + key.len();

            tokens.push(YamlToken {
                kind: YamlTokenKind::Key,
                span: Span::new(key_start, key_end),
            });

            let colon_start = key_end;

            tokens.push(YamlToken {
                kind: YamlTokenKind::Colon,
                span: Span::new(colon_start, colon_start + 1),
            });

            let value = &content[colon + 1..];
            let value = value.trim_start();

            if !value.is_empty() {
                let leading = content[colon + 1..].len() - value.len();

                let value_start = offset + indentation + colon + 1 + leading;

                tokens.push(YamlToken {
                    kind: YamlTokenKind::Scalar,
                    span: Span::new(value_start, value_start + value.len()),
                });
            }
        } else {
            let start = offset + indentation;
            let end = start + content.len();

            tokens.push(YamlToken {
                kind: YamlTokenKind::Scalar,
                span: Span::new(start, end),
            });
        }

        self.push_newline(line, offset, tokens);
    }

    fn push_newline(&self, line: &str, offset: usize, tokens: &mut Vec<YamlToken>) {
        if line.ends_with('\n') {
            let newline_start = offset + line.len() - 1;

            tokens.push(YamlToken {
                kind: YamlTokenKind::Newline,
                span: Span::new(newline_start, newline_start + 1),
            });
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_tokenize_indentation() {
        let input = "\
    database:
      host: localhost
      port: 5432
    ";

        let lexer = YamlLexer;
        let tokens = lexer.tokenize(input);

        let kinds: Vec<_> = tokens.iter().map(|token| &token.kind).collect();

        assert!(kinds.contains(&&YamlTokenKind::Indent));
        assert!(kinds.contains(&&YamlTokenKind::Dedent));
    }

    #[test]
    fn should_preserve_absolute_spans() {
        let input = "\
    database:
      host: localhost
      port: 5432
    ";

        let lexer = YamlLexer;
        let tokens = lexer.tokenize(input);

        let host = tokens
            .iter()
            .find(|token| token.kind == YamlTokenKind::Key)
            .unwrap();

        assert_eq!(&input[host.span.start..host.span.end], "database");

        let keys: Vec<_> = tokens
            .iter()
            .filter(|token| token.kind == YamlTokenKind::Key)
            .collect();

        assert_eq!(&input[keys[1].span.start..keys[1].span.end], "host");
        assert_eq!(&input[keys[2].span.start..keys[2].span.end], "port");
    }

    #[test]
    fn should_tokenize_nested_mapping() {
        let input = "\
    database:
      host: localhost
      connection:
        port: 5432
    ";

        let lexer = YamlLexer;
        let tokens = lexer.tokenize(input);

        let indent_count = tokens
            .iter()
            .filter(|token| token.kind == YamlTokenKind::Indent)
            .count();

        let dedent_count = tokens
            .iter()
            .filter(|token| token.kind == YamlTokenKind::Dedent)
            .count();

        assert_eq!(indent_count, 2);
        assert_eq!(dedent_count, 2);
    }

    #[test]
    fn should_tokenize_sequence_mapping() {
        let input = "\
    users:
      - name: Dhomini
      - name: Maria
    ";

        let lexer = YamlLexer;
        let tokens = lexer.tokenize(input);

        let kinds: Vec<_> = tokens.iter().map(|token| &token.kind).collect();

        assert_eq!(
            kinds
                .iter()
                .filter(|kind| ***kind == YamlTokenKind::Dash)
                .count(),
            2
        );

        assert_eq!(
            kinds
                .iter()
                .filter(|kind| ***kind == YamlTokenKind::Key)
                .count(),
            3
        );
    }

    #[test]
    fn should_debug_sequence_of_objects() {
        let input = "\
    users:
      - name: Dhomini
        age: 25
      - name: Maria
        age: 30
    ";

        let lexer = YamlLexer;
        let tokens = lexer.tokenize(input);

        for token in tokens {
            println!(
                "{:?} => {:?}",
                token.kind,
                &input[token.span.start..token.span.end]
            );
        }
    }
}
