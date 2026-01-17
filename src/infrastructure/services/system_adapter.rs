use std::process::Command;

pub struct SystemAdapter;

impl SystemAdapter {
    pub fn new() -> Self {
        Self
    }

    fn run_cmd(cmd: &str, args: &[&str]) {
        let _ = Command::new(cmd).args(args).spawn();
    }

    pub fn shutdown(&self) {
        Self::run_cmd("systemctl", &["poweroff"]);
    }

    pub fn reboot(&self) {
        Self::run_cmd("systemctl", &["reboot"]);
    }

    pub fn suspend(&self) {
        Self::run_cmd("systemctl", &["suspend"]);
    }

    pub fn hibernate(&self) {
        Self::run_cmd("systemctl", &["hibernate"]);
    }

    pub fn lock(&self) {
        // Try loginctl first, commonplace on systemd systems
        Self::run_cmd("loginctl", &["lock-session"]);
    }

    pub fn mute_audio(&self) {
        // Toggle default sink mute
        Self::run_cmd("wpctl", &["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"]);
    }

    pub fn mute_mic(&self) {
        // Toggle default source mute
        Self::run_cmd("wpctl", &["set-mute", "@DEFAULT_AUDIO_SOURCE@", "toggle"]);
    }

    pub fn mute_all(&self) {
        self.mute_audio();
        self.mute_mic();
    }

    pub fn toggle_night_light(&self) {
        // We need to check current state to toggle? Or is there a toggle?
        // gsettings doesn't have a toggle. We need to read then write.
        // For simplicity in this "fire and forget" context, we might implement a small script logic
        // or just spawn a shell command that does the toggle.
        let script = "
            schema='org.gnome.settings-daemon.plugins.color'
            key='night-light-enabled'
            current=$(gsettings get $schema $key)
            if [ \"$current\" = \"true\" ]; then
                gsettings set $schema $key false
            else
                gsettings set $schema $key true
            fi
        ";
        Self::run_cmd("sh", &["-c", script]);
    }

    pub fn toggle_dark_mode(&self) {
        let script = "
            schema='org.gnome.desktop.interface'
            key='color-scheme'
            current=$(gsettings get $schema $key)
            if [ \"$current\" = \"'prefer-dark'\" ]; then
                gsettings set $schema $key 'default'
            else
                gsettings set $schema $key 'prefer-dark'
            fi
        ";
        Self::run_cmd("sh", &["-c", script]);
    }

    pub fn toggle_dnd(&self) {
        // 'show-banners' -> false means DND is ON (notifications hidden)
        // 'show-banners' -> true means DND is OFF (notifications shown)
        let script = "
            schema='org.gnome.desktop.notifications'
            key='show-banners'
            current=$(gsettings get $schema $key)
            if [ \"$current\" = \"true\" ]; then
                gsettings set $schema $key false
            else
                gsettings set $schema $key true
            fi
        ";
        Self::run_cmd("sh", &["-c", script]);
    }
}
use crate::domain::ports::ISystemPower;

impl ISystemPower for SystemAdapter {
    fn execute(&self, action: &str) -> Result<(), String> {
        match action {
            "shutdown" | "poweroff" => { self.shutdown(); Ok(()) },
            "reboot" => { self.reboot(); Ok(()) },
            "suspend" => { self.suspend(); Ok(()) },
            "hibernate" => { self.hibernate(); Ok(()) },
            "lock" => { self.lock(); Ok(()) },
            "mute" | "mute_audio" => { self.mute_audio(); Ok(()) },
            "mute_mic" => { self.mute_mic(); Ok(()) },
            "mute_all" => { self.mute_all(); Ok(()) },
            "toggle_night_light" => { self.toggle_night_light(); Ok(()) },
            "toggle_dark_mode" => { self.toggle_dark_mode(); Ok(()) },
            "toggle_dnd" => { self.toggle_dnd(); Ok(()) },
            _ => Err(format!("Unknown system action: {}", action))
        }
    }
}
