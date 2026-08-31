use code_weave_core::{Span, SyntaxKind, SyntaxNode};

use crate::lexer::{TypeScriptToken, TypeScriptTokenKind};

pub struct TypeScriptSyntaxParser<'a> {
    tokens: &'a [TypeScriptToken],
    source: &'a str,
    current: usize,
}

impl<'a> TypeScriptSyntaxParser<'a> {
    pub fn new(tokens: &'a [TypeScriptToken], source: &'a str) -> Self {
        Self {
            tokens,
            source,
            current: 0,
        }
    }

    pub fn parse(mut self) -> SyntaxNode {
        let start = 0;
        let mut children = Vec::new();

        self.skip_newlines();

        while !self.is_at_end() {
            if let Some(node) = self.parse_variable_declaration() {
                children.push(node);
            } else if let Some(node) = self.parse_function_declaration() {
                children.push(node);
            } else {
                self.advance();
            }

            self.skip_newlines();
        }

        SyntaxNode {
            kind: SyntaxKind::Root,
            span: Span::new(start, self.source.len()),
            name: None,
            children,
        }
    }

    fn parse_function_declaration(&mut self) -> Option<SyntaxNode> {
        if !self.check(TypeScriptTokenKind::Function) {
            return None;
        }

        let start = self.advance().span.start;

        let name_token = self.expect(TypeScriptTokenKind::Identifier)?;
        let name_span = name_token.span;

        let name = self.source[name_span.start..name_span.end].to_string();

        self.expect(TypeScriptTokenKind::OpenParen)?;

        let parameters = self.parse_function_parameters();

        self.expect(TypeScriptTokenKind::CloseParen)?;

        if self.check(TypeScriptTokenKind::Colon) {
            self.advance();

            if !self.is_at_end() {
                self.advance();
            }
        }

        let body = self.parse_block()?;
        let body_end = body.span.end;

        Some(SyntaxNode {
            kind: SyntaxKind::Function,
            span: Span::new(start, body_end),
            name: Some(name),
            children: {
                let mut children = parameters;
                children.push(body);
                children
            },
        })
    }

    fn parse_function_parameters(&mut self) -> Vec<SyntaxNode> {
        let mut parameters = Vec::new();

        while !self.is_at_end() && !self.check(TypeScriptTokenKind::CloseParen) {
            let Some(name_token) = self.expect(TypeScriptTokenKind::Identifier) else {
                self.advance();
                continue;
            };

            let name_span = name_token.span;

            let name = self.source[name_span.start..name_span.end].to_string();

            if self.check(TypeScriptTokenKind::Colon) {
                self.advance();

                if self.check(TypeScriptTokenKind::Identifier) {
                    self.advance();
                }
            }

            parameters.push(SyntaxNode {
                kind: SyntaxKind::Property,
                span: name_span,
                name: Some(name),
                children: Vec::new(),
            });

            if self.check(TypeScriptTokenKind::Comma) {
                self.advance();
            }
        }

        parameters
    }

    fn parse_value(&mut self) -> Option<SyntaxNode> {
        let token = self.current_token()?;

        match token.kind {
            TypeScriptTokenKind::String => {
                let token = self.advance();

                Some(SyntaxNode {
                    kind: SyntaxKind::String,
                    span: token.span,
                    name: None,
                    children: Vec::new(),
                })
            }

            TypeScriptTokenKind::Number => {
                let token = self.advance();

                Some(SyntaxNode {
                    kind: SyntaxKind::Number,
                    span: token.span,
                    name: None,
                    children: Vec::new(),
                })
            }

            TypeScriptTokenKind::Boolean => {
                let token = self.advance();

                Some(SyntaxNode {
                    kind: SyntaxKind::Boolean,
                    span: token.span,
                    name: None,
                    children: Vec::new(),
                })
            }

            TypeScriptTokenKind::OpenBrace => self.parse_object(),

            TypeScriptTokenKind::OpenBracket => self.parse_array(),

            TypeScriptTokenKind::Identifier => {
                let token = self.advance();

                Some(SyntaxNode {
                    kind: SyntaxKind::Identifier,
                    span: token.span,
                    name: Some(self.source[token.span.start..token.span.end].to_string()),
                    children: Vec::new(),
                })
            }

            _ => None,
        }
    }

