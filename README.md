# Code Weave

A fast, extensible, language-agnostic code parsing and chunking engine.

Code Weave transforms source files into structured code chunks while preserving their hierarchy, paths, source spans, and context. It is designed to provide a consistent foundation for indexing, searching, analyzing, and building knowledge bases from source code.

## Overview

Code Weave processes source files through a simple pipeline:

```text
Source File
    │
    ▼
  Lexer
    │
    ▼
  Tokens
    │
    ▼
Syntax Parser
    │
    ▼
 Syntax Tree
    │
    ▼
  Chunker
    │
    ▼
 Code Chunks
```

The resulting chunks retain structural information about the original source, making them suitable for downstream applications such as:

* Code search
* Semantic indexing
* Retrieval-Augmented Generation (RAG)
* Code intelligence
* Documentation generation
* Repository analysis
* Knowledge base construction

## Architecture

The project is organized as a Rust workspace with a shared core and format-specific implementations.

```text
code-weave/
├── crates/
│   ├── code-weave-core/
│   ├── code-weave-json/
│   ├── code-weave-yaml/
│   └── code-weave-typescript/
├── Cargo.toml
└── README.md
```

### `code-weave-core`

Contains the shared data structures and abstractions used by all supported formats.

Some of the core concepts include:

* `Source`
* `Span`
* `SyntaxNode`
* `SyntaxKind`
* `CodeChunk`
* `ChunkKind`
* `ChunkPath`
* `Language`

### Format crates

Each supported format is implemented independently and follows the same general architecture:

```text
Lexer → Syntax Parser → Chunker
```

This keeps format-specific parsing logic isolated while allowing all formats to produce the same `CodeChunk` representation.

## Code Chunks

A `CodeChunk` represents a meaningful structural portion of a source file.

Each chunk contains information such as:

```rust
pub struct CodeChunk {
    pub kind: ChunkKind,
    pub name: Option<String>,
    pub parent: Option<ChunkPath>,
    pub path: Option<ChunkPath>,
    pub language: Language,
    pub start_line: usize,
    pub end_line: usize,
    pub content: String,
}
```

For example, a YAML document such as:

```yaml
users:
  - name: Dhomini
    roles:
      - admin
      - developer
```

can be represented using hierarchical paths such as:

```text
users
users[0]
users[0].name
users[0].roles
users[0].roles[0]
users[0].roles[1]
```

This provides a stable structural representation that can be consumed by indexing or retrieval systems.

## Supported Formats

### JSON

* Object parsing
* Nested objects
* Arrays
* Arrays of objects
* Nested array paths
* Source spans
* Structural chunking

### YAML

* Mappings
* Nested mappings
* Scalars
* Indentation and dedentation
* Sequences
* Sequences of objects
* Nested sequences
* Structural chunking
* Hierarchical paths
* Source spans

### TypeScript

* Variable declarations (const, let, var)
* Primitive values
* Object literals
* Nested objects
* Arrays
* Arrays of objects
* Nested arrays
* Function declarations
* Function parameters
* Function bodies
* Function calls
* Function call arguments
* Comments
* Arrow functions
* Structural syntax parsing
* Source spans
* Hierarchical syntax nodes

### Planned

Additional language and data-format support will be added progressively.

Potential targets include:

* TOML
* JavaScript
* Python
* Rust
* Markdown

## Design Goals

### Language Agnostic

Different source formats should produce a common structural representation.

### Structural Preservation

Chunks should preserve meaningful relationships from the original source, including:

* Parent-child relationships
* Hierarchical paths
* Source locations
* Original source content

### Extensibility

Adding support for a new language or format should not require changing the existing implementations unnecessarily.

### Deterministic Output

Given the same source input, the lexer, parser, and chunker should produce deterministic results.

### Source-Aware

Code Weave works with source spans rather than relying exclusively on reconstructed values. This allows chunks to retain their original source context.

## Development

### Requirements

* Rust
* Cargo

### Check the workspace

```bash
cargo check
```

### Run all tests

```bash
cargo test
```

### Format the code

```bash
cargo fmt
```

### Run tests for a specific crate

```bash
cargo test -p code-weave-json
```

```bash
cargo test -p code-weave-yaml
```

```bash
cargo test -p code-weave-typescript
```

## Testing

Code Weave uses unit tests extensively across the lexer, syntax parser, and chunker layers.

The test suite currently covers scenarios such as:

* Tokenization
* Source span preservation
* Nested structures
* Arrays and sequences
* Nested paths
* Parent-child relationships
* Syntax tree construction
* Chunk generation

Run the complete workspace test suite with:

```bash
cargo test
```

## Roadmap

The project is being developed incrementally.

Current focus:

```text
Core       ✓
JSON       ✓
YAML       ✓
TypeScript ✓
JavaScript → Next
```

Future work will expand language support and improve the abstractions used for parsing, chunking, indexing, and retrieval.

## License

Code Weave is licensed under the [MIT License](LICENSE).
