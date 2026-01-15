
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Window {
    pub id: String,
    pub title: String,
    pub app_name: String,
    pub workspace: i32,
    pub screen: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct App {
    pub name: String,
    pub exec_path: String,
    pub icon: Option<String>,
    pub is_running: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum MacroAction {
    LaunchApp(String),
    Command(String),
    OpenUrl(String),
    TypeText(String),
    Sleep(u64),
    System(String),
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Macro {
    pub name: String,
    pub actions: Vec<MacroAction>,
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