    fn parse_object(&mut self) -> Option<SyntaxNode> {
        let start = self.expect(TypeScriptTokenKind::OpenBrace)?.span.start;

        let mut children = Vec::new();

        self.skip_newlines();

        while !self.is_at_end() && !self.check(TypeScriptTokenKind::CloseBrace) {
            let name_token = self.expect(TypeScriptTokenKind::Identifier)?;
            let name_span = name_token.span;

            let name = self.source[name_span.start..name_span.end].to_string();

            self.expect(TypeScriptTokenKind::Colon)?;

            let value = self.parse_value()?;

            let end = value.span.end;

            children.push(SyntaxNode {
                kind: SyntaxKind::Property,
                span: Span::new(name_span.start, end),
                name: Some(name),
                children: vec![value],
            });

            self.skip_newlines();

            if self.check(TypeScriptTokenKind::Comma) {
                self.advance();
                self.skip_newlines();
            }
        }

        let end = self.expect(TypeScriptTokenKind::CloseBrace)?.span.end;

        Some(SyntaxNode {
            kind: SyntaxKind::Object,
            span: Span::new(start, end),
            name: None,
            children,
        })
    }

    fn parse_function_call(&mut self) -> Option<SyntaxNode> {
        let start = self.current_token()?.span.start;

        let first = self.expect(TypeScriptTokenKind::Identifier)?;
        let first_span = first.span;

        let mut name = self.source[first_span.start..first_span.end].to_string();
        let mut end = first_span.end;

        while self.check(TypeScriptTokenKind::Dot) {
            self.advance();

            let identifier = self.expect(TypeScriptTokenKind::Identifier)?;
            let span = identifier.span;

            name.push('.');
            name.push_str(&self.source[span.start..span.end]);

            end = span.end;
        }

        self.expect(TypeScriptTokenKind::OpenParen)?;

        let mut arguments = Vec::new();

        self.skip_newlines();

        while !self.is_at_end() && !self.check(TypeScriptTokenKind::CloseParen) {
            let argument = self.parse_value()?;
            end = argument.span.end;

            arguments.push(argument);

            self.skip_newlines();

            if self.check(TypeScriptTokenKind::Comma) {
                self.advance();
                self.skip_newlines();
            } else {
                break;
            }
        }

        let close_paren = self.expect(TypeScriptTokenKind::CloseParen)?;
        end = close_paren.span.end;

        Some(SyntaxNode {
            kind: SyntaxKind::Call,
            span: Span::new(start, end),
            name: Some(name),
            children: arguments,
        })
    }

    fn parse_array(&mut self) -> Option<SyntaxNode> {
        let start = self.expect(TypeScriptTokenKind::OpenBracket)?.span.start;

        let mut children = Vec::new();

        self.skip_newlines();

        while !self.is_at_end() && !self.check(TypeScriptTokenKind::CloseBracket) {
            children.push(self.parse_value()?);

            self.skip_newlines();

            if self.check(TypeScriptTokenKind::Comma) {
                self.advance();
                self.skip_newlines();
            }
        }

        let end = self.expect(TypeScriptTokenKind::CloseBracket)?.span.end;

        Some(SyntaxNode {
            kind: SyntaxKind::Array,
            span: Span::new(start, end),
            name: None,
            children,
        })
    }

    fn parse_variable_declaration(&mut self) -> Option<SyntaxNode> {
        if !self.check_any(&[
            TypeScriptTokenKind::Const,
            TypeScriptTokenKind::Let,
            TypeScriptTokenKind::Var,
        ]) {
            return None;
        }

        self.advance();

        let name_token = self.expect(TypeScriptTokenKind::Identifier)?;
        let name_span = name_token.span;

        let name = self.source[name_span.start..name_span.end].to_string();

        self.expect(TypeScriptTokenKind::Equals)?;

        let value = self.parse_value()?;

        let end = if self.check(TypeScriptTokenKind::Semicolon) {
            self.advance().span.end
        } else {
            value.span.end
        };

        Some(SyntaxNode {
            kind: SyntaxKind::Property,
            span: Span::new(name_span.start, end),
            name: Some(name),
            children: vec![value],
        })
    }

