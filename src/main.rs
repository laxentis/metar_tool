use eframe::{Frame, Storage};
use egui::{Ui};

pub mod metar;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
struct App {
    icao: String,
    wind_dir: f32,
    wind_left: f32,
    wind_right: f32,
    wind_vrb: bool,
    wind_varying: bool,
    wind_speed: f32,
    wind_max_speed: f32,
    visibility: f32,
}

impl Default for App {
    fn default() -> Self {
        Self{
            icao: "EPBC".to_owned(),
            wind_dir: 280.0,
            wind_left: 90.0,
            wind_right: 180.0,
            wind_vrb: false,
            wind_varying: false,
            wind_speed: 0.0,
            wind_max_speed: 0.0,
            visibility: 0.0,
        }
    }
}

impl App {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = _cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut Frame) {
        egui::Panel::top("top_panel").show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ui.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ui, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("METAR Tool");

            ui.horizontal(|ui| {
                ui.label("ICAO: ");
                ui.text_edit_singleline(&mut self.icao);
            });

            ui.label("Wind:");
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.wind_vrb, "VRB");
                ui.add_enabled(!self.wind_vrb, egui::Slider::new(&mut self.wind_dir, 0.0..=359.0).text("°"));
                ui.checkbox(&mut self.wind_varying, "V");
                ui.add_enabled(self.wind_varying, egui::RangeSlider::new(&mut self.wind_left, &mut self.wind_right, 0.0..=359.0).text("°"));
            });
            ui.horizontal(|ui| {
                ui.label("Speed:");
                ui.add(egui::Slider::new(&mut self.wind_speed, 0.0..=100.0).text("kts"));
                ui.label("Max:");
                ui.add(egui::Slider::new(&mut self.wind_max_speed, 0.0..=100.0).text("kts"));
            });

            ui.label("Visibility:");
            ui.add(egui::Slider::new(&mut self.visibility, 0.0..=10000.0).text("m"));

            ui.separator();

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::warn_if_debug_build(ui);
            });
        });
    }

    fn save(&mut self, storage: &mut dyn Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}

fn main() -> eframe::Result {

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0]),
        ..Default::default()
    };
    eframe::run_native(
        "METAR Tool",
        native_options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}
