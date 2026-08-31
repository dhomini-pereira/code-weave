use code_weave_core::{ChunkKind, ChunkPath, CodeChunk, Language, Source, SyntaxKind, SyntaxNode};

pub struct YamlChunker;

impl YamlChunker {
    pub fn chunk(&self, root: &SyntaxNode, source: &Source) -> Vec<CodeChunk> {
        let mut chunks = Vec::new();

        let start = source.position(root.span.start);
        let end = source.position(root.span.end);

        chunks.push(CodeChunk {
            kind: ChunkKind::File,
            name: None,
            parent: None,
            path: None,
            language: Language::Yaml,
            start_line: start.line,
            end_line: end.line,
            content: source.span(root.span).to_string(),
        });

        for child in &root.children {
            self.visit(child, source, None, None, &mut chunks);
        }

        chunks
    }

    fn visit(
        &self,
        node: &SyntaxNode,
        source: &Source,
        parent: Option<&ChunkPath>,
        parent_path: Option<&ChunkPath>,
        chunks: &mut Vec<CodeChunk>,
    ) {
        match node.kind {
            SyntaxKind::Object => {
                self.visit_object(node, source, parent, parent_path, chunks);
            }

            SyntaxKind::Array => {
                self.visit_array(node, source, parent, parent_path, chunks);
            }

            SyntaxKind::Property
            | SyntaxKind::String
            | SyntaxKind::Number
            | SyntaxKind::Boolean
            | SyntaxKind::Null => {
                if node.name.is_some() {
                    self.visit_property(node, source, parent, parent_path, chunks);
                } else {
                    for child in &node.children {
                        self.visit(child, source, parent, parent_path, chunks);
                    }
                }
            }

            _ => {
                for child in &node.children {
                    self.visit(child, source, parent, parent_path, chunks);
                }
            }
        }
    }

    fn visit_array(
        &self,
        node: &SyntaxNode,
        source: &Source,
        parent: Option<&ChunkPath>,
        parent_path: Option<&ChunkPath>,
        chunks: &mut Vec<CodeChunk>,
    ) {
        let Some(name) = node.name.as_deref() else {
            for (index, child) in node.children.iter().enumerate() {
                let item_path = match parent_path {
                    Some(parent_path) => {
                        ChunkPath::new(format!("{}[{index}]", parent_path.as_str()))
                    }
                    None => ChunkPath::new(format!("[{index}]")),
                };

                let start = source.position(child.span.start);
                let end = source.position(child.span.end);

                let kind = match child.kind {
                    SyntaxKind::Object => ChunkKind::Object,
                    SyntaxKind::Array => ChunkKind::Array,
                    SyntaxKind::Property
                    | SyntaxKind::String
                    | SyntaxKind::Number
                    | SyntaxKind::Boolean
                    | SyntaxKind::Null => ChunkKind::Property,
                    _ => ChunkKind::Block,
                };

                chunks.push(CodeChunk {
                    kind,
                    name: Some(format!("[{index}]")),
                    parent: parent_path.cloned(),
                    path: Some(item_path.clone()),
                    language: Language::Yaml,
                    start_line: start.line,
                    end_line: end.line,
                    content: source.span(child.span).to_string(),
                });

                self.visit(child, source, Some(&item_path), Some(&item_path), chunks);
            }

            return;
        };

        let path = match parent_path {
            Some(parent_path) => {
                format!("{}.{}", parent_path.as_str(), name)
            }
            None => name.to_string(),
        };

        let path = ChunkPath::new(path);

        let start = source.position(node.span.start);
        let end = source.position(node.span.end);

        chunks.push(CodeChunk {
            kind: ChunkKind::Array,
            name: Some(name.to_string()),
            parent: parent.cloned(),
            path: Some(path.clone()),
            language: Language::Yaml,
            start_line: start.line,
            end_line: end.line,
            content: source.span(node.span).to_string(),
        });

        for (index, child) in node.children.iter().enumerate() {
            let item_path = ChunkPath::new(format!("{}[{index}]", path.as_str()));

            let start = source.position(child.span.start);
            let end = source.position(child.span.end);

            let kind = match child.kind {
                SyntaxKind::Object => ChunkKind::Object,
                SyntaxKind::Array => ChunkKind::Array,
                SyntaxKind::String => ChunkKind::Property,
                SyntaxKind::Number => ChunkKind::Property,
                SyntaxKind::Boolean => ChunkKind::Property,
                SyntaxKind::Null => ChunkKind::Property,
                _ => ChunkKind::Block,
            };

            chunks.push(CodeChunk {
                kind,
                name: Some(format!("[{index}]")),
                parent: Some(path.clone()),
                path: Some(item_path.clone()),
                language: Language::Yaml,
                start_line: start.line,
                end_line: end.line,
                content: source.span(child.span).to_string(),
            });

            self.visit(child, source, Some(&item_path), Some(&item_path), chunks);
        }
    }