    fn parse_block(&mut self) -> Option<SyntaxNode> {
        let start = self.expect(TypeScriptTokenKind::OpenBrace)?.span.start;

        let mut children = Vec::new();

        self.skip_newlines();

        while !self.is_at_end() && !self.check(TypeScriptTokenKind::CloseBrace) {
            if let Some(node) = self.parse_variable_declaration() {
                children.push(node);
            } else if let Some(node) = self.parse_function_call() {
                children.push(node);
            } else {
                self.advance();
            }

            self.skip_newlines();
        }

        let end = self.expect(TypeScriptTokenKind::CloseBrace)?.span.end;

        Some(SyntaxNode {
            kind: SyntaxKind::Block,
            span: Span::new(start, end),
            name: None,
            children,
        })
    }

    fn current_token(&self) -> Option<&TypeScriptToken> {
        self.tokens.get(self.current)
    }

    fn advance(&mut self) -> TypeScriptToken {
        let token = self.tokens[self.current].clone();

        self.current += 1;

        token
    }

    fn check(&self, kind: TypeScriptTokenKind) -> bool {
        self.current_token().is_some_and(|token| token.kind == kind)
    }

    fn check_any(&self, kinds: &[TypeScriptTokenKind]) -> bool {
        self.current_token()
            .is_some_and(|token| kinds.contains(&token.kind))
    }

    fn expect(&mut self, kind: TypeScriptTokenKind) -> Option<TypeScriptToken> {
        if !self.check(kind) {
            return None;
        }

        Some(self.advance())
    }

