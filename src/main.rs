use crate::metar::ObservationMessage;
use eframe::{Frame, Storage};
use egui::Ui;

pub mod metar;
mod reminder;

use reminder::{ReminderConfig, ReminderService};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
struct App {
    hour: u32,
    minute: u32,
    wind_dir: i32,
    wind_left: i32,
    wind_right: i32,
    wind_vrb: bool,
    wind_varying: bool,
    wind_gusting: bool,
    wind_max_speed: i32,
    visibility: f32,
    metar: ObservationMessage,
    reminder_config: ReminderConfig,
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    reminder: ReminderService,
}

impl Default for App {
    fn default() -> Self {
        let config = ReminderConfig::default();
        Self {
            hour: 6,
            minute: 0,
            wind_dir: 280,
            wind_left: 90,
            wind_right: 180,
            wind_vrb: false,
            wind_varying: false,
            wind_gusting: false,
            wind_max_speed: 0,
            visibility: 0.0,
            metar: ObservationMessage::default(),
            reminder: ReminderService::start(config),
            reminder_config: config,
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
                ui.text_edit_singleline(&mut self.metar.location);
            });
            ui.horizontal(|ui| {
                ui.label("Day:");
                ui.add(egui::DragValue::new(&mut self.metar.observation_day).range(1..=31));
            });
            ui.horizontal(|ui| {
                ui.label("Time:");
                ui.add(egui::DragValue::new(&mut self.hour).range(0..=23));
                ui.label(":");
                ui.add(egui::DragValue::new(&mut self.minute).range(0..=59));
            });

            self.metar.observation_time = self.hour * 100 + self.minute;

            ui.label("Wind:");
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.wind_vrb, "VRB");
                ui.add_enabled(
                    !self.wind_vrb,
                    egui::Slider::new(&mut self.wind_dir, 0..=350)
                        .text("°")
                        .step_by(10.0),
                );
                ui.checkbox(&mut self.wind_varying, "V");
                ui.add_enabled(
                    self.wind_varying,
                    egui::RangeSlider::new(&mut self.wind_left, &mut self.wind_right, 0..=350)
                        .step_by(10.0)
                        .text("°"),
                );
            });
            if self.wind_vrb {
                self.metar.wind.direction = None;
            } else {
                self.metar.wind.direction = Some(self.wind_dir);
            }
            if self.wind_varying {
                self.metar.wind.variance =
                    Some(metar::WindVariance(self.wind_left, self.wind_right));
            } else {
                self.metar.wind.variance = None;
            }
            ui.horizontal(|ui| {
                ui.label("Speed:");
                ui.add(egui::Slider::new(&mut self.metar.wind.speed, 0..=100).text("kts"));
                ui.checkbox(&mut self.wind_gusting, "G");
                ui.add_enabled(
                    self.wind_gusting,
                    egui::Slider::new(&mut self.wind_max_speed, 0..=100).text("kts"),
                );
            });

            if self.wind_gusting {
                self.metar.wind.gust_speed = Some(self.wind_max_speed);
            } else {
                self.metar.wind.gust_speed = None;
            }

            ui.horizontal(|ui| {
                ui.label("Visibility:");
                ui.add(
                    egui::Slider::new(
                        &mut self
                            .metar
                            .visibility_phenomena_clouds_group
                            .visibility
                            .visibility,
                        0..=10000,
                    )
                    .text("m"),
                );
            });

            ui.horizontal(|ui| {
                ui.label("Temperature:");
                ui.add(egui::Slider::new(&mut self.metar.temperature, -50..=50).text("°C"));
                ui.label("Dew Point:");
                ui.add(egui::Slider::new(&mut self.metar.dew_point, -50..=50).text("°C"));
            });
            ui.horizontal(|ui| {
                ui.label("QNH:");
                ui.add(
                    egui::Slider::new(&mut self.metar.pressure.value, 800.0..=1200.0)
                        .text("hPa")
                        .step_by(1.0),
                );
            });

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("📋").clicked() {
                    ui.ctx().copy_text(self.metar.to_string());
                };
                ui.add_space(10.0);
                ui.label(self.metar.to_string());
            });
            ui.separator();

            ui.horizontal(|ui| {
                let mut changed = false;

                changed |= ui
                    .checkbox(&mut self.reminder_config.enabled, "🔊")
                    .changed();
                changed |= ui
                    .add(egui::RangeSlider::new(
                        &mut self.reminder_config.start_hour,
                        &mut self.reminder_config.end_hour,
                        0..=23,
                    ))
                    .changed();
                if changed {
                    self.reminder.update_config(self.reminder_config);
                }
                if ui.button("Test sound").clicked() {
                    self.reminder.test_sound();
                }
            });

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
