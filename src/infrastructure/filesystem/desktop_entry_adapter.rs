use crate::domain::model::App;
use crate::domain::ports::IAppRepository;
use walkdir::WalkDir;
use std::fs;
use std::path::Path;
use regex::Regex;
use once_cell::sync::Lazy;

pub struct LinuxAppRepoAdapter;

impl LinuxAppRepoAdapter {
    pub fn new() -> Self {
        Self
    }

    fn parse_desktop_file(path: &Path) -> Option<App> {
        let content = fs::read_to_string(path).ok()?;
        
        static RE_NAME: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?m)^Name=(.*)$").unwrap());
        static RE_EXEC: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?m)^Exec=(.*)$").unwrap());
        static RE_ICON: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?m)^Icon=(.*)$").unwrap());
        static RE_NO_DISPLAY: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?m)^NoDisplay=true$").unwrap());

        if RE_NO_DISPLAY.is_match(&content) {
            return None;
        }

        let name = RE_NAME.captures(&content)?.get(1)?.as_str().trim().to_string();
        let exec_raw = RE_EXEC.captures(&content)?.get(1)?.as_str().trim();
        
        // Clean up Exec command (remove %u, %F, etc.)
        let exec_path = exec_raw.split_whitespace().next()?.to_string();

        let icon = RE_ICON.captures(&content).map(|c| c.get(1).unwrap().as_str().trim().to_string());

        Some(App {
            name,
            exec_path,
            icon,
            is_running: false, // Default state
            is_favorite: false,
        })
    }
}

impl IAppRepository for LinuxAppRepoAdapter {
    fn find_apps(&self) -> Vec<App> {
        let mut apps = Vec::new();
        let paths = vec![
            "/usr/share/applications",
            "/usr/local/share/applications",
            // Add home directory handling carefully if needed, relying on std::env::home_dir is deprecated, 
            // but we can assume linux environment.
        ];
        
        // Expand home dir manually for now or use a crate if strictly needed, 
        // but let's stick to system paths + simple home expansion
        let home = std::env::var("HOME").unwrap_or_default();
        let local_apps = format!("{}/.local/share/applications", home);
        
        let all_paths = [paths.as_slice(), &[local_apps.as_str()]].concat();

        for dir in all_paths {
            for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
                if entry.path().extension().map_or(false, |ext| ext == "desktop") {
                    if let Some(app) = Self::parse_desktop_file(entry.path()) {
                        apps.push(app);
                    }
                }
            }
        }
        
        // Deduplicate based on name + exec or just assume list is fine? 
        // Let's simple-dedup by name for now just in case.
        apps.sort_by(|a, b| a.name.cmp(&b.name));
        apps.dedup_by(|a, b| a.name == b.name);

        apps
    }
}
