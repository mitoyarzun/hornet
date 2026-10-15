//! The settings window: the game file, the screen size, the display mode
//! and the menu bar, with a Play button.
//!
//! The launcher runs this window in a second copy of itself (`--settings`).
//! On macOS, a process can create its window event loop only one time, and
//! the launcher must be able to show the window again after an error.

use crate::settings::{
    load_settings, save_game_path, save_settings, saved_game_path, size_label, Settings,
    SCREEN_SIZES,
};
use eframe::egui;
use std::cell::Cell;
use std::path::PathBuf;
use std::rc::Rc;

/// The exit status of the settings process when the user clicks Play.
pub const PLAY: i32 = 0;
/// The exit status when the user closes the window or clicks Quit.
pub const QUIT: i32 = 1;

struct SettingsWindow {
    game: Option<PathBuf>,
    settings: Settings,
    message: Option<String>,
    play: Rc<Cell<bool>>,
}

/// Show the settings window. Return true if the user clicked Play. The
/// window saves the game file and the settings before it closes.
pub fn run(message: Option<String>) -> bool {
    let play = Rc::new(Cell::new(false));
    let app = SettingsWindow {
        game: saved_game_path(),
        settings: load_settings().unwrap_or_default(),
        message,
        play: play.clone(),
    };
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(crate::TITLE)
            .with_inner_size([520.0, 400.0])
            .with_resizable(false),
        centered: true,
        ..Default::default()
    };
    let result = eframe::run_native(
        crate::TITLE,
        options,
        Box::new(|_| Ok(Box::new(app))),
    );
    if let Err(error) = result {
        crate::error(&format!("The settings window did not open: {error}"));
        return false;
    }
    play.get()
}

impl eframe::App for SettingsWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.spacing_mut().item_spacing.y = 8.0;
            ui.heading(crate::TITLE);
            if let Some(message) = &self.message {
                ui.colored_label(egui::Color32::from_rgb(200, 40, 40), message);
            }
            ui.add_space(4.0);

            egui::Grid::new("settings")
                .num_columns(2)
                .spacing([16.0, 10.0])
                .show(ui, |ui| {
                    ui.label("Game file");
                    ui.horizontal(|ui| {
                        if ui.button("Choose…").clicked() {
                            if let Some(path) = choose_game_file() {
                                self.game = Some(path);
                            }
                        }
                        match &self.game {
                            Some(path) => ui.label(file_name(path)).on_hover_text(path.display().to_string()),
                            None => ui.weak("No file selected"),
                        };
                    });
                    ui.end_row();

                    ui.label("Game screen");
                    ui.horizontal(|ui| {
                        for size in SCREEN_SIZES {
                            let selected = (self.settings.width, self.settings.height) == size;
                            if ui.radio(selected, size_label(size)).clicked() {
                                (self.settings.width, self.settings.height) = size;
                            }
                        }
                    });
                    ui.end_row();

                    ui.label("Display");
                    ui.horizontal(|ui| {
                        ui.radio_value(&mut self.settings.fullscreen, false, "Window");
                        ui.radio_value(&mut self.settings.fullscreen, true, "Full screen");
                    });
                    ui.end_row();

                    if cfg!(target_os = "macos") {
                        ui.label("Menu bar");
                        ui.horizontal(|ui| {
                            ui.radio_value(
                                &mut self.settings.menu_bar_in_window,
                                true,
                                "In the game window",
                            );
                            ui.radio_value(
                                &mut self.settings.menu_bar_in_window,
                                false,
                                "In the macOS menu bar",
                            );
                        });
                        ui.end_row();
                    }
                });

            ui.add_space(4.0);
            ui.weak(
                "A larger game screen shows more of the game. You can also resize \
                 the game window at any time.",
            );
            if cfg!(target_os = "macos") {
                ui.weak(
                    "With the menu bar in the game window, the menus work as on a real \
                     Mac. In the macOS menu bar, the Dock shows the game's name and icon.",
                );
            }
            ui.checkbox(
                &mut self.settings.show_window,
                "Show this window every time the game starts",
            );
            ui.weak("To show this window again, hold down Shift when you start the game.");

            ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
                ui.horizontal(|ui| {
                    let play = ui.add_enabled(
                        self.game.is_some(),
                        egui::Button::new("Play").min_size(egui::vec2(90.0, 28.0)),
                    );
                    if play.clicked() {
                        if let Some(game) = &self.game {
                            save_game_path(game);
                        }
                        save_settings(&self.settings);
                        self.play.set(true);
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    if ui
                        .add(egui::Button::new("Quit").min_size(egui::vec2(90.0, 28.0)))
                        .clicked()
                    {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            });
        });
    }
}

fn file_name(path: &std::path::Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string())
}

fn choose_game_file() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title("Select your copy of F/A-18 Hornet 2.0")
        .add_filter(
            "Game archives and disk images",
            &["sit", "bin", "toast", "iso", "img", "dsk", "image", "hfv", "cdr"],
        )
        .add_filter("All files", &["*"])
        .pick_file()
}
