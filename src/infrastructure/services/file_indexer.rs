use crate::interface::ports::IFileIndexer;
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
            
            for entry in WalkDir::new(&home)
                .max_depth(4) // Don't go too deep for performance
                .into_iter()
                .filter_entry(|e| {
                    let name = e.file_name().to_string_lossy();
                    !name.starts_with('.') && name != "node_modules" && name != "target"
                })
                .filter_map(|e| e.ok()) 
            {
                if entry.file_type().is_dir() {
                    paths.push(entry.path().to_string_lossy().to_string());
                }
            }
            
            let mut index = index_clone.lock().unwrap();
            *index = paths;
            println!("Indexing complete. Found {} locations.", index.len());
        });
    }
}
