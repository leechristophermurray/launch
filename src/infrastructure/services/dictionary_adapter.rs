use crate::domain::ports::IDictionaryService;

pub struct SmartDictionaryAdapter;

impl SmartDictionaryAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl IDictionaryService for SmartDictionaryAdapter {
    fn lookup(&self, term: &str) -> Option<String> {
        // Try webster offline dictionary
        // Returns None if not found, which triggers Google search fallback in UI
        webster::dictionary(term).map(|s| s.to_string())
    }
}

