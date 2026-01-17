use crate::domain::ports::IFileSystem;
use std::path::Path;

pub struct LocalFileSystemAdapter;

impl LocalFileSystemAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl IFileSystem for LocalFileSystemAdapter {
    fn list_dir(&self, path: &str) -> Vec<String> {
        let mut results = Vec::new();
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                 if let Ok(name) = entry.file_name().into_string() {
                     results.push(name);
                 }
            }
        }
        results
    }

    fn is_dir(&self, path: &str) -> bool {
        Path::new(path).is_dir()
    }

    fn exists(&self, path: &str) -> bool {
        Path::new(path).exists()
    }
}
