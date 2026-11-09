use crate::models::{Todo, UpdateTodo};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

#[derive(Serialize, Deserialize, Default)]
struct Db {
    #[serde(default = "default_next_id")]
    next_id: u64,
    #[serde(default)]
    todos: Vec<Todo>,
}

fn default_next_id() -> u64 {
    1
}

#[derive(Clone)]
pub struct JsonStore {
    path: PathBuf,
    inner: Arc<RwLock<Db>>,
}

impl JsonStore {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let path = path.as_ref().to_path_buf();
        let db = if path.exists() {
            let data = fs::read_to_string(&path)?;
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            Db {
                next_id: 1,
                todos: vec![],
            }
        };
        // normalize next_id if 0
        let db = if db.next_id == 0 {
            Db {
                next_id: 1,
                todos: db.todos,
            }
        } else {
            db
        };
        Ok(Self {
            path,
            inner: Arc::new(RwLock::new(db)),
        })
    }

    fn save(&self, db: &Db) -> Result<(), std::io::Error> {
        let data = serde_json::to_string_pretty(db).map_err(std::io::Error::other)?;
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&self.path, data)
    }

    pub fn all(&self) -> Vec<Todo> {
        self.inner.read().unwrap().todos.clone()
    }

    pub fn get(&self, id: u64) -> Option<Todo> {
        self.inner
            .read()
            .unwrap()
            .todos
            .iter()
            .find(|t| t.id == id)
            .cloned()
    }

    pub fn create(&self, title: String) -> Result<Todo, String> {
        if !Todo::validate_title(&title) {
            return Err("title must not be empty".into());
        }
        let mut db = self.inner.write().unwrap();
        let id = db.next_id;
        db.next_id += 1;
        let todo = Todo::new(id, title);
        db.todos.push(todo.clone());
        self.save(&db).map_err(|e| e.to_string())?;
        Ok(todo)
    }

    pub fn update(&self, id: u64, patch: UpdateTodo) -> Result<Option<Todo>, String> {
        let mut db = self.inner.write().unwrap();
        if let Some(t) = db.todos.iter_mut().find(|t| t.id == id) {
            if let Some(title) = patch.title {
                if !Todo::validate_title(&title) {
                    return Err("title must not be empty".into());
                }
                t.title = title.trim().to_string();
            }
            if let Some(completed) = patch.completed {
                t.completed = completed;
            }
            let updated = t.clone();
            self.save(&db).map_err(|e| e.to_string())?;
            Ok(Some(updated))
        } else {
            Ok(None)
        }
    }

    pub fn delete(&self, id: u64) -> Result<bool, String> {
        let mut db = self.inner.write().unwrap();
        let before = db.todos.len();
        db.todos.retain(|t| t.id != id);
        let removed = db.todos.len() < before;
        if removed {
            self.save(&db).map_err(|e| e.to_string())?;
        }
        Ok(removed)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn create_and_get() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("db.json");
        let store = JsonStore::new(&path).unwrap();
        let t = store.create("buy milk".into()).unwrap();
        assert_eq!(t.id, 1);
        assert_eq!(store.get(1).unwrap().title, "buy milk");
        assert_eq!(store.all().len(), 1);
    }

    #[test]
    fn empty_title_rejected() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("db.json");
        let store = JsonStore::new(&path).unwrap();
        assert!(store.create("   ".into()).is_err());
    }

    #[test]
    fn update_and_delete() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("db.json");
        let store = JsonStore::new(&path).unwrap();
        let t = store.create("task".into()).unwrap();
        let updated = store
            .update(
                t.id,
                UpdateTodo {
                    title: Some("done task".into()),
                    completed: Some(true),
                },
            )
            .unwrap()
            .unwrap();
        assert_eq!(updated.title, "done task");
        assert!(updated.completed);
        assert!(store.delete(t.id).unwrap());
        assert!(store.get(t.id).is_none());
        assert!(!store.delete(t.id).unwrap());
    }

    #[test]
    fn update_missing_returns_none() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("db.json");
        let store = JsonStore::new(&path).unwrap();
        let res = store
            .update(
                999,
                UpdateTodo {
                    title: None,
                    completed: Some(true),
                },
            )
            .unwrap();
        assert!(res.is_none());
    }

    #[test]
    fn persists_to_disk() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("db.json");
        {
            let store = JsonStore::new(&path).unwrap();
            store.create("persisted".into()).unwrap();
        }
        let store2 = JsonStore::new(&path).unwrap();
        assert_eq!(store2.all().len(), 1);
        assert_eq!(store2.all()[0].title, "persisted");
        // next_id continues
        let t2 = store2.create("second".into()).unwrap();
        assert_eq!(t2.id, 2);
    }

    #[test]
    fn get_missing_returns_none() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("db.json");
        let store = JsonStore::new(&path).unwrap();
        assert!(store.get(42).is_none());
        assert!(store.all().is_empty());
    }
}