    fn skip_newlines(&mut self) {
        while self.check(TypeScriptTokenKind::Newline) {
            self.advance();
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::lexer::TypeScriptLexer;

    #[test]
    fn should_parse_variable_declarations() {
        let input = "\
const name = \"Dhomini\";
let age = 25;
var active = true;
";

        let lexer = TypeScriptLexer;
        let tokens = lexer.tokenize(input);

        let parser = TypeScriptSyntaxParser::new(&tokens, input);

        for token in &tokens {
            println!(
                "{:?} => {:?}",
                token.kind,
                &input[token.span.start..token.span.end]
            );
        }

        let root = parser.parse();

        assert_eq!(root.kind, SyntaxKind::Root);
        assert_eq!(root.children.len(), 3);

        assert_eq!(root.children[0].kind, SyntaxKind::Property);
        assert_eq!(root.children[0].name.as_deref(), Some("name"));
        assert_eq!(root.children[0].children[0].kind, SyntaxKind::String);

        assert_eq!(root.children[1].kind, SyntaxKind::Property);
        assert_eq!(root.children[1].name.as_deref(), Some("age"));
        assert_eq!(root.children[1].children[0].kind, SyntaxKind::Number);

        assert_eq!(root.children[2].kind, SyntaxKind::Property);
        assert_eq!(root.children[2].name.as_deref(), Some("active"));
        assert_eq!(root.children[2].children[0].kind, SyntaxKind::Boolean);
    }

    #[test]
    fn should_parse_object_literal() {
        let input = "\
    const user = {
        name: \"Dhomini\",
        age: 25,
        active: true,
    };
    ";

        let lexer = TypeScriptLexer;
        let tokens = lexer.tokenize(input);

        let parser = TypeScriptSyntaxParser::new(&tokens, input);
        let root = parser.parse();

        assert_eq!(root.children.len(), 1);

        let user = &root.children[0];

        assert_eq!(user.kind, SyntaxKind::Property);
        assert_eq!(user.name.as_deref(), Some("user"));

        let object = &user.children[0];

        assert_eq!(object.kind, SyntaxKind::Object);
        assert_eq!(object.children.len(), 3);

        assert_eq!(object.children[0].name.as_deref(), Some("name"));
        assert_eq!(object.children[0].children[0].kind, SyntaxKind::String);

        assert_eq!(object.children[1].name.as_deref(), Some("age"));
        assert_eq!(object.children[1].children[0].kind, SyntaxKind::Number);

        assert_eq!(object.children[2].name.as_deref(), Some("active"));
        assert_eq!(object.children[2].children[0].kind, SyntaxKind::Boolean);
    }

    #[test]
    fn should_parse_nested_object_and_array() {
        let input = "\
    const config = {
        name: \"Code Weave\",
        ports: [8000, 8001, 8002],
        database: {
            host: \"localhost\",
            port: 5432,
        },
    };
    ";

        let lexer = TypeScriptLexer;
        let tokens = lexer.tokenize(input);

        let parser = TypeScriptSyntaxParser::new(&tokens, input);
        let root = parser.parse();

        assert_eq!(root.children.len(), 1);

        let config = &root.children[0];

        assert_eq!(config.kind, SyntaxKind::Property);
        assert_eq!(config.name.as_deref(), Some("config"));

        let object = &config.children[0];

        assert_eq!(object.kind, SyntaxKind::Object);
        assert_eq!(object.children.len(), 3);

        assert_eq!(object.children[0].name.as_deref(), Some("name"));
        assert_eq!(object.children[0].children[0].kind, SyntaxKind::String);

        let ports = &object.children[1];

        assert_eq!(ports.name.as_deref(), Some("ports"));
        assert_eq!(ports.children[0].kind, SyntaxKind::Array);
        assert_eq!(ports.children[0].children.len(), 3);

        assert_eq!(ports.children[0].children[0].kind, SyntaxKind::Number);
        assert_eq!(ports.children[0].children[1].kind, SyntaxKind::Number);
        assert_eq!(ports.children[0].children[2].kind, SyntaxKind::Number);

        let database = &object.children[2];

        assert_eq!(database.name.as_deref(), Some("database"));
        assert_eq!(database.children[0].kind, SyntaxKind::Object);
        assert_eq!(database.children[0].children.len(), 2);

        assert_eq!(
            database.children[0].children[0].name.as_deref(),
            Some("host")
        );

        assert_eq!(
            database.children[0].children[1].name.as_deref(),
            Some("port")
        );
    }

    #[test]
    fn should_parse_array_of_objects() {
        let input = "\
    const users = [
        {
            name: \"Dhomini\",
            age: 25,
        },
        {
            name: \"Maria\",
            age: 30,
        },
    ];
    ";

        let lexer = TypeScriptLexer;
        let tokens = lexer.tokenize(input);

        let parser = TypeScriptSyntaxParser::new(&tokens, input);
        let root = parser.parse();

        assert_eq!(root.children.len(), 1);

        let users = &root.children[0];

        assert_eq!(users.kind, SyntaxKind::Property);
        assert_eq!(users.name.as_deref(), Some("users"));

        let array = &users.children[0];

        assert_eq!(array.kind, SyntaxKind::Array);
        assert_eq!(array.children.len(), 2);

        let first_user = &array.children[0];

        assert_eq!(first_user.kind, SyntaxKind::Object);
        assert_eq!(first_user.children.len(), 2);

        assert_eq!(first_user.children[0].name.as_deref(), Some("name"));
        assert_eq!(first_user.children[0].children[0].kind, SyntaxKind::String);

        assert_eq!(first_user.children[1].name.as_deref(), Some("age"));
        assert_eq!(first_user.children[1].children[0].kind, SyntaxKind::Number);

        let second_user = &array.children[1];

        assert_eq!(second_user.kind, SyntaxKind::Object);
        assert_eq!(second_user.children.len(), 2);

