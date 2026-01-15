use crate::interface::ports::ISystemPower;
use std::process::Command;

pub struct LinuxSystemPowerAdapter;

impl LinuxSystemPowerAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl ISystemPower for LinuxSystemPowerAdapter {
    fn execute(&self, action: &str) -> Result<(), String> {
        let cmd = match action {
            "suspend" => "systemctl suspend",
            "reboot" => "systemctl reboot",
            "poweroff" => "systemctl poweroff",
            "hibernate" => "systemctl hibernate",
            // Lock handling might depend on DE, trying generic xdg-screensaver or loginctl
            "lock" => "loginctl lock-session", 
            _ => return Err(format!("Unknown action: {}", action)),
        };

        // Split cmd
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if parts.is_empty() { return Err("Empty command".to_string()); }

        Command::new(parts[0])
            .args(&parts[1..])
            .spawn()
            .map_err(|e| e.to_string())?
            .wait()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
}
