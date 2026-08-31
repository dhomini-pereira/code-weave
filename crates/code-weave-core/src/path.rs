#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChunkPath(String);

impl ChunkPath {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<String> for ChunkPath {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for ChunkPath {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_path() {
        let path = ChunkPath::new("users[0].profile");

        assert_eq!(path.as_str(), "users[0].profile");
    }

    #[test]
    fn should_create_path_from_string() {
        let path = ChunkPath::from(String::from("users"));

        assert_eq!(path.as_str(), "users");
    }

    #[test]
    fn should_create_path_from_str() {
        let path = ChunkPath::from("users");

        assert_eq!(path.as_str(), "users");
    }

    #[test]
    fn should_compare_paths() {
        assert_eq!(ChunkPath::new("users"), ChunkPath::new("users"));
    }
}