        assert_eq!(second_user.children[0].name.as_deref(), Some("name"));
        assert_eq!(second_user.children[1].name.as_deref(), Some("age"));
    }

    #[test]
    fn should_parse_function_declaration() {
        let input = "\
    function greet(name: string, age: number) {
    }
    ";

        let lexer = TypeScriptLexer;
        let tokens = lexer.tokenize(input);

        let parser = TypeScriptSyntaxParser::new(&tokens, input);

        let root = parser.parse();

        assert_eq!(root.kind, SyntaxKind::Root);
        assert_eq!(root.children.len(), 1);

        let function = &root.children[0];

        assert_eq!(function.kind, SyntaxKind::Function);
        assert_eq!(function.name.as_deref(), Some("greet"));
    }

    #[test]
    fn should_parse_function_parameters() {
        let input = "\
    function greet(name: string, age: number) {
    }
    ";

        let lexer = TypeScriptLexer;
        let tokens = lexer.tokenize(input);

        let parser = TypeScriptSyntaxParser::new(&tokens, input);

        let root = parser.parse();

        let function = &root.children[0];

        assert_eq!(function.kind, SyntaxKind::Function);
        assert_eq!(function.name.as_deref(), Some("greet"));

        assert_eq!(function.children.len(), 3);

        assert_eq!(function.children[0].kind, SyntaxKind::Property);
        assert_eq!(function.children[0].name.as_deref(), Some("name"));

        assert_eq!(function.children[1].kind, SyntaxKind::Property);
        assert_eq!(function.children[1].name.as_deref(), Some("age"));
    }

    #[test]
    fn should_parse_function_body() {
        let input = "\
    function greet(name: string) {
        const message = \"Hello\";
        let count = 1;
    }
    ";

        let lexer = TypeScriptLexer;
        let tokens = lexer.tokenize(input);

        let parser = TypeScriptSyntaxParser::new(&tokens, input);
        let root = parser.parse();

        let function = &root.children[0];
        let block = function
            .children
            .iter()
            .find(|node| node.kind == SyntaxKind::Block)
            .unwrap();

        assert_eq!(block.children.len(), 2);

        assert_eq!(block.children[0].kind, SyntaxKind::Property);
        assert_eq!(block.children[0].name.as_deref(), Some("message"));
        assert_eq!(block.children[0].children[0].kind, SyntaxKind::String);

        assert_eq!(block.children[1].kind, SyntaxKind::Property);
        assert_eq!(block.children[1].name.as_deref(), Some("count"));
        assert_eq!(block.children[1].children[0].kind, SyntaxKind::Number);
    }

    #[test]
    fn should_parse_function_call() {
        let input = "\
    function greet(name: string) {
        console.log(name);
    }
    ";

        let lexer = TypeScriptLexer;
        let tokens = lexer.tokenize(input);

        let parser = TypeScriptSyntaxParser::new(&tokens, input);
        let root = parser.parse();

        let function = &root.children[0];
        let block = function
            .children
            .iter()
            .find(|node| node.kind == SyntaxKind::Block)
            .unwrap();

        assert_eq!(block.children.len(), 1);

        let call = &block.children[0];

        assert_eq!(call.kind, SyntaxKind::Call);
        assert_eq!(call.name.as_deref(), Some("console.log"));
    }

    #[test]
    fn should_parse_function_call_arguments() {
        let input = "\
    function greet(name: string) {
        console.log(name);
    }
    ";

        let lexer = TypeScriptLexer;
        let tokens = lexer.tokenize(input);

        let parser = TypeScriptSyntaxParser::new(&tokens, input);
        let root = parser.parse();

        let function = &root.children[0];

        let block = function
            .children
            .iter()
            .find(|node| node.kind == SyntaxKind::Block)
            .unwrap();

        let call = &block.children[0];

        assert_eq!(call.kind, SyntaxKind::Call);
        assert_eq!(call.name.as_deref(), Some("console.log"));

        assert_eq!(call.children.len(), 1);
        assert_eq!(call.children[0].kind, SyntaxKind::Identifier);
        assert_eq!(call.children[0].name.as_deref(), Some("name"));
    }
}