    fn visit_object(
        &self,
        node: &SyntaxNode,
        source: &Source,
        parent: Option<&ChunkPath>,
        parent_path: Option<&ChunkPath>,
        chunks: &mut Vec<CodeChunk>,
    ) {
        let Some(name) = node.name.as_deref() else {
            for child in &node.children {
                self.visit(child, source, parent, parent_path, chunks);
            }

            return;
        };

        let path = match parent_path {
            Some(parent_path) => {
                format!("{}.{}", parent_path.as_str(), name)
            }
            None => name.to_string(),
        };

        let path = ChunkPath::new(path);

        let start = source.position(node.span.start);
        let end = source.position(node.span.end);

        chunks.push(CodeChunk {
            kind: ChunkKind::Object,
            name: Some(name.to_string()),
            parent: parent.cloned(),
            path: Some(path.clone()),
            language: Language::Yaml,
            start_line: start.line,
            end_line: end.line,
            content: source.span(node.span).to_string(),
        });

        for child in &node.children {
            self.visit(child, source, Some(&path), Some(&path), chunks);
        }
    }

    fn visit_property(
        &self,
        node: &SyntaxNode,
        source: &Source,
        parent: Option<&ChunkPath>,
        parent_path: Option<&ChunkPath>,
        chunks: &mut Vec<CodeChunk>,
    ) {
        let Some(name) = node.name.as_deref() else {
            return;
        };

        let path = match parent_path {
            Some(parent_path) => {
                format!("{}.{}", parent_path.as_str(), name)
            }
            None => name.to_string(),
        };

        let path = ChunkPath::new(path);

        let start = source.position(node.span.start);
        let end = source.position(node.span.end);

        chunks.push(CodeChunk {
            kind: ChunkKind::Property,
            name: Some(name.to_string()),
            parent: parent.cloned(),
            path: Some(path),
            language: Language::Yaml,
            start_line: start.line,
            end_line: end.line,
            content: source.span(node.span).to_string(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{lexer::YamlLexer, syntax::YamlSyntaxParser};

    #[test]
    fn should_create_chunks_for_nested_mapping() {
        let input = "\
database:
  host: localhost
  port: 5432
";

        let source = Source::new(input);

        let lexer = YamlLexer;
        let tokens = lexer.tokenize(input);

        let parser = YamlSyntaxParser::new(&tokens, input);
        let tree = parser.parse();

        let chunker = YamlChunker;
        let chunks = chunker.chunk(&tree, &source);

        assert_eq!(chunks.len(), 4);

        assert_eq!(chunks[0].kind, ChunkKind::File);

        assert_eq!(chunks[1].kind, ChunkKind::Object);
        assert_eq!(
            chunks[1].path.as_ref().map(ChunkPath::as_str),
            Some("database")
        );

        assert_eq!(chunks[2].kind, ChunkKind::Property);
        assert_eq!(
            chunks[2].path.as_ref().map(ChunkPath::as_str),
            Some("database.host")
        );
        assert_eq!(
            chunks[2].parent.as_ref().map(ChunkPath::as_str),
            Some("database")
        );

        assert_eq!(chunks[3].kind, ChunkKind::Property);
        assert_eq!(
            chunks[3].path.as_ref().map(ChunkPath::as_str),
            Some("database.port")
        );
        assert_eq!(
            chunks[3].parent.as_ref().map(ChunkPath::as_str),
            Some("database")
        );
    }

    #[test]
    fn should_create_chunks_for_sequence_of_objects() {
        let input = "\
    users:
      - name: Dhomini
        age: 25
      - name: Maria
        age: 30
    ";

        let source = Source::new(input);

        let lexer = YamlLexer;
        let tokens = lexer.tokenize(input);

        let parser = YamlSyntaxParser::new(&tokens, input);
        let tree = parser.parse();

        let chunker = YamlChunker;
        let chunks = chunker.chunk(&tree, &source);

        assert_eq!(chunks.len(), 8);

        assert_eq!(chunks[0].kind, ChunkKind::File);

        assert_eq!(chunks[1].kind, ChunkKind::Array);
        assert_eq!(
            chunks[1].path.as_ref().map(ChunkPath::as_str),
            Some("users")
        );

        assert_eq!(chunks[2].kind, ChunkKind::Object);
        assert_eq!(
            chunks[2].path.as_ref().map(ChunkPath::as_str),
            Some("users[0]")
        );
        assert_eq!(
            chunks[2].parent.as_ref().map(ChunkPath::as_str),
            Some("users")
        );

        assert_eq!(chunks[3].kind, ChunkKind::Property);
        assert_eq!(
            chunks[3].path.as_ref().map(ChunkPath::as_str),
            Some("users[0].name")
        );
        assert_eq!(
            chunks[3].parent.as_ref().map(ChunkPath::as_str),
            Some("users[0]")
        );

        assert_eq!(chunks[4].kind, ChunkKind::Property);
        assert_eq!(
            chunks[4].path.as_ref().map(ChunkPath::as_str),
            Some("users[0].age")
        );
        assert_eq!(
            chunks[4].parent.as_ref().map(ChunkPath::as_str),
            Some("users[0]")
        );

        assert_eq!(chunks[5].kind, ChunkKind::Object);
        assert_eq!(
            chunks[5].path.as_ref().map(ChunkPath::as_str),
            Some("users[1]")
        );
        assert_eq!(
            chunks[5].parent.as_ref().map(ChunkPath::as_str),
            Some("users")
        );

        assert_eq!(chunks[6].kind, ChunkKind::Property);
        assert_eq!(
            chunks[6].path.as_ref().map(ChunkPath::as_str),
            Some("users[1].name")
        );
        assert_eq!(
            chunks[6].parent.as_ref().map(ChunkPath::as_str),
            Some("users[1]")
        );

        assert_eq!(chunks[7].kind, ChunkKind::Property);
        assert_eq!(
            chunks[7].path.as_ref().map(ChunkPath::as_str),
            Some("users[1].age")
        );
        assert_eq!(
            chunks[7].parent.as_ref().map(ChunkPath::as_str),
            Some("users[1]")
        );
    }

    #[test]
    fn should_create_chunks_for_scalar_sequence() {
        let input = "\
    tags:
      - rust
      - typescript
      - postgres
    ";

        let source = Source::new(input);

        let lexer = YamlLexer;
        let tokens = lexer.tokenize(input);

        let parser = YamlSyntaxParser::new(&tokens, input);
        let tree = parser.parse();

        let chunker = YamlChunker;
        let chunks = chunker.chunk(&tree, &source);

        assert_eq!(chunks.len(), 5);

        assert_eq!(chunks[0].kind, ChunkKind::File);

        assert_eq!(chunks[1].kind, ChunkKind::Array);
        assert_eq!(chunks[1].path.as_ref().map(ChunkPath::as_str), Some("tags"));

        assert_eq!(chunks[2].kind, ChunkKind::Property);
        assert_eq!(
            chunks[2].path.as_ref().map(ChunkPath::as_str),
            Some("tags[0]")
        );
        assert_eq!(
            chunks[2].parent.as_ref().map(ChunkPath::as_str),
            Some("tags")
        );

        assert_eq!(chunks[3].kind, ChunkKind::Property);
        assert_eq!(
            chunks[3].path.as_ref().map(ChunkPath::as_str),
            Some("tags[1]")
        );
        assert_eq!(
            chunks[3].parent.as_ref().map(ChunkPath::as_str),
            Some("tags")
        );

        assert_eq!(chunks[4].kind, ChunkKind::Property);
        assert_eq!(
            chunks[4].path.as_ref().map(ChunkPath::as_str),
            Some("tags[2]")
        );
        assert_eq!(
            chunks[4].parent.as_ref().map(ChunkPath::as_str),
            Some("tags")
        );
    }

    #[test]
    fn should_create_chunks_for_nested_sequence() {
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

        let source = Source::new(input);

        let lexer = YamlLexer;
        let tokens = lexer.tokenize(input);

        let parser = YamlSyntaxParser::new(&tokens, input);
        let tree = parser.parse();

        let chunker = YamlChunker;
        let chunks = chunker.chunk(&tree, &source);

        assert_eq!(chunks.len(), 11);

        assert_eq!(chunks[0].kind, ChunkKind::File);

        assert_eq!(chunks[1].kind, ChunkKind::Array);
        assert_eq!(
            chunks[1].path.as_ref().map(ChunkPath::as_str),
            Some("users")
        );

        assert_eq!(chunks[2].kind, ChunkKind::Object);
        assert_eq!(
            chunks[2].path.as_ref().map(ChunkPath::as_str),
            Some("users[0]")
        );
        assert_eq!(
            chunks[2].parent.as_ref().map(ChunkPath::as_str),
            Some("users")
        );

        assert_eq!(chunks[3].kind, ChunkKind::Property);
        assert_eq!(
            chunks[3].path.as_ref().map(ChunkPath::as_str),
            Some("users[0].name")
        );
        assert_eq!(
            chunks[3].parent.as_ref().map(ChunkPath::as_str),
            Some("users[0]")
        );

        assert_eq!(chunks[4].kind, ChunkKind::Array);
        assert_eq!(
            chunks[4].path.as_ref().map(ChunkPath::as_str),
            Some("users[0].roles")
        );
        assert_eq!(
            chunks[4].parent.as_ref().map(ChunkPath::as_str),
            Some("users[0]")
        );

        assert_eq!(chunks[5].kind, ChunkKind::Property);
        assert_eq!(
            chunks[5].path.as_ref().map(ChunkPath::as_str),
            Some("users[0].roles[0]")
        );
        assert_eq!(
            chunks[5].parent.as_ref().map(ChunkPath::as_str),
            Some("users[0].roles")
        );

        assert_eq!(chunks[6].kind, ChunkKind::Property);
        assert_eq!(
            chunks[6].path.as_ref().map(ChunkPath::as_str),
            Some("users[0].roles[1]")
        );
        assert_eq!(
            chunks[6].parent.as_ref().map(ChunkPath::as_str),
            Some("users[0].roles")
        );

        assert_eq!(chunks[7].kind, ChunkKind::Object);
        assert_eq!(
            chunks[7].path.as_ref().map(ChunkPath::as_str),
            Some("users[1]")
        );
        assert_eq!(
            chunks[7].parent.as_ref().map(ChunkPath::as_str),
            Some("users")
        );

        assert_eq!(chunks[8].kind, ChunkKind::Property);
        assert_eq!(
            chunks[8].path.as_ref().map(ChunkPath::as_str),
            Some("users[1].name")
        );
        assert_eq!(
            chunks[8].parent.as_ref().map(ChunkPath::as_str),
            Some("users[1]")
        );

        assert_eq!(chunks[9].kind, ChunkKind::Array);
        assert_eq!(
            chunks[9].path.as_ref().map(ChunkPath::as_str),
            Some("users[1].roles")
        );
        assert_eq!(
            chunks[9].parent.as_ref().map(ChunkPath::as_str),
            Some("users[1]")
        );

        assert_eq!(chunks[10].kind, ChunkKind::Property);
        assert_eq!(
            chunks[10].path.as_ref().map(ChunkPath::as_str),
            Some("users[1].roles[0]")
        );
        assert_eq!(
            chunks[10].parent.as_ref().map(ChunkPath::as_str),
            Some("users[1].roles")
        );

        assert!(chunks.get(11).is_none());
    }
}
