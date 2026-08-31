use code_weave_core::{Span, SyntaxKind, SyntaxNode};

use crate::lexer::{YamlToken, YamlTokenKind};

pub struct YamlSyntaxParser<'a> {
    tokens: &'a [YamlToken],
    source: &'a str,
    current: usize,
}

impl<'a> YamlSyntaxParser<'a> {
    pub fn new(tokens: &'a [YamlToken], source: &'a str) -> Self {
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
            if let Some(node) = self.parse_mapping_entry() {
                children.push(node);
            } else {
                self.advance();
            }

            self.skip_newlines();
        }

        let end = self.source.len();

        SyntaxNode {
            kind: SyntaxKind::Object,
            span: Span::new(start, end),
            name: None,
            children,
        }
    }

    fn parse_mapping_entry(&mut self) -> Option<SyntaxNode> {
        let key = self.current_token()?;

        if key.kind != YamlTokenKind::Key {
            return None;
        }

        let key_span = key.span;
        let name = self.source[key_span.start..key_span.end].to_string();

        self.advance();

        self.expect(YamlTokenKind::Colon)?;

        if self.check(YamlTokenKind::Scalar) {
            let value_token = self.advance();

            return Some(SyntaxNode {
                kind: SyntaxKind::String,
                span: Span::new(key_span.start, value_token.span.end),
                name: Some(name),
                children: Vec::new(),
            });
        }

        if self.check(YamlTokenKind::Newline) {
            self.advance();
        }

        if self.check(YamlTokenKind::Indent) {
            let indent = self.advance();

            if self.check(YamlTokenKind::Dash) {
                let children = self.parse_sequence_entries();

                let end = children
                    .last()
                    .map(|child| child.span.end)
                    .unwrap_or(indent.span.end);

                return Some(SyntaxNode {
                    kind: SyntaxKind::Array,
                    span: Span::new(key_span.start, end),
                    name: Some(name),
                    children,
                });
            }

            let children = self.parse_indented_entries();

            let end = children
                .last()
                .map(|child| child.span.end)
                .unwrap_or(indent.span.end);

            return Some(SyntaxNode {
                kind: SyntaxKind::Object,
                span: Span::new(key_span.start, end),
                name: Some(name),
                children,
            });
        }

        Some(SyntaxNode {
            kind: SyntaxKind::Null,
            span: key_span,
            name: Some(name),
            children: Vec::new(),
        })
    }

    fn parse_sequence_entries(&mut self) -> Vec<SyntaxNode> {
        let mut children = Vec::new();

        while !self.is_at_end() {
            self.skip_newlines();

            if !self.check(YamlTokenKind::Dash) {
                break;
            }

            let dash = self.advance();

            if self.check(YamlTokenKind::Key) {
                let Some(first) = self.parse_mapping_entry() else {
                    continue;
                };

                let mut object_children = vec![first];

                self.skip_newlines();

                if self.check(YamlTokenKind::Indent) {
                    self.advance();

                    let remaining = self.parse_indented_entries();

                    object_children.extend(remaining);
                }

                let end = object_children
                    .last()
                    .map(|child| child.span.end)
                    .unwrap_or(dash.span.end);

                children.push(SyntaxNode {
                    kind: SyntaxKind::Object,
                    span: Span::new(dash.span.start, end),
                    name: None,
                    children: object_children,
                });

                continue;
            }

            if self.check(YamlTokenKind::Scalar) {
                let value = self.advance();

                children.push(SyntaxNode {
                    kind: SyntaxKind::String,
                    span: Span::new(dash.span.start, value.span.end),
                    name: None,
                    children: Vec::new(),
                });

                continue;
            }

            children.push(SyntaxNode {
                kind: SyntaxKind::Null,
                span: dash.span,
                name: None,
                children: Vec::new(),
            });
        }

        if self.check(YamlTokenKind::Dedent) {
            self.advance();
        }

        children
    }

    fn parse_indented_entries(&mut self) -> Vec<SyntaxNode> {
        let mut children = Vec::new();

        self.skip_newlines();

        while !self.is_at_end() {
            if self.check(YamlTokenKind::Dedent) {
                self.advance();
                break;
            }

            if let Some(node) = self.parse_mapping_entry() {
                children.push(node);
            } else {
                self.advance();
            }

            self.skip_newlines();
        }

        children
    }

    fn current_token(&self) -> Option<&YamlToken> {
        self.tokens.get(self.current)
    }

    fn advance(&mut self) -> YamlToken {
        let token = self.tokens[self.current].clone();

        self.current += 1;

        token
    }

    fn check(&self, kind: YamlTokenKind) -> bool {
        self.current_token().is_some_and(|token| token.kind == kind)
    }

    fn expect(&mut self, kind: YamlTokenKind) -> Option<YamlToken> {
        if !self.check(kind) {
            return None;
        }

        Some(self.advance())
    }

    fn skip_newlines(&mut self) {
        while self.check(YamlTokenKind::Newline) {
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

    use crate::lexer::YamlLexer;

    #[test]
    fn should_parse_simple_mapping() {
        let input = "\
name: Dhomini
age: 25
";

        let lexer = YamlLexer;
        let tokens = lexer.tokenize(input);

        let parser = YamlSyntaxParser::new(&tokens, input);

        let root = parser.parse();

        assert_eq!(root.kind, SyntaxKind::Object);
        assert_eq!(root.children.len(), 2);

        assert_eq!(root.children[0].name.as_deref(), Some("name"));
        assert_eq!(root.children[0].kind, SyntaxKind::String);

        assert_eq!(root.children[1].name.as_deref(), Some("age"));
        assert_eq!(root.children[1].kind, SyntaxKind::String);
    }

    #[test]
    fn should_parse_nested_mapping() {
        let input = "\
database:
  host: localhost
  port: 5432
";

        let lexer = YamlLexer;
        let tokens = lexer.tokenize(input);

        let parser = YamlSyntaxParser::new(&tokens, input);

        let root = parser.parse();

        assert_eq!(root.children.len(), 1);

        let database = &root.children[0];

        assert_eq!(database.kind, SyntaxKind::Object);
        assert_eq!(database.name.as_deref(), Some("database"));
        assert_eq!(database.children.len(), 2);

        assert_eq!(database.children[0].name.as_deref(), Some("host"));
        assert_eq!(database.children[0].kind, SyntaxKind::String);

        assert_eq!(database.children[1].name.as_deref(), Some("port"));
        assert_eq!(database.children[1].kind, SyntaxKind::String);
    }

    #[test]
    fn should_parse_deeply_nested_mapping() {
        let input = "\
database:
  connection:
    credentials:
      username: admin
      password: secret
";

        let lexer = YamlLexer;
        let tokens = lexer.tokenize(input);

        let parser = YamlSyntaxParser::new(&tokens, input);

        let root = parser.parse();

        let database = &root.children[0];
        let connection = &database.children[0];
        let credentials = &connection.children[0];

        assert_eq!(credentials.name.as_deref(), Some("credentials"));
        assert_eq!(credentials.children.len(), 2);

        assert_eq!(credentials.children[0].name.as_deref(), Some("username"));

        assert_eq!(credentials.children[1].name.as_deref(), Some("password"));
    }

    #[test]
    fn should_parse_scalar_sequence() {
        let input = "\
users:
  - Dhomini
  - Maria
";

        let lexer = YamlLexer;
        let tokens = lexer.tokenize(input);

        let parser = YamlSyntaxParser::new(&tokens, input);

        let root = parser.parse();

        assert_eq!(root.children.len(), 1);

        let users = &root.children[0];

        assert_eq!(users.kind, SyntaxKind::Array);
        assert_eq!(users.name.as_deref(), Some("users"));
        assert_eq!(users.children.len(), 2);

        assert_eq!(users.children[0].kind, SyntaxKind::String);
        assert_eq!(users.children[1].kind, SyntaxKind::String);
    }

    #[test]
    fn should_parse_sequence_of_objects() {
        let input = "\
    users:
      - name: Dhomini
        age: 25
      - name: Maria
        age: 30
    ";

        let lexer = YamlLexer;
        let tokens = lexer.tokenize(input);

        let parser = YamlSyntaxParser::new(&tokens, input);

        let root = parser.parse();

        assert_eq!(root.children.len(), 1);

        let users = &root.children[0];

        assert_eq!(users.kind, SyntaxKind::Array);
        assert_eq!(users.name.as_deref(), Some("users"));
        assert_eq!(users.children.len(), 2);

        let first = &users.children[0];

        assert_eq!(first.kind, SyntaxKind::Object);
        assert_eq!(first.children.len(), 2);

        assert_eq!(first.children[0].name.as_deref(), Some("name"));
        assert_eq!(first.children[1].name.as_deref(), Some("age"));

        let second = &users.children[1];

        assert_eq!(second.kind, SyntaxKind::Object);
        assert_eq!(second.children.len(), 2);

        assert_eq!(second.children[0].name.as_deref(), Some("name"));
        assert_eq!(second.children[1].name.as_deref(), Some("age"));
    }

    #[test]
    fn should_debug_nested_sequence() {
        let input = "\
    users:
      - name: Dhomini
        roles:
          - admin
          - developer
      - name: Maria
        roles:
          - user
    ";

        let lexer = YamlLexer;
        let tokens = lexer.tokenize(input);

        let parser = YamlSyntaxParser::new(&tokens, input);
        let root = parser.parse();

        fn print_tree(node: &SyntaxNode, depth: usize) {
            println!(
                "{}{:?} {:?} {:?}",
                "  ".repeat(depth),
                node.kind,
                node.name,
                node.span
            );

            for child in &node.children {
                print_tree(child, depth + 1);
            }
        }

        print_tree(&root, 0);
    }
}
