pub mod lexer;
pub mod syntax;

pub use lexer::{TypeScriptLexer, TypeScriptToken, TypeScriptTokenKind};
pub use syntax::TypeScriptSyntaxParser;
