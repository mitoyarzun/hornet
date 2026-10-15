//! Double-click launcher for F/A-18 Hornet 2.0 on Systemless.
//!
//! The launcher remembers the game file, asks for it with a native file
//! dialog when it does not know it, and starts the systemless binary that is
//! beside the launcher. Nobody needs a terminal.

// No console window on Windows.
#![windows_subsystem = "windows"]

use rfd::{FileDialog, MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};

const TITLE: &str = "F/A-18 Hornet 2.0";

fn main() {
    let Some(systemless) = systemless_path() else {
        error(&format!(
            "The systemless program is missing. Put it in the same folder as this launcher:\n\n{}",
            launcher_dir().display()
        ));
        return;
    };

    // A file dropped on the launcher icon (Windows) replaces the saved file.
    let mut game = std::env::args_os().nth(1).map(PathBuf::from);
    if let Some(dropped) = &game {
        save_game_path(dropped);
    }

    // Ask for the display settings at the first start, or when Shift is held
    // down while the launcher starts.
    let mut settings = load_settings();
    let mut ask_settings = settings.is_none() || shift_key_down();

    loop {
        let path = match game.take().or_else(saved_game_path) {
            Some(path) => path,
            None => match choose_game_file() {
                Some(path) => {
                    save_game_path(&path);
                    path
                }
                None => return,
            },
        };

        if ask_settings {
            let chosen = choose_settings(settings.unwrap_or_default());
            save_settings(&chosen);
            settings = Some(chosen);
            ask_settings = false;
        }

        match run_game(&systemless, &path, settings.unwrap_or_default()) {
            Ok(()) => return,
            Err(message) => {
                let answer = MessageDialog::new()
                    .set_level(MessageLevel::Error)
                    .set_title(TITLE)
                    .set_description(format!(
                        "{message}\n\nDo you want to choose a different game file?"
                    ))
                    .set_buttons(MessageButtons::YesNo)
                    .show();
                if answer != MessageDialogResult::Yes {
                    return;
                }
                forget_game_path();
            }
        }
    }
}

/// Start the game and wait for it. Return an error message if it fails.
fn run_game(systemless: &Path, game: &Path, settings: Settings) -> Result<(), String> {
    if !game.is_file() {
        return Err(format!("The game file is missing:\n{}", game.display()));
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

    let mut child = command
        .spawn()
        .map_err(|err| format!("The game did not start: {err}"))?;
    let status = wait_for_game(&mut child)
        .map_err(|err| format!("The launcher lost contact with the game: {err}"))?;
    if status.success() {
        return Ok(());
    }
    let mut message = format!("The game stopped with an error ({status}).");
    if let Some(path) = log_path {
        message.push_str(&format!("\n\nThe log is in:\n{}", path.display()));
    }
    Err(message)
}

/// The dialogs make the launcher an AppKit application. If it then blocks,
/// macOS marks it as not responding and shows the beachball over the game.
/// Remove the launcher from the Dock and keep it answering events until the
/// game stops.
#[cfg(target_os = "macos")]
fn wait_for_game(child: &mut Child) -> std::io::Result<ExitStatus> {
    use objc2::MainThreadMarker;
    use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy, NSEventMask};
    use objc2_foundation::{NSDate, NSDefaultRunLoopMode};

    let Some(mtm) = MainThreadMarker::new() else {
        return child.wait();
    };
    let app = NSApplication::sharedApplication(mtm);
    app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        let until = NSDate::dateWithTimeIntervalSinceNow(0.1);
        let event = unsafe {
            app.nextEventMatchingMask_untilDate_inMode_dequeue(
                NSEventMask::Any,
                Some(&until),
                NSDefaultRunLoopMode,
                true,
            )
        };
        if let Some(event) = event {
            app.sendEvent(&event);
        }
    };
    // An error dialog can follow, so put the launcher back in the Dock.
    app.setActivationPolicy(NSApplicationActivationPolicy::Regular);
    Ok(status)
}

#[cfg(not(target_os = "macos"))]
fn wait_for_game(child: &mut Child) -> std::io::Result<ExitStatus> {
    child.wait()
}

/// The size of the emulated Mac screen, and the window mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Settings {
    width: u16,
    height: u16,
    fullscreen: bool,
    /// Draw the game's menu bar inside the game window, as on a real Mac.
    /// The other choice (macOS only) moves the menus to the macOS menu bar.
    menu_bar_in_window: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            fullscreen: false,
            menu_bar_in_window: true,
        }
    }
}

/// Mac screen sizes that the game supports. Its largest view is 1152 pixels
/// wide.
const SCREEN_SIZES: [(u16, u16); 3] = [(800, 600), (1024, 768), (1152, 870)];

fn size_label((width, height): (u16, u16)) -> String {
    format!("{width} × {height}")
}

