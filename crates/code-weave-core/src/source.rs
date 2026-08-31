#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub byte: usize,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub const fn len(self) -> usize {
        self.end - self.start
    }

    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }
}

pub struct Source<'a> {
    content: &'a str,
    line_starts: Vec<usize>,
}

impl<'a> Source<'a> {
    pub fn new(content: &'a str) -> Self {
        let mut line_starts = vec![0];

        for (index, byte) in content.bytes().enumerate() {
            if byte == b'\n' {
                line_starts.push(index + 1);
            }
        }

        Self {
            content,
            line_starts,
        }
    }

    pub fn content(&self) -> &'a str {
        self.content
    }

    pub fn position(&self, byte: usize) -> Position {
        let line = self
            .line_starts
            .partition_point(|&start| start <= byte)
            .saturating_sub(1);

        let line_start = self.line_starts[line];

        Position {
            byte,
            line: line + 1,
            column: byte - line_start + 1,
        }
    }

    pub fn span(&self, span: Span) -> &'a str {
        &self.content[span.start..span.end]
    }

    pub fn line(&self, line: usize) -> Option<&'a str> {
        if line == 0 {
            return None;
        }

        let index = line - 1;

        let start = *self.line_starts.get(index)?;

        let end = self
            .line_starts
            .get(index + 1)
            .copied()
            .unwrap_or(self.content.len());

        Some(self.content[start..end].trim_end_matches('\n'))
    }
}

#[cfg(test)]
mod tests {
    use std::assert_eq;

    use super::*;

    #[test]
    fn should_get_position_from_byte_offset() {
        let source = Source::new("hello\nworld");

        let position = source.position(6);

        assert_eq!(
            position,
            Position {
                byte: 6,
                line: 2,
                column: 1,
            }
        );
    }

    #[test]
    fn should_get_line_content() {
        let source = Source::new("hello\nworld\nrust");

        assert_eq!(source.line(1), Some("hello"));
        assert_eq!(source.line(2), Some("world"));
        assert_eq!(source.line(3), Some("rust"));
        assert_eq!(source.line(4), None);
    }

    #[test]
    fn should_get_content_from_span() {
        let source = Source::new("hello world");

        let content = source.span(Span { start: 0, end: 5 });

        assert_eq!(content, "hello");
    }

    #[test]
    fn should_handle_utf8() {
        let source = Source::new("Olá\nMundo");

        let position = source.position(5);

        assert_eq!(position.line, 2);
        assert_eq!(position.column, 1);
    }
}
