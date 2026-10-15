//! The saved launcher state: the game file and the display settings.
//!
//! Both live in the user's configuration folder, in `hornet/`.

use std::fs;
use std::path::{Path, PathBuf};

/// The size of the emulated Mac screen, the window mode, and the menu bar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Settings {
    pub width: u16,
    pub height: u16,
    pub fullscreen: bool,
    /// Draw the game's menu bar inside the game window, as on a real Mac.
    /// The other choice (macOS only) moves the menus to the macOS menu bar.
    pub menu_bar_in_window: bool,
    /// Show the settings window at every start, not only at the first one.
    pub show_window: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            fullscreen: false,
            menu_bar_in_window: true,
            show_window: false,
        }
    }
}

/// Mac screen sizes that the game supports. Its largest view is 1152 pixels
/// wide.
pub const SCREEN_SIZES: [(u16, u16); 3] = [(800, 600), (1024, 768), (1152, 870)];

pub fn size_label((width, height): (u16, u16)) -> String {
    format!("{width} × {height}")
}

pub fn parse_settings(text: &str) -> Settings {
    let mut settings = Settings::default();
    for line in text.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let value = value.trim();
        match key.trim() {
            "screen" => {
                if let Some(size) = value
                    .split_once('x')
                    .and_then(|(w, h)| Some((w.parse().ok()?, h.parse().ok()?)))
                {
                    if SCREEN_SIZES.contains(&size) {
                        (settings.width, settings.height) = size;
                    }
                }
            }
            "fullscreen" => settings.fullscreen = value == "yes",
            "menu_bar" => settings.menu_bar_in_window = value != "macos",
            "show_window" => settings.show_window = value == "yes",
            _ => {}
        }
    }
    settings
}

fn format_settings(settings: &Settings) -> String {
    let yes_no = |value: bool| if value { "yes" } else { "no" };
    format!(
        "screen={}x{}\nfullscreen={}\nmenu_bar={}\nshow_window={}\n",
        settings.width,
        settings.height,
        yes_no(settings.fullscreen),
        if settings.menu_bar_in_window { "window" } else { "macos" },
        yes_no(settings.show_window),
    )
}

pub fn state_dir() -> Option<PathBuf> {
    let dir = dirs::config_dir()?.join("hornet");
    fs::create_dir_all(&dir).ok()?;
    Some(dir)
}

pub fn load_settings() -> Option<Settings> {
    let text = fs::read_to_string(state_dir()?.join("settings.txt")).ok()?;
    Some(parse_settings(&text))
}

pub fn save_settings(settings: &Settings) {
    if let Some(dir) = state_dir() {
        let _ = fs::write(dir.join("settings.txt"), format_settings(settings));
    }
}

/// The saved game file, if it still exists.
pub fn saved_game_path() -> Option<PathBuf> {
    let text = fs::read_to_string(state_dir()?.join("game-path.txt")).ok()?;
    let path = PathBuf::from(text.trim_end_matches(['\r', '\n']));
    path.is_file().then_some(path)
}

pub fn save_game_path(path: &Path) {
    if let Some(dir) = state_dir() {
        let _ = fs::write(dir.join("game-path.txt"), path.to_string_lossy().as_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_round_trip_through_the_file_format() {
        let settings = Settings {
            width: 1024,
            height: 768,
            fullscreen: true,
            menu_bar_in_window: false,
            show_window: true,
        };
        assert_eq!(parse_settings(&format_settings(&settings)), settings);
    }

    #[test]
    fn older_settings_files_keep_the_new_defaults() {
        let settings = parse_settings("screen=1152x870\nfullscreen=no\n");
        assert_eq!((settings.width, settings.height), (1152, 870));
        assert!(settings.menu_bar_in_window);
        assert!(!settings.show_window);
    }

    #[test]
    fn unknown_screen_sizes_keep_the_default() {
        assert_eq!(parse_settings("screen=4000x3000\n"), Settings::default());
        assert_eq!(parse_settings("garbage"), Settings::default());
    }
}
