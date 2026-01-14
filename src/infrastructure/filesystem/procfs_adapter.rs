use crate::domain::model::App;
use crate::interface::ports::IProcessMonitor;
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
        // Simple approach: get all command lines from /proc and partial match
        // This is expensive, but correct for "checking status".
        // Use a set of running binaries for O(1) lookups per app.
        
        let mut running_binaries = std::collections::HashSet::new();
        
        if let Ok(entries) = fs::read_dir("/proc") {
            for entry in entries.flatten() {
                if let Ok(file_name) = entry.file_name().into_string() {
                    if file_name.parse::<u32>().is_ok() {
                        let comm_path = entry.path().join("comm");
                        if let Ok(comm) = fs::read_to_string(comm_path) {
                            running_binaries.insert(comm.trim().to_string());
                        }
                    }
                }
            }
        }

        for app in apps {
            // This matching is naive. Exec path in .desktop might be full path, 
            // process name might be just binary name.
            // We'll try to match the binary name from the app's exec_path.
            let app_bin_name = Path::new(&app.exec_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&app.exec_path);

            if running_binaries.contains(app_bin_name) {
                app.is_running = true;
            } else {
                app.is_running = false;
            }
        }
    }
}

use std::path::Path;