fn choose_settings(current: Settings) -> Settings {
    let labels = SCREEN_SIZES.map(size_label);
    let answer = MessageDialog::new()
        .set_level(MessageLevel::Info)
        .set_title(TITLE)
        .set_description(format!(
            "Choose the size of the game screen. A larger screen shows more of the game.\n\n\
             Now: {}. To change this later, hold down Shift when you start the game.",
            size_label((current.width, current.height))
        ))
        .set_buttons(MessageButtons::YesNoCancelCustom(
            labels[0].clone(),
            labels[1].clone(),
            labels[2].clone(),
        ))
        .show();
    let (width, height) = match answer {
        MessageDialogResult::Custom(label) => labels
            .iter()
            .position(|candidate| *candidate == label)
            .map_or((current.width, current.height), |index| SCREEN_SIZES[index]),
        _ => (current.width, current.height),
    };

    let answer = MessageDialog::new()
        .set_level(MessageLevel::Info)
        .set_title(TITLE)
        .set_description("Play in a window or in full screen?")
        .set_buttons(MessageButtons::OkCancelCustom(
            "Full screen".to_string(),
            "Window".to_string(),
        ))
        .show();
    let fullscreen = match answer {
        MessageDialogResult::Custom(label) => label == "Full screen",
        _ => current.fullscreen,
    };

    Settings {
        width,
        height,
        fullscreen,
        menu_bar_in_window: choose_menu_bar(current.menu_bar_in_window),
    }
}

/// Only macOS can move the game menus to the system menu bar.
#[cfg(target_os = "macos")]
fn choose_menu_bar(current: bool) -> bool {
    let answer = MessageDialog::new()
        .set_level(MessageLevel::Info)
        .set_title(TITLE)
        .set_description(
            "Where do you want the game's menu bar?\n\n\
             In the game window, it looks and works as on a real Mac. In the macOS \
             menu bar, the Dock shows the game's name and icon.",
        )
        .set_buttons(MessageButtons::OkCancelCustom(
            "In the game window".to_string(),
            "In the macOS menu bar".to_string(),
        ))
        .show();
    match answer {
        MessageDialogResult::Custom(label) => label == "In the game window",
        _ => current,
    }
}

#[cfg(not(target_os = "macos"))]
fn choose_menu_bar(current: bool) -> bool {
    current
}

fn parse_settings(text: &str) -> Settings {
    let mut settings = Settings::default();
    for line in text.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key.trim() {
            "screen" => {
                if let Some(size) = value.trim().split_once('x').and_then(|(w, h)| {
                    Some((w.parse().ok()?, h.parse().ok()?))
                }) {
                    if SCREEN_SIZES.contains(&size) {
                        (settings.width, settings.height) = size;
                    }
                }
            }
            "fullscreen" => settings.fullscreen = value.trim() == "yes",
            "menu_bar" => settings.menu_bar_in_window = value.trim() != "macos",
            _ => {}
        }
    }
    settings
}

fn load_settings() -> Option<Settings> {
    let text = fs::read_to_string(state_dir()?.join("settings.txt")).ok()?;
    Some(parse_settings(&text))
}

fn save_settings(settings: &Settings) {
    if let Some(dir) = state_dir() {
        let text = format!(
            "screen={}x{}\nfullscreen={}\nmenu_bar={}\n",
            settings.width,
            settings.height,
            if settings.fullscreen { "yes" } else { "no" },
            if settings.menu_bar_in_window { "window" } else { "macos" }
        );
        let _ = fs::write(dir.join("settings.txt"), text);
    }
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

fn choose_game_file() -> Option<PathBuf> {
    MessageDialog::new()
        .set_level(MessageLevel::Info)
        .set_title(TITLE)
        .set_description(
            "Select your copy of F/A-18 Hornet 2.0.\n\n\
             Use a StuffIt archive (.sit), a MacBinary file (.bin), or a disk image \
             of the game disc (.toast, .iso, .img, .dsk). The launcher remembers \
             the file for the next start.",
        )
        .set_buttons(MessageButtons::Ok)
        .show();
    FileDialog::new()
        .set_title("Select your copy of F/A-18 Hornet 2.0")
        .add_filter(
            "Game archives and disk images",
            &["sit", "bin", "toast", "iso", "img", "dsk", "image", "hfv", "cdr"],
        )
        .add_filter("All files", &["*"])
        .pick_file()
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

fn state_dir() -> Option<PathBuf> {
    let dir = dirs::config_dir()?.join("hornet");
    fs::create_dir_all(&dir).ok()?;
    Some(dir)
}

fn saved_game_path() -> Option<PathBuf> {
    let text = fs::read_to_string(state_dir()?.join("game-path.txt")).ok()?;
    let path = PathBuf::from(text.trim_end_matches(['\r', '\n']));
    path.is_file().then_some(path)
}

fn save_game_path(path: &Path) {
    if let Some(dir) = state_dir() {
        let _ = fs::write(dir.join("game-path.txt"), path.to_string_lossy().as_bytes());
    }
}

fn forget_game_path() {
    if let Some(dir) = state_dir() {
        let _ = fs::remove_file(dir.join("game-path.txt"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_round_trip_through_the_file_format() {
        let settings = parse_settings("screen=1024x768\nfullscreen=yes\nmenu_bar=macos\n");
        assert_eq!(
            settings,
            Settings {
                width: 1024,
                height: 768,
                fullscreen: true,
                menu_bar_in_window: false,
            }
        );
        // Settings files from before the menu bar choice keep the default.
        assert!(parse_settings("screen=800x600\nfullscreen=no\n").menu_bar_in_window);
    }

    #[test]
    fn unknown_screen_sizes_keep_the_default() {
        assert_eq!(parse_settings("screen=4000x3000\n"), Settings::default());
        assert_eq!(parse_settings("garbage"), Settings::default());
    }
}
