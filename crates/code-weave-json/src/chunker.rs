use code_weave_core::{ChunkKind, ChunkPath, CodeChunk, Language, Source, SyntaxKind, SyntaxNode};

pub struct JsonChunker;

impl JsonChunker {
    pub fn chunk(&self, root: &SyntaxNode, source: &Source) -> Vec<CodeChunk> {
        let mut chunks = Vec::new();

        let start = source.position(root.span.start);
        let end = source.position(root.span.end);

        chunks.push(CodeChunk {
            kind: ChunkKind::File,
            name: None,
            parent: None,
            path: None,
            language: Language::Json,
            start_line: start.line,
            end_line: end.line,
            content: source.span(root.span).to_string(),
        });

        self.visit(root, source, None, None, &mut chunks);

        chunks
    }

    fn visit(
        &self,
        node: &SyntaxNode,
        source: &Source,
        parent: Option<&str>,
        parent_path: Option<&str>,
        chunks: &mut Vec<CodeChunk>,
    ) {
        match node.kind {
            SyntaxKind::Object => {
                self.visit_object(node, source, parent, parent_path, chunks);
            }

            SyntaxKind::Array => {
                self.visit_array(node, source, parent, parent_path, chunks);
            }

            _ => {
                for child in &node.children {
                    self.visit(child, source, parent, parent_path, chunks);
                }
            }
        }
    }

    fn visit_object(
        &self,
        node: &SyntaxNode,
        source: &Source,
        parent: Option<&str>,
        parent_path: Option<&str>,
        chunks: &mut Vec<CodeChunk>,
    ) {
        let Some(name) = node.name.as_deref() else {
            if let Some(path) = parent_path {
                let start = source.position(node.span.start);
                let end = source.position(node.span.end);

                chunks.push(CodeChunk {
                    kind: ChunkKind::Object,
                    name: parent.map(str::to_string),
                    parent: parent_path
                        .and_then(|path| path.rsplit_once('[').map(|(parent, _)| parent))
                        .map(ChunkPath::new),
                    path: Some(ChunkPath::new(path)),
                    language: Language::Json,
                    start_line: start.line,
                    end_line: end.line,
                    content: source.span(node.span).to_string(),
                });
            }

            for child in &node.children {
                self.visit(child, source, parent, parent_path, chunks);
            }

            return;
        };

        let path = match parent_path {
            Some(parent_path) => {
                format!("{parent_path}.{name}")
            }
            None => name.to_string(),
        };

        let start = source.position(node.span.start);
        let end = source.position(node.span.end);

        chunks.push(CodeChunk {
            kind: ChunkKind::Object,
            name: Some(name.to_string()),
            parent: parent.map(ChunkPath::new),
            path: Some(ChunkPath::new(path.clone())),
            language: Language::Json,
            start_line: start.line,
            end_line: end.line,
            content: source.span(node.span).to_string(),
        });

        for child in &node.children {
            self.visit(child, source, Some(name), Some(&path), chunks);
        }
    }

    fn visit_array(
        &self,
        node: &SyntaxNode,
        source: &Source,
        parent: Option<&str>,
        parent_path: Option<&str>,
        chunks: &mut Vec<CodeChunk>,
    ) {
        let array_path = node.name.as_deref().map(|name| match parent_path {
            Some(parent_path) => {
                format!("{parent_path}.{name}")
            }
            None => name.to_string(),
        });

        if let Some(name) = node.name.as_deref() {
            let start = source.position(node.span.start);
            let end = source.position(node.span.end);

            chunks.push(CodeChunk {
                kind: ChunkKind::Array,
                name: Some(name.to_string()),
                parent: parent.map(ChunkPath::new),
                path: array_path.clone().map(ChunkPath::new),
                language: Language::Json,
                start_line: start.line,
                end_line: end.line,
                content: source.span(node.span).to_string(),
            });
        }

        for (index, child) in node.children.iter().enumerate() {
            let path = match array_path.as_deref() {
                Some(path) => format!("{path}[{index}]"),
                None => format!("[{index}]"),
            };

            self.visit(child, source, Some(&path), Some(&path), chunks);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{lexer::JsonLexer, syntax::JsonSyntaxParser};

    #[test]
    fn should_create_chunks_for_array_objects() {
        let input = r#"{
  "users": [
    {
      "name": "Dhomini"
    },
    {
      "name": "Maria"
    }
  ]
}"#;

        let source = Source::new(input);

        let lexer = JsonLexer;
        let tokens = lexer.tokenize(input);

        let parser = JsonSyntaxParser::new(&tokens, input);

        let tree = parser.parse().unwrap();

        let chunker = JsonChunker;

        let chunks = chunker.chunk(&tree, &source);

        for chunk in &chunks {
            println!("{:?} {:?} {:?}", chunk.kind, chunk.name, chunk.path);
        }
    }

    #[test]
    fn should_create_nested_paths_inside_arrays() {
        let input = r#"{
      "users": [
        {
          "profile": {
            "name": "Dhomini"
          }
        }
      ]
    }"#;

        let source = Source::new(input);

        let lexer = JsonLexer;
        let tokens = lexer.tokenize(input);

        let parser = JsonSyntaxParser::new(&tokens, input);

        let tree = parser.parse().unwrap();

        let chunker = JsonChunker;

        let chunks = chunker.chunk(&tree, &source);

        assert_eq!(chunks.len(), 4);

        assert_eq!(chunks[0].kind, ChunkKind::File);

        assert_eq!(
            chunks[1].path.as_ref().map(ChunkPath::as_str),
            Some("users")
        );

        assert_eq!(
            chunks[2].path.as_ref().map(ChunkPath::as_str),
            Some("users[0]")
        );

        assert_eq!(
            chunks[2].parent.as_ref().map(ChunkPath::as_str),
            Some("users")
        );

        assert_eq!(
            chunks[3].path.as_ref().map(ChunkPath::as_str),
            Some("users[0].profile")
        );

        assert_eq!(
            chunks[3].parent.as_ref().map(ChunkPath::as_str),
            Some("users[0]")
        );
    }
}
