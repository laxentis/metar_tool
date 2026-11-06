use crate::metar::{MessageType, ObservationMessage};
use crate::reminder::{ReminderConfig, ReminderService};
use crate::server::send_to_servers;
use eframe::{Frame, Storage};
use egui::Ui;
use std::sync::{Arc, Mutex};

#[derive(serde::Deserialize, serde::Serialize, Default)]
#[serde(default)]
struct ServerConfig {
    url: String,
    api_key: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct App {
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
    overwrite: bool,
    overwrite_metar: String,
    reminder_config: ReminderConfig,
    sound_settings_window_open: bool,
    server_settings_window_open: bool,
    servers: Vec<ServerConfig>,
    #[serde(skip)]
    server_results: Arc<Mutex<Vec<String>>>,
    cloud_editor: Option<usize>,
    phenomenon_editor: Option<usize>,
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
            sound_settings_window_open: false,
            server_settings_window_open: false,
            servers: Vec::new(),
            server_results: Arc::new(Mutex::new(Vec::new())),
            cloud_editor: None,
            phenomenon_editor: None,
            overwrite: false,
            overwrite_metar: String::new(),
        }
    }
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
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
                    ui.menu_button("Settings", |ui| {
                        if ui.button("Sound").clicked() {
                            self.sound_settings_window_open = true;
                        }
                        if ui.button("Servers").clicked() {
                            self.server_settings_window_open = true;
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

            egui::ComboBox::from_id_salt("message_type")
                .selected_text(self.metar.message_type.to_string())
                .show_ui(ui, |ui| {
                    for message_type in [
                        MessageType::Metar,
                        MessageType::MetarCor,
                        MessageType::Speci,
                        MessageType::SpeciCor,
                    ] {
                        let label = message_type.to_string();
                        ui.selectable_value(&mut self.metar.message_type, message_type, label);
                    }
                });

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
                    egui::DragValue::new(&mut self.wind_dir)
                        .range(0..=350)
                        .suffix("°"),
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
                    Some(crate::metar::WindVariance(self.wind_left, self.wind_right));
            } else {
                self.metar.wind.variance = None;
            }
            ui.horizontal(|ui| {
                ui.label("Speed:");
                ui.add(
                    egui::DragValue::new(&mut self.metar.wind.speed)
                        .suffix(" kts")
                        .range(0..=100),
                );
                ui.checkbox(&mut self.wind_gusting, "G");
                ui.add_enabled(
                    self.wind_gusting,
                    egui::DragValue::new(&mut self.wind_max_speed)
                        .suffix(" kts")
                        .range(0..=100),
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
                ui.label("Clouds:");
                let clouds = &mut self.metar.visibility_phenomena_clouds_group.cloud_layers;
                if let Some(clouds) = clouds {
                    for (index, cloud) in clouds.iter().enumerate() {
                        ui.label(cloud.to_string());
                        if ui.small_button("📝").clicked() {
                            self.cloud_editor = Some(index);
                        }
                        if ui.small_button("🗑️").clicked() {
                            clouds.remove(index);
                            break;
                        }
                    }
                }
                if ui.button("➕").clicked() {
                    let clouds = self
                        .metar
                        .visibility_phenomena_clouds_group
                        .cloud_layers
                        .get_or_insert_with(Vec::new);
                    clouds.push(crate::metar::CloudLayer {
                        coverage: crate::metar::CloudCoverage::FEW,
                        height: 0,
                        cloud_type: None,
                    });
                    self.cloud_editor = Some(clouds.len() - 1);
                }
            });

            ui.horizontal(|ui| {
                ui.label("Phenomenon:");
                let phenomena = &mut self.metar.visibility_phenomena_clouds_group.phenomena;
                if let Some(phenomena) = phenomena {
                    for (index, phenomenon) in phenomena.iter().enumerate() {
                        ui.label(phenomenon.to_string());
                        if ui.small_button("📝").clicked() {
                            self.phenomenon_editor = Some(index);
                        }
                        if ui.small_button("🗑️").clicked() {
                            phenomena.remove(index);
                            break;
                        }
                    }
                }
                if ui.button("➕").clicked() {
                    let items = self
                        .metar
                        .visibility_phenomena_clouds_group
                        .phenomena
                        .get_or_insert_with(Vec::new);
                    items.push(crate::metar::Phenomenon {
                        intensity: None,
                        modifiers: None,
                        code: crate::metar::PhenomenonCode::RA,
                    });
                    self.phenomenon_editor = Some(items.len() - 1);
                }
            });

            if let Some(index) = self.cloud_editor {
                let mut open = true;
                egui::Window::new("Edit cloud layer")
                    .open(&mut open)
                    .show(ui.ctx(), |ui| {
                        if let Some(cloud) = self
                            .metar
                            .visibility_phenomena_clouds_group
                            .cloud_layers
                            .as_mut()
                            .and_then(|v| v.get_mut(index))
                        {
                            ui.horizontal(|ui| {
                                ui.label("Coverage");
                                egui::ComboBox::from_id_salt("cloud_coverage")
                                    .selected_text(cloud.coverage.to_string())
                                    .show_ui(ui, |ui| {
                                        for (i, label) in
                                            ["FEW", "SCT", "BKN", "OVC"].iter().enumerate()
                                        {
                                            if ui
                                                .selectable_label(
                                                    cloud.coverage.to_string() == *label,
                                                    *label,
                                                )
                                                .clicked()
                                            {
                                                cloud.coverage = [
                                                    crate::metar::CloudCoverage::FEW,
                                                    crate::metar::CloudCoverage::SCT,
                                                    crate::metar::CloudCoverage::BKN,
                                                    crate::metar::CloudCoverage::OVC,
                                                ]
                                                .into_iter()
                                                .nth(i)
                                                .unwrap();
                                            }
                                        }
                                    });
                            });
                            ui.horizontal(|ui| {
                                ui.label("Height: ");
                                ui.add(
                                    egui::DragValue::new(&mut cloud.height)
                                        .range(0..=60000)
                                        .speed(100)
                                        .suffix(" ft"),
                                );
                            });

                            let mut kind = cloud
                                .cloud_type
                                .as_ref()
                                .map(ToString::to_string)
                                .unwrap_or_else(|| "None".into());
                            egui::ComboBox::from_id_salt("cloud_type")
                                .selected_text(&kind)
                                .show_ui(ui, |ui| {
                                    for value in ["None", "CB", "TCU"] {
                                        ui.selectable_value(&mut kind, value.to_string(), value);
                                    }
                                });
                            cloud.cloud_type = match kind.as_str() {
                                "CB" => Some(crate::metar::CloudType::CB),
                                "TCU" => Some(crate::metar::CloudType::TCU),
                                _ => None,
                            };
                        }
                    });
                if !open {
                    self.cloud_editor = None;
                }
            }
            if let Some(index) = self.phenomenon_editor {
                let mut open = true;
                egui::Window::new("Edit phenomenon")
                    .open(&mut open)
                    .show(ui.ctx(), |ui| {
                        if let Some(item) = self
                            .metar
                            .visibility_phenomena_clouds_group
                            .phenomena
                            .as_mut()
                            .and_then(|v| v.get_mut(index))
                        {
                            let mut intensity = item
                                .intensity
                                .as_ref()
                                .map(ToString::to_string)
                                .unwrap_or_else(|| "None".into());
                            egui::ComboBox::from_id_salt("phenomenon_intensity")
                                .selected_text(&intensity)
                                .show_ui(ui, |ui| {
                                    for value in ["None", "+", "-"] {
                                        ui.selectable_value(
                                            &mut intensity,
                                            value.to_string(),
                                            value,
                                        );
                                    }
                                });
                            item.intensity = match intensity.as_str() {
                                "+" => Some(crate::metar::PhenomenonIntensity::Plus),
                                "-" => Some(crate::metar::PhenomenonIntensity::Minus),
                                _ => None,
                            };
                            let codes = [
                                "DZ", "RA", "SN", "SG", "PL", "GR", "GS", "UP", "BR", "FG", "FU",
                                "VA", "DU", "SA", "HZ", "PO", "SQ", "FC", "SS", "DS", "TS", "NSW",
                            ];
                            let mut code = item.code.to_string();
                            egui::ComboBox::from_id_salt("phenomenon_code")
                                .selected_text(&code)
                                .show_ui(ui, |ui| {
                                    for value in codes {
                                        ui.selectable_value(&mut code, value.to_string(), value);
                                    }
                                });
                            item.code = match code.as_str() {
                                "DZ" => crate::metar::PhenomenonCode::DZ,
                                "RA" => crate::metar::PhenomenonCode::RA,
                                "SN" => crate::metar::PhenomenonCode::SN,
                                "SG" => crate::metar::PhenomenonCode::SG,
                                "PL" => crate::metar::PhenomenonCode::PL,
                                "GR" => crate::metar::PhenomenonCode::GR,
                                "GS" => crate::metar::PhenomenonCode::GS,
                                "UP" => crate::metar::PhenomenonCode::UP,
                                "BR" => crate::metar::PhenomenonCode::BR,
                                "FG" => crate::metar::PhenomenonCode::FG,
                                "FU" => crate::metar::PhenomenonCode::FU,
                                "VA" => crate::metar::PhenomenonCode::VA,
                                "DU" => crate::metar::PhenomenonCode::DU,
                                "SA" => crate::metar::PhenomenonCode::SA,
                                "HZ" => crate::metar::PhenomenonCode::HZ,
                                "PO" => crate::metar::PhenomenonCode::PO,
                                "SQ" => crate::metar::PhenomenonCode::SQ,
                                "FC" => crate::metar::PhenomenonCode::FC,
                                "SS" => crate::metar::PhenomenonCode::SS,
                                "DS" => crate::metar::PhenomenonCode::DS,
                                "TS" => crate::metar::PhenomenonCode::TS,
                                _ => crate::metar::PhenomenonCode::NSW,
                            };
                            let mut modifier = item
                                .modifiers
                                .as_ref()
                                .and_then(|m| m.first())
                                .map(ToString::to_string)
                                .unwrap_or_else(|| "None".into());
                            egui::ComboBox::from_id_salt("phenomenon_modifier")
                                .selected_text(&modifier)
                                .show_ui(ui, |ui| {
                                    for value in [
                                        "None", "MI", "BC", "PR", "DR", "BL", "SH", "TS", "FZ",
                                        "VC",
                                    ] {
                                        ui.selectable_value(
                                            &mut modifier,
                                            value.to_string(),
                                            value,
                                        );
                                    }
                                });
                            item.modifiers = match modifier.as_str() {
                                "MI" => Some(vec![crate::metar::PhenomenonModifier::MI]),
                                "BC" => Some(vec![crate::metar::PhenomenonModifier::BC]),
                                "PR" => Some(vec![crate::metar::PhenomenonModifier::PR]),
                                "DR" => Some(vec![crate::metar::PhenomenonModifier::DR]),
                                "BL" => Some(vec![crate::metar::PhenomenonModifier::BL]),
                                "SH" => Some(vec![crate::metar::PhenomenonModifier::SH]),
                                "TS" => Some(vec![crate::metar::PhenomenonModifier::TS]),
                                "FZ" => Some(vec![crate::metar::PhenomenonModifier::FZ]),
                                "VC" => Some(vec![crate::metar::PhenomenonModifier::VC]),
                                _ => None,
                            };
                        }
                    });
                if !open {
                    self.phenomenon_editor = None;
                }
            }

            ui.horizontal(|ui| {
                ui.label("Temperature:");
                ui.add(
                    egui::DragValue::new(&mut self.metar.temperature)
                        .suffix(" °C")
                        .range(-50..=50),
                );
                ui.label("Dew Point:");
                ui.add(
                    egui::DragValue::new(&mut self.metar.dew_point)
                        .suffix(" °C")
                        .range(-50..=50),
                );
            });
            ui.horizontal(|ui| {
                ui.label("QNH:");
                ui.add(
                    egui::DragValue::new(&mut self.metar.pressure.value)
                        .suffix(" hPa")
                        .range(800.0..=1200.0),
                );
            });

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Send to servers").clicked() {
                    let report = if self.overwrite {
                        self.overwrite_metar.trim().to_owned()
                    } else {
                        self.metar.to_string()
                    };
                    let servers: Vec<_> = self.servers.iter()
                        .filter(|server| !server.url.trim().is_empty())
                        .map(|server| (server.url.trim().to_owned(), server.api_key.trim().to_owned()))
                        .collect();
                    if servers.is_empty() {
                        self.server_results.lock().unwrap().push("Add at least one server URL in Settings → Servers.".into());
                    } else if report.is_empty() {
                        self.server_results.lock().unwrap().push("METAR is empty; nothing was sent.".into());
                    } else {
                        self.server_results.lock().unwrap().clear();
                        let results = Arc::clone(&self.server_results);
                        let context = ui.ctx().clone();
                        send_to_servers(servers, report, move |result| {
                            if let Ok(mut results) = results.lock() {
                                results.push(result);
                            }
                            context.request_repaint();
                        });
                    }
                }
                if let Ok(results) = self.server_results.lock() {
                    for result in results.iter() {
                        ui.label(result);
                    }
                }
            });
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("📋").clicked() {
                    ui.ctx().copy_text(self.metar.to_string());
                };
                ui.add_space(10.0);
                if !self.overwrite {
                    self.overwrite_metar = self.metar.to_string();
                }
                ui.vertical(|ui| {
                    ui.label(self.metar.to_string());
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.overwrite, "Overwrite");
                        ui.add_enabled(
                            self.overwrite,
                            egui::TextEdit::singleline(&mut self.overwrite_metar)
                                .hint_text("METAR")
                                .clip_text(false)
                        );
                    });
                });
            });
            ui.separator();

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::warn_if_debug_build(ui);
            });
        });

        egui::Window::new("Sound settings")
            .open(&mut self.sound_settings_window_open)
            .show(ui.ctx(), |ui| {
                let mut changed = false;
                changed |= ui
                    .checkbox(&mut self.reminder_config.enabled, "Enabled")
                    .changed();
                changed |= ui
                    .add(
                        egui::RangeSlider::new(
                            &mut self.reminder_config.start_hour,
                            &mut self.reminder_config.end_hour,
                            0..=23,
                        )
                        .text("Active hours"),
                    )
                    .changed();
                if changed {
                    self.reminder.update_config(self.reminder_config);
                }
                if ui.button("Test sound").clicked() {
                    self.reminder.test_sound();
                }
            });

        egui::Window::new("Server settings")
            .open(&mut self.server_settings_window_open)
            .show(ui.ctx(), |ui| {
                ui.label("List of servers to which METARs are sent:");
                ui.label("Each URL receives an HTTP POST with the report as plain text.");
                let mut remove = None;
                for (index, server) in self.servers.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        ui.add(egui::TextEdit::singleline(&mut server.url).hint_text("https://server.example/metar"));
                        ui.add(
                            egui::TextEdit::singleline(&mut server.api_key)
                                .password(true)
                                .hint_text("Optional API key"),
                        );
                        if ui.button("Remove").clicked() {
                            remove = Some(index);
                        }
                    });
                }
                if let Some(index) = remove {
                    self.servers.remove(index);
                }
                if ui.button("Add server").clicked() {
                    self.servers.push(ServerConfig::default());
                }
            });
    }

    fn save(&mut self, storage: &mut dyn Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}
