use crate::{ParseError, Source};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub source_line: String,
}

impl Diagnostic {
    pub fn from_error(source: &Source, error: &ParseError) -> Self {
        let position = error.span.map(|span| source.position(span.start));

        let (line, column) = position
            .map(|position| (position.line, position.column))
            .unwrap_or((0, 0));

        let source_line = position
            .and_then(|position| {
                source
                    .content()
                    .lines()
                    .nth(position.line.saturating_sub(1))
            })
            .unwrap_or_default()
            .to_string();

        Self {
            message: error.message.clone(),
            line,
            column,
            source_line,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_diagnostic_from_error() {
        let source = Source::new("{\n  \"name\": \"Dhomini\"\n}");

        let error = ParseError::unexpected_token("expected comma", crate::Span::new(4, 10));

        let diagnostic = Diagnostic::from_error(&source, &error);

        assert_eq!(diagnostic.message, "expected comma");

        assert_eq!(diagnostic.line, 2);

        assert_eq!(diagnostic.source_line, "  \"name\": \"Dhomini\"");
    }
}
