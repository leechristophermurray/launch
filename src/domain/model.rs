
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct App {
    pub name: String,
    pub exec_path: String,
    pub icon: Option<String>,
    pub is_running: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shortcut {
    pub key: String,
    pub command: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchMode {
    App,
    Settings,
    Command,
}
