use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub const APP_ID: &str = "dev.agzes.antiafk-rbx-sober";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppState {
    pub running: bool,
    pub jump: bool,
    pub walk: bool,
    pub spin_jiggle: bool,
    pub interval_seq: u64,
    pub mode: usize,

    pub auto_start: bool,
    pub multi_instance: bool,
    pub hides_game: bool,
    pub user_safe: bool,
    pub auto_reconnect: bool,
    pub fps_capper: bool,
    pub fps_limit: u32,
    pub stop_limit_on_focus: bool,
    pub stealth: bool,
    pub niri_pin: bool,
    pub last_run_version: Option<String>,
    pub shown_warning: bool,
    #[serde(skip)]
    pub manually_stopped: bool,
    #[serde(skip)]
    pub action_active: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            running: false,
            jump: true,
            walk: false,
            spin_jiggle: false,
            interval_seq: 60,
            mode: 0,
            auto_start: false,
            multi_instance: false,
            hides_game: false,
            user_safe: false,
            auto_reconnect: false,
            fps_capper: false,
            fps_limit: 30,
            stop_limit_on_focus: false,
            stealth: false,
            niri_pin: false,
            last_run_version: None,
            shown_warning: false,
            manually_stopped: false,
            action_active: false,
        }
    }
}

pub type SharedState = Arc<Mutex<AppState>>;

impl AppState {
    fn get_config_path() -> Option<PathBuf> {
        let mut path = dirs::config_dir()?;
        path.push("antiafk-rbx-sober");
        if !path.exists() {
            let _ = fs::create_dir_all(&path);
        }
        path.push("config.json");
        Some(path)
    }

    pub fn load() -> Self {
        let mut state = if let Some(path) = Self::get_config_path()
            && let Ok(data) = fs::read_to_string(path)
            && let Ok(mut state) = serde_json::from_str::<AppState>(&data)
        {
            state.running = false;
            state
        } else {
            Self::default()
        };

        let detected = Self::detect_de_mode();
        state.mode = detected;

        state
    }

    fn detect_de_mode() -> usize {
        if Self::is_hyprland() {
            0
        } else if Self::is_plasma() {
            1
        } else if Self::is_niri() {
            2
        } else {
            3
        }
    }

    pub fn is_hyprland() -> bool {
        std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok()
    }

    pub fn is_plasma() -> bool {
        std::env::var("XDG_CURRENT_DESKTOP").map_or(false, |v| {
            let v = v.to_uppercase();
            v.contains("KDE") || v.contains("PLASMA")
        }) || std::env::var("KDE_FULL_SESSION").is_ok()
    }

    pub fn is_niri() -> bool {
        std::env::var("NIRI_SOCKET").is_ok() || std::env::var("XDG_CURRENT_DESKTOP").map_or(false, |v| {
            v.to_lowercase() == "niri"
        })
    }

    pub fn save(&self) {
        if let Some(path) = Self::get_config_path() {
            let mut state_to_save = self.clone();
            state_to_save.running = false;
            if let Ok(data) = serde_json::to_string_pretty(&state_to_save) {
                let _ = fs::write(path, data);
            }
        }
    }

    const NIRI_RULE_FILE: &str = "antiafk-rbx-sober.kdl";

    fn get_niri_rules_dir() -> Option<PathBuf> {
        let mut path = dirs::config_dir()?;
        path.push("niri");
        path.push("config.d");
        Some(path)
    }

    pub fn apply_niri_rule(enable: bool) {
        let Some(dir) = Self::get_niri_rules_dir() else { return };
        let _ = fs::create_dir_all(&dir);
        let file_path = dir.join(Self::NIRI_RULE_FILE);
        if enable {
            let content = format!(
                "window-rule {{\n    match app-id=\"{APP_ID}\"\n    open-floating false\n    max-width 467\n    max-height 690\n    min-width 467\n    min-height 690\n}}\n"
            );
            let _ = fs::write(&file_path, content);
        } else {
            let _ = fs::remove_file(&file_path);
        }
    }

}
