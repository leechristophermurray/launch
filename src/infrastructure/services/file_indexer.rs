use crate::domain::ports::IFileIndexer;
use walkdir::WalkDir;
use std::sync::{Arc, Mutex};
use std::path::PathBuf;

pub struct FileIndexerAdapter {
    index: Arc<Mutex<Vec<String>>>,
    home_dir: PathBuf,
}

impl FileIndexerAdapter {
    pub fn new() -> Self {
        let home_dir = dirs::home_dir().expect("Could not find home directory");
        Self {
            index: Arc::new(Mutex::new(Vec::new())),
            home_dir,
        }
    }
}

impl IFileIndexer for FileIndexerAdapter {
    fn search(&self, term: &str) -> Vec<String> {
        let term_lower = term.to_lowercase();
        let index = self.index.lock().unwrap();
        index.iter()
            .filter(|path| path.to_lowercase().contains(&term_lower))
            .take(10) // Limit context to top 10 matches
            .cloned()
            .collect()
    }

    fn index_home(&self) {
        let index_clone = self.index.clone();
        let home = self.home_dir.clone();
        
        std::thread::spawn(move || {
            let mut paths = Vec::new();
            println!("Starting file indexing of {}...", home.display());
            
            // Allowed extensions for "source code / documents" awareness
            let allowed_extensions: Vec<&str> = vec![
                "md", "txt", "rs", "toml", "json", "yaml", "yml", "js", "ts", "py", "sh", "html", "css"
            ];

            for entry in WalkDir::new(&home)
                .max_depth(5) 
                .into_iter()
                .filter_entry(|e| {
                    let name = e.file_name().to_string_lossy();
                    // strict exclusions
                    !name.starts_with('.') && 
                    name != "node_modules" && 
                    name != "target" && 
                    name != "dist" && 
                    name != "build" &&
                    name != "venv"
                })
                .filter_map(|e| e.ok()) 
            {
                if entry.file_type().is_file() {
                    let path = entry.path();
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        if allowed_extensions.contains(&ext.to_lowercase().as_str()) {
                            paths.push(path.to_string_lossy().to_string());
                        }
                    }
                }
            }
            
            let mut index = index_clone.lock().unwrap();
            *index = paths;
            println!("Indexing complete. Found {} locations.", index.len());
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_indexing_files() {
        // Create a temp directory
        let dir = tempdir().unwrap();
        let path = dir.path();

        // Create a non-hidden subdirectory to act as home, 
        // because tempdir() creates a hidden folder starting with .tmp, 
        // which our indexer logic correctly ignores.
        let home_path = path.join("home");
        std::fs::create_dir(&home_path).unwrap();

        // Create some files
        let file1_path = home_path.join("test.txt");
        let mut file1 = File::create(&file1_path).unwrap();
        writeln!(file1, "Hello World").unwrap();

        let file2_path = home_path.join("ignored.bin");
        let mut file2 = File::create(&file2_path).unwrap();
        writeln!(file2, "Binary data").unwrap();

        let nested_dir = home_path.join("nested");
        std::fs::create_dir(&nested_dir).unwrap();
        let file3_path = nested_dir.join("deep.rs");
        let mut file3 = File::create(&file3_path).unwrap();
        writeln!(file3, "fn main() {{}}").unwrap();

        // Mock indexer with this temp dir as home
        let indexer = FileIndexerAdapter {
            index: Arc::new(Mutex::new(Vec::new())),
            home_dir: home_path,
        };

        // Run index_home 
        indexer.index_home();
        
        // Wait for thread
        std::thread::sleep(std::time::Duration::from_secs(2));

        // Create a search term (should match "test" in "test.txt")
        let results = indexer.search("test");
        assert!(!results.is_empty(), "Should find test.txt");
        // We expect the full path
        assert!(results.iter().any(|p| p.contains("test.txt")), "Results should contain test.txt");

        // Check index directly
        let index = indexer.index.lock().unwrap();
        assert!(index.iter().any(|p| p.contains("test.txt")), "Index should contain test.txt");
        assert!(index.iter().any(|p| p.contains("deep.rs")), "Index should contain deep.rs");
        
        // Should NOT contain ignored.bin (not in allowed extensions)
        assert!(!index.iter().any(|p| p.contains("ignored.bin")), "Index should NOT contain ignored.bin");
    }
}
