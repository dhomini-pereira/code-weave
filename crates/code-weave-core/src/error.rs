use crate::{Diagnostic, Source, Span};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub message: String,
    pub span: Option<Span>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseErrorKind {
    UnexpectedToken,
    UnexpectedEndOfInput,
    InvalidSyntax,
}

impl ParseError {
    pub fn unexpected_token(message: impl Into<String>, span: Span) -> Self {
        Self {
            kind: ParseErrorKind::UnexpectedToken,
            message: message.into(),
            span: Some(span),
        }
    }

    pub fn unexpected_end_of_input(message: impl Into<String>) -> Self {
        Self {
            kind: ParseErrorKind::UnexpectedEndOfInput,
            message: message.into(),
            span: None,
        }
    }

    pub fn invalid_syntax(message: impl Into<String>) -> Self {
        Self {
            kind: ParseErrorKind::InvalidSyntax,
            message: message.into(),
            span: None,
        }
    }

    pub fn diagnostic(&self, source: &Source) -> Diagnostic {
        Diagnostic::from_error(source, self)
    }
}
