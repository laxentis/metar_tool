#![windows_subsystem = "windows"]

mod app;
mod i18n;
pub mod metar;
mod reminder;
mod server;

use egui::WindowLevel;
use crate::app::App;

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([600.0, 500.0])
            .with_window_level(WindowLevel::AlwaysOnTop),
        ..Default::default()
    };
    eframe::run_native(
        "METAR Tool",
        native_options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}
