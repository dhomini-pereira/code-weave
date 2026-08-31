use code_weave_core::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeScriptToken {
    pub kind: TypeScriptTokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeScriptTokenKind {
    Identifier,
    String,
    Number,
    Import,
    Export,
    Const,
    Let,
    Var,
    Function,
    Class,
    Interface,
    Type,
    Enum,
    OpenBrace,
    CloseBrace,
    OpenParen,
    CloseParen,
    OpenBracket,
    CloseBracket,
    Colon,
    Comma,
    Semicolon,
    Dot,
    Question,
    Equals,
    Arrow,
    LineComment,
    BlockComment,
    Newline,
    Unknown,
    Boolean,
}

pub struct TypeScriptLexer;

impl TypeScriptLexer {
    pub fn tokenize(&self, input: &str) -> Vec<TypeScriptToken> {
        let mut tokens = Vec::new();
        let mut current = 0;

        while current < input.len() {
            let byte = input.as_bytes()[current];

            if byte == b'/' {
                if input.as_bytes().get(current + 1) == Some(&b'/') {
                    let start = current;

                    current += 2;

                    while current < input.len() && input.as_bytes()[current] != b'\n' {
                        current += 1;
                    }

                    tokens.push(TypeScriptToken {
                        kind: TypeScriptTokenKind::LineComment,
                        span: Span::new(start, current),
                    });

                    continue;
                }

                if input.as_bytes().get(current + 1) == Some(&b'*') {
                    let start = current;

                    current += 2;

                    while current + 1 < input.len()
                        && !(input.as_bytes()[current] == b'*'
                            && input.as_bytes()[current + 1] == b'/')
                    {
                        current += 1;
                    }

                    if current + 1 < input.len() {
                        current += 2;
                    }

                    tokens.push(TypeScriptToken {
                        kind: TypeScriptTokenKind::BlockComment,
                        span: Span::new(start, current),
                    });

                    continue;
                }
            }

            if byte.is_ascii_whitespace() {
                if byte == b'\n' {
                    tokens.push(TypeScriptToken {
                        kind: TypeScriptTokenKind::Newline,
                        span: Span::new(current, current + 1),
                    });
                }

                current += 1;
                continue;
            }

            if byte == b'"' || byte == b'\'' || byte == b'`' {
                let quote = byte;
                let start = current;

                current += 1;

                while current < input.len() {
                    if input.as_bytes()[current] == quote {
                        current += 1;
                        break;
                    }

                    current += 1;
                }

                tokens.push(TypeScriptToken {
                    kind: TypeScriptTokenKind::String,
                    span: Span::new(start, current),
                });

                continue;
            }

            if byte.is_ascii_digit() {
                let start = current;

                while current < input.len() && input.as_bytes()[current].is_ascii_digit() {
                    current += 1;
                }

                tokens.push(TypeScriptToken {
                    kind: TypeScriptTokenKind::Number,
                    span: Span::new(start, current),
                });

                continue;
            }

            if is_identifier_start(byte) {
                let start = current;

                current += 1;

                while current < input.len() && is_identifier_continue(input.as_bytes()[current]) {
                    current += 1;
                }

                let text = &input[start..current];

                tokens.push(TypeScriptToken {
                    kind: self.classify_keyword(text),
                    span: Span::new(start, current),
                });

                continue;
            }

            if byte == b'=' && input.as_bytes().get(current + 1) == Some(&b'>') {
                tokens.push(TypeScriptToken {
                    kind: TypeScriptTokenKind::Arrow,
                    span: Span::new(current, current + 2),
                });

                current += 2;
                continue;
            }

            let kind = match byte {
                b'{' => TypeScriptTokenKind::OpenBrace,
                b'}' => TypeScriptTokenKind::CloseBrace,
                b'(' => TypeScriptTokenKind::OpenParen,
                b')' => TypeScriptTokenKind::CloseParen,
                b'[' => TypeScriptTokenKind::OpenBracket,
                b']' => TypeScriptTokenKind::CloseBracket,
                b':' => TypeScriptTokenKind::Colon,
                b',' => TypeScriptTokenKind::Comma,
                b';' => TypeScriptTokenKind::Semicolon,
                b'.' => TypeScriptTokenKind::Dot,
                b'?' => TypeScriptTokenKind::Question,
                b'=' => TypeScriptTokenKind::Equals,
                _ => TypeScriptTokenKind::Unknown,
            };

            tokens.push(TypeScriptToken {
                kind,
                span: Span::new(current, current + 1),
            });

            current += 1;
        }

        tokens
    }

    fn classify_keyword(&self, value: &str) -> TypeScriptTokenKind {
        match value {
            "import" => TypeScriptTokenKind::Import,
            "export" => TypeScriptTokenKind::Export,
            "const" => TypeScriptTokenKind::Const,
            "let" => TypeScriptTokenKind::Let,
            "var" => TypeScriptTokenKind::Var,
            "function" => TypeScriptTokenKind::Function,
            "class" => TypeScriptTokenKind::Class,
            "interface" => TypeScriptTokenKind::Interface,
            "type" => TypeScriptTokenKind::Type,
            "enum" => TypeScriptTokenKind::Enum,
            "true" | "false" => TypeScriptTokenKind::Boolean,
            _ => TypeScriptTokenKind::Identifier,
        }
    }
}

fn is_identifier_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_' || byte == b'$'
}

