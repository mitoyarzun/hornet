//! Double-click launcher for F/A-18 Hornet 2.0 on Systemless.
//!
//! The launcher remembers the game file and the display settings, shows a
//! settings window when it needs them, and starts the systemless binary that
//! is beside the launcher. Nobody needs a terminal.

// No console window on Windows.
#![windows_subsystem = "windows"]

mod settings;
mod window;

use rfd::{MessageButtons, MessageDialog, MessageLevel};
use settings::{load_settings, save_game_path, saved_game_path, state_dir, Settings};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const TITLE: &str = "F/A-18 Hornet 2.0";
const SETTINGS_ARG: &str = "--settings";

fn main() {
    let mut args = std::env::args_os().skip(1);
    let first = args.next();

    // The second copy of the launcher shows the settings window.
    if first.as_deref() == Some(SETTINGS_ARG.as_ref()) {
        let message = args.next().map(|text| text.to_string_lossy().into_owned());
        let status = if window::run(message) {
            window::PLAY
        } else {
            window::QUIT
        };
        std::process::exit(status);
    }

    let Some(systemless) = systemless_path() else {
        error(&format!(
            "The systemless program is missing. Put it in the same folder as this launcher:\n\n{}",
            launcher_dir().display()
        ));
        return;
    };

    // A file dropped on the launcher icon (Windows) replaces the saved file.
    if let Some(dropped) = &first {
        save_game_path(Path::new(dropped));
    }

    // Show the settings window at the first start, when the user asks for it
    // at every start, or when Shift is held down while the launcher starts.
    let mut show_window = match load_settings() {
        None => true,
        Some(settings) => settings.show_window || shift_key_down(),
    };
    let mut message = None;

    loop {
        if show_window && !open_settings_window(message.take()) {
            return;
        }
        let Some(game) = saved_game_path() else {
            show_window = true;
            message = Some("Select your copy of the game.".to_string());
            continue;
        };
        match run_game(&systemless, &game, load_settings().unwrap_or_default()) {
            Ok(()) => return,
            Err(error) => {
                show_window = true;
                message = Some(error);
            }
        }
    }
}

/// Run the settings window in a second copy of the launcher. Return true if
/// the user clicked Play.
fn open_settings_window(message: Option<String>) -> bool {
    let Ok(launcher) = std::env::current_exe() else {
        return false;
    };
    let mut command = Command::new(launcher);
    command.arg(SETTINGS_ARG);
    if let Some(message) = message {
        command.arg(message);
    }
    match command.status() {
        Ok(status) => status.code() == Some(window::PLAY),
        Err(err) => {
            error(&format!("The settings window did not open: {err}"));
            false
        }
    }
}

/// Start the game and wait for it. Return an error message if it fails.
fn run_game(systemless: &Path, game: &Path, settings: Settings) -> Result<(), String> {
    if !game.is_file() {
        return Err(format!("The game file is missing: {}", game.display()));
    }
    let log_path = state_dir().map(|dir| dir.join("last-run.log"));
    let stderr = log_path
        .as_ref()
        .and_then(|path| fs::File::create(path).ok())
        .map(Stdio::from)
        .unwrap_or_else(Stdio::null);

    let mut command = Command::new(systemless);
    command
        .arg("--prefer-68k")
        .arg(game)
        .env("SYSTEMLESS_LOAD_EXECUTABLE", "Hornet")
        .env("SYSTEMLESS_SCREEN_WIDTH", settings.width.to_string())
        .env("SYSTEMLESS_SCREEN_HEIGHT", settings.height.to_string())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(stderr);
    if settings.fullscreen {
        command.arg("--fullscreen");
    }
    // Without the native integrations, Systemless draws the classic menu bar
    // in the window and sends Command-key shortcuts to the game as keys.
    if settings.menu_bar_in_window && cfg!(target_os = "macos") {
        command.arg("--no-native-integrations");
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }

    // This process shows no window before the game starts, so it can block
    // here. An AppKit application that blocks gets the beachball.
    let status = command
        .status()
        .map_err(|err| format!("The game did not start: {err}"))?;
    if status.success() {
        return Ok(());
    }
    let mut message = format!("The game stopped with an error ({status}).");
    if let Some(path) = log_path {
        message.push_str(&format!(" The log is in {}.", path.display()));
    }
    Err(message)
}

#[cfg(target_os = "macos")]
fn shift_key_down() -> bool {
    use objc2_app_kit::{NSEvent, NSEventModifierFlags};
    NSEvent::modifierFlags_class().contains(NSEventModifierFlags::Shift)
}

#[cfg(windows)]
fn shift_key_down() -> bool {
    #[link(name = "user32")]
    extern "system" {
        fn GetAsyncKeyState(key: i32) -> i16;
    }
    const VK_SHIFT: i32 = 0x10;
    // The high bit is set while the key is down.
    unsafe { GetAsyncKeyState(VK_SHIFT) < 0 }
}

#[cfg(not(any(target_os = "macos", windows)))]
fn shift_key_down() -> bool {
    false
}

fn error(message: &str) {
    MessageDialog::new()
        .set_level(MessageLevel::Error)
        .set_title(TITLE)
        .set_description(message)
        .set_buttons(MessageButtons::Ok)
        .show();
}

fn launcher_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(Path::to_path_buf))
        .unwrap_or_default()
}

fn systemless_path() -> Option<PathBuf> {
    let name = if cfg!(windows) { "systemless.exe" } else { "systemless" };
    let path = launcher_dir().join(name);
    path.is_file().then_some(path)
}
