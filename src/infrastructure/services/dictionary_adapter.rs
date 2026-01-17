use crate::domain::ports::IDictionaryService;
use webster;

pub struct SmartDictionaryAdapter;

impl SmartDictionaryAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl IDictionaryService for SmartDictionaryAdapter {
    fn lookup(&self, term: &str) -> Option<String> {
        // webster::dictionary returns Option<&str>
        webster::dictionary(term).map(|s| s.to_string())
    }
}
