mod app;
pub mod metar;
mod reminder;
mod server;

use crate::app::App;

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([600.0, 500.0]),
        ..Default::default()
    };
    eframe::run_native(
        "METAR Tool",
        native_options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}
