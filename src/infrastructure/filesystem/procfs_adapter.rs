use crate::domain::model::App;
use crate::domain::ports::IProcessMonitor;
use std::fs;

pub struct ProcFsMonitorAdapter;

impl ProcFsMonitorAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl IProcessMonitor for ProcFsMonitorAdapter {
    fn get_running_pids(&self) -> Vec<u32> {
        let mut pids = Vec::new();
        if let Ok(entries) = fs::read_dir("/proc") {
            for entry in entries.flatten() {
                if let Ok(file_name) = entry.file_name().into_string() {
                    if let Ok(pid) = file_name.parse::<u32>() {
                        pids.push(pid);
                    }
                }
            }
        }
        pids
    }

    fn update_app_status(&self, apps: &mut [App]) {
        let mut running_commands = std::collections::HashSet::new();
        
        if let Ok(entries) = fs::read_dir("/proc") {
            for entry in entries.flatten() {
                if let Ok(file_name) = entry.file_name().into_string() {
                    if file_name.parse::<u32>().is_ok() {
                        let path = entry.path();
                        
                        // 1. Check `comm` (short name)
                        if let Ok(comm) = fs::read_to_string(path.join("comm")) {
                            running_commands.insert(comm.trim().to_string());
                        }
                        
                        // 2. Check `cmdline` (full args)
                        if let Ok(cmdline) = fs::read_to_string(path.join("cmdline")) {
                            // cmdline is null-separated
                            if let Some(first_arg) = cmdline.split('\0').next() {
                                if !first_arg.is_empty() {
                                    // Add the full path/command
                                    running_commands.insert(first_arg.to_string());
                                    // Also add just the binary name
                                    if let Some(name) = Path::new(first_arg).file_name().and_then(|n| n.to_str()) {
                                        running_commands.insert(name.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        for app in apps {
            // exec_path line e.g. "/usr/bin/firefox %u" or "gnome-terminal"
            // We want the primary command token
            let exec_clean = app.exec_path.split_whitespace().next().unwrap_or(&app.exec_path);
            let exec_bin = Path::new(exec_clean).file_name().and_then(|n| n.to_str()).unwrap_or(exec_clean);
            
            // Check if our exact binary name is running
            if running_commands.contains(exec_bin) {
                app.is_running = true;
                continue;
            }
            
            // Fallback: Check if any running command contains our binary name (fuzzy)
            // Or if our exec path contains the running command (e.g. wrapper scripts)
            // But strict matching is better to avoid false positives.
            // Let's try matching ignoring case?
            if running_commands.iter().any(|rc| rc.eq_ignore_ascii_case(exec_bin)) {
                app.is_running = true;
                continue; 
            }
            
            // Special case for some common browsers that have different process names
            // e.g. chrome -> chrome, google-chrome-stable
            if exec_bin.contains("google-chrome") && running_commands.iter().any(|rc| rc.contains("chrome")) {
                 app.is_running = true;
                 continue;
            }
            
            app.is_running = false;
        }
    }
}

use std::path::Path;