fn is_identifier_continue(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_tokenize_declarations() {
        let input = "\
const name = \"Dhomini\";
let age = 25;
var active = true;
";

        let lexer = TypeScriptLexer;
        let tokens = lexer.tokenize(input);

        let kinds: Vec<_> = tokens.iter().map(|token| &token.kind).collect();

        assert_eq!(
            kinds,
            vec![
                &TypeScriptTokenKind::Const,
                &TypeScriptTokenKind::Identifier,
                &TypeScriptTokenKind::Equals,
                &TypeScriptTokenKind::String,
                &TypeScriptTokenKind::Semicolon,
                &TypeScriptTokenKind::Newline,
                &TypeScriptTokenKind::Let,
                &TypeScriptTokenKind::Identifier,
                &TypeScriptTokenKind::Equals,
                &TypeScriptTokenKind::Number,
                &TypeScriptTokenKind::Semicolon,
                &TypeScriptTokenKind::Newline,
                &TypeScriptTokenKind::Var,
                &TypeScriptTokenKind::Identifier,
                &TypeScriptTokenKind::Equals,
                &TypeScriptTokenKind::Boolean,
                &TypeScriptTokenKind::Semicolon,
                &TypeScriptTokenKind::Newline,
            ]
        );
    }

    #[test]
    fn should_tokenize_declaration_keywords() {
        let input = "\
import foo from \"foo\";
export function test() {}
class User {}
interface Person {}
type UserId = string;
enum Status {}
";

        let lexer = TypeScriptLexer;
        let tokens = lexer.tokenize(input);

        let kinds: Vec<_> = tokens.iter().map(|token| &token.kind).collect();

        assert!(kinds.contains(&&TypeScriptTokenKind::Import));
        assert!(kinds.contains(&&TypeScriptTokenKind::Export));
        assert!(kinds.contains(&&TypeScriptTokenKind::Function));
        assert!(kinds.contains(&&TypeScriptTokenKind::Class));
        assert!(kinds.contains(&&TypeScriptTokenKind::Interface));
        assert!(kinds.contains(&&TypeScriptTokenKind::Type));
        assert!(kinds.contains(&&TypeScriptTokenKind::Enum));
    }

    #[test]
    fn should_preserve_absolute_spans() {
        let input = "\
const name = \"Dhomini\";
";

        let lexer = TypeScriptLexer;
        let tokens = lexer.tokenize(input);

        assert_eq!(&input[tokens[0].span.start..tokens[0].span.end], "const");

        assert_eq!(&input[tokens[1].span.start..tokens[1].span.end], "name");

        assert_eq!(
            &input[tokens[3].span.start..tokens[3].span.end],
            "\"Dhomini\""
        );
    }

    #[test]
    fn should_tokenize_comments_and_arrow_functions() {
        let input = "\
    // comment
    const getName = (user: User) => user.name;

    /* block comment */
    ";

        let lexer = TypeScriptLexer;
        let tokens = lexer.tokenize(input);

        let kinds: Vec<_> = tokens.iter().map(|token| &token.kind).collect();

        assert_eq!(
            kinds,
            vec![
                &TypeScriptTokenKind::LineComment,
                &TypeScriptTokenKind::Newline,
                &TypeScriptTokenKind::Const,
                &TypeScriptTokenKind::Identifier,
                &TypeScriptTokenKind::Equals,
                &TypeScriptTokenKind::OpenParen,
                &TypeScriptTokenKind::Identifier,
                &TypeScriptTokenKind::Colon,
                &TypeScriptTokenKind::Identifier,
                &TypeScriptTokenKind::CloseParen,
                &TypeScriptTokenKind::Arrow,
                &TypeScriptTokenKind::Identifier,
                &TypeScriptTokenKind::Dot,
                &TypeScriptTokenKind::Identifier,
                &TypeScriptTokenKind::Semicolon,
                &TypeScriptTokenKind::Newline,
                &TypeScriptTokenKind::Newline,
                &TypeScriptTokenKind::BlockComment,
                &TypeScriptTokenKind::Newline,
            ]
        );
    }

    #[test]
    fn should_preserve_comment_spans() {
        let input = "// hello\n";

        let lexer = TypeScriptLexer;
        let tokens = lexer.tokenize(input);

        assert_eq!(&input[tokens[0].span.start..tokens[0].span.end], "// hello");
    }
}
