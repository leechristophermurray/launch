use crate::interface::ports::IShortcutRepository;
use std::collections::HashMap;

pub struct StaticShortcutAdapter {
    shortcuts: HashMap<String, String>,
}

impl StaticShortcutAdapter {
    pub fn new() -> Self {
        let shortcuts = HashMap::from([
            ("term".to_string(), "gnome-terminal".to_string()),
            ("calc".to_string(), "gnome-calculator".to_string()),
            ("files".to_string(), "nautilus".to_string()),
            ("web".to_string(), "firefox".to_string()),
        ]);
        Self { shortcuts }
    }
}

impl IShortcutRepository for StaticShortcutAdapter {
    fn get(&self, key: &str) -> Option<String> {
        self.shortcuts.get(key).cloned()
    }

    fn get_all(&self) -> HashMap<String, String> {
        self.shortcuts.clone()
    }
}
