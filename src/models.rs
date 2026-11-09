use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Todo {
    pub id: u64,
    pub title: String,
    pub completed: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateTodo {
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTodo {
    pub title: Option<String>,
    pub completed: Option<bool>,
}

impl Todo {
    pub fn new(id: u64, title: impl Into<String>) -> Self {
        Self {
            id,
            title: title.into().trim().to_string(),
            completed: false,
        }
    }

    pub fn validate_title(title: &str) -> bool {
        !title.trim().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_trims_title() {
        let t = Todo::new(1, "  hello  ");
        assert_eq!(t.title, "hello");
        assert!(!t.completed);
    }

    #[test]
    fn validate_title_rejects_empty() {
        assert!(!Todo::validate_title(""));
        assert!(!Todo::validate_title("   "));
        assert!(Todo::validate_title("x"));
    }
}
