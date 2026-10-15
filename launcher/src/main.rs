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

        match run_game(&systemless, &path) {
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
fn run_game(systemless: &Path, game: &Path) -> Result<(), String> {
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
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(stderr);
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
