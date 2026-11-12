use crate::i18n::{Language, localized};
use crate::metar::{MessageType, ObservationMessage};
use crate::reminder::{ReminderConfig, ReminderService};
use crate::server::send_to_servers;
use eframe::{Frame, Storage};
use egui::Ui;
use std::sync::{Arc, Mutex};

fn selected_report(metar: &ObservationMessage, overwrite: bool, overwrite_metar: &str) -> String {
    if overwrite {
        overwrite_metar.trim().to_owned()
    } else {
        metar.to_string()
    }
}

#[derive(serde::Deserialize, serde::Serialize, Default)]
#[serde(default)]
#[serde(deny_unknown_fields)]
struct ServerConfig {
    name: String,
    url: String,
    api_key: String,
}

#[cfg(test)]
mod tests {
    use super::{parse_server_config, selected_report};
    use crate::i18n::Language;
    use crate::metar::ObservationMessage;

    #[test]
    fn report_selection_respects_overwrite() {
        let metar = ObservationMessage::default();
        let edited = "METAR EPBC 061200Z 28005KT 5000 NSC M05/M12 Q1013=";
        assert_eq!(selected_report(&metar, false, edited), metar.to_string());
        assert_eq!(selected_report(&metar, true, edited), edited);
        assert_eq!(
            selected_report(&metar, true, &format!("  {edited}\n")),
            edited
        );
        assert_eq!(selected_report(&metar, true, "  "), "");
    }

    #[test]
    fn loads_servers_with_optional_api_keys() {
        let servers = parse_server_config(r#"{"servers":[{"url":"https://example.com/metar","api_key":"secret"},{"url":"http://localhost:8080"}]}"#, Language::English).unwrap();
        assert_eq!(servers.len(), 2);
        assert_eq!(servers[0].url, "https://example.com/metar");
        assert_eq!(servers[0].api_key, "secret");
        assert!(servers[1].api_key.is_empty());
    }

    #[test]
    fn rejects_invalid_configs_without_exposing_credentials() {
        for contents in [
            r#"{"servers":[{"url":"https://example.com"},{"url":"ftp://example.com","api_key":"secret"}]}"#,
            r#"{"servers":[{"api_key":"secret"}]}"#,
            r#"{"servers":[{"url":"https://example.com","api_key":123456}]}"#,
            r#"{"servers":[{"url":"https://example.com","api_ke":"secret"}]}"#,
            r#"{}"#,
        ] {
            let error = parse_server_config(contents, Language::English)
                .err()
                .expect("invalid config");
            assert!(!error.contains("secret"));
            assert!(!error.contains("123456"));
        }
    }

    #[test]
    fn config_errors_are_localized_without_exposing_credentials() {
        let error = parse_server_config(
            r#"{"servers":[{"url":"ftp://example.com","api_key":"secret"}]}"#,
            Language::Polish,
        )
        .err()
        .unwrap();
        assert_eq!(error, "Serwer 1 musi mieć poprawny adres URL HTTP(S).");
        assert!(!error.contains("secret"));
    }
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ServerConfigFile {
    servers: Vec<ServerConfig>,
}

#[derive(serde::Serialize)]
struct ServerConfigExport<'a> {
    servers: &'a [ServerConfig],
}

fn parse_server_config(contents: &str, language: Language) -> Result<Vec<ServerConfig>, String> {
    let config: ServerConfigFile = serde_json::from_str(contents).map_err(|error| {
        localized!(language, "Invalid server config at line {}, column {}. Expected a JSON object with a servers list containing url, optional name and optional api_key fields.", "Niepoprawna konfiguracja serwerów: wiersz {}, kolumna {}. Oczekiwano obiektu JSON z listą servers zawierającą url oraz opcjonalne pola name i api_key.", error.line(), error.column())
    })?;
    for (index, server) in config.servers.iter().enumerate() {
        let valid = reqwest::Url::parse(server.url.trim())
            .map(|url| matches!(url.scheme(), "http" | "https") && url.host_str().is_some())
            .unwrap_or(false);
        if !valid {
            return Err(localized!(
                language,
                "Server {} must have a valid HTTP(S) URL.",
                "Serwer {} musi mieć poprawny adres URL HTTP(S).",
                index + 1
            ));
        }
    }
    Ok(config.servers)
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct App {
    language: Language,
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
    server_config_path: String,
    #[serde(skip)]
    server_config_status: String,
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
            language: Language::default(),
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
            server_config_path: String::new(),
            server_config_status: String::new(),
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
        let language = self.language;
        egui::Panel::top("top_panel").show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button(language.text("File", "Plik"), |ui| {
                        if ui.button(language.text("Quit", "Zakończ")).clicked() {
                            ui.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                    ui.menu_button(language.text("Settings", "Ustawienia"), |ui| {
                        ui.menu_button(language.text("Language", "Język"), |ui| {
                            ui.selectable_value(&mut self.language, Language::English, "English");
                            ui.selectable_value(&mut self.language, Language::Polish, "Polski");
                        });
                        if ui.button(language.text("Sound", "Dźwięk")).clicked() {
                            self.sound_settings_window_open = true;
                        }
                        if ui.button(language.text("Servers", "Serwery")).clicked() {
                            self.server_settings_window_open = true;
                        }
                    });
                    ui.add_space(16.0);
                }

                let current = ui.ctx().options(|options| options.theme_preference);
                for (theme, label) in [
                    (
                        egui::ThemePreference::System,
                        language.text("System", "Systemowy"),
                    ),
                    (
                        egui::ThemePreference::Light,
                        language.text("Light", "Jasny"),
                    ),
                    (egui::ThemePreference::Dark, language.text("Dark", "Ciemny")),
                ] {
                    if ui.selectable_label(current == theme, label).clicked() {
                        ui.ctx().set_theme(theme);
                    }
                }
            });
        });

        let language = self.language;
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
                ui.label(language.text("Day:", "Dzień:"));
                ui.add(egui::DragValue::new(&mut self.metar.observation_day).range(1..=31));
            });
            ui.horizontal(|ui| {
                ui.label(language.text("Time:", "Czas UTC:"));
                ui.add(egui::DragValue::new(&mut self.hour).range(0..=23));
                ui.label(":");
                ui.add(egui::DragValue::new(&mut self.minute).range(0..=59));
            });

            self.metar.observation_time = self.hour * 100 + self.minute;

            ui.label(language.text("Wind:", "Wiatr:"));
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
                ui.label(language.text("Speed:", "Prędkość:"));
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
                ui.label(language.text("Visibility:", "Widzialność:"));
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
                ui.label(language.text("Clouds:", "Chmury:"));
                let clouds = &mut self.metar.visibility_phenomena_clouds_group.cloud_layers;
                if let Some(clouds) = clouds {
                    for (index, cloud) in clouds.iter().enumerate() {
                        ui.label(cloud.to_string());
                        if ui
                            .small_button("📝")
                            .on_hover_text(language.text("Edit", "Edytuj"))
                            .clicked()
                        {
                            self.cloud_editor = Some(index);
                        }
                        if ui
                            .small_button("🗑️")
                            .on_hover_text(language.text("Remove", "Usuń"))
                            .clicked()
                        {
                            clouds.remove(index);
                            break;
                        }
                    }
                }
                if ui
                    .button("➕")
                    .on_hover_text(language.text("Add", "Dodaj"))
                    .clicked()
                {
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
                ui.label(language.text("Phenomenon:", "Zjawiska:"));
                let phenomena = &mut self.metar.visibility_phenomena_clouds_group.phenomena;
                if let Some(phenomena) = phenomena {
                    for (index, phenomenon) in phenomena.iter().enumerate() {
                        ui.label(phenomenon.to_string());
                        if ui
                            .small_button("📝")
                            .on_hover_text(language.text("Edit", "Edytuj"))
                            .clicked()
                        {
                            self.phenomenon_editor = Some(index);
                        }
                        if ui
                            .small_button("🗑️")
                            .on_hover_text(language.text("Remove", "Usuń"))
                            .clicked()
                        {
                            phenomena.remove(index);
                            break;
                        }
                    }
                }
                if ui
                    .button("➕")
                    .on_hover_text(language.text("Add", "Dodaj"))
                    .clicked()
                {
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
                egui::Window::new(language.text("Edit cloud layer", "Edycja warstwy chmur"))
                    .id(egui::Id::unique("cloud_editor"))
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
                                ui.label(language.text("Coverage", "Zachmurzenie"));
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
                                ui.label(language.text("Height: ", "Wysokość: "));
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
                                .selected_text(
                                    language
                                        .text(&kind, if kind == "None" { "Brak" } else { &kind }),
                                )
                                .show_ui(ui, |ui| {
                                    for value in ["None", "CB", "TCU"] {
                                        ui.selectable_value(
                                            &mut kind,
                                            value.to_string(),
                                            language.text(
                                                value,
                                                if value == "None" { "Brak" } else { value },
                                            ),
                                        );
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
                egui::Window::new(language.text("Edit phenomenon", "Edycja zjawiska"))
                    .id(egui::Id::unique("phenomenon_editor"))
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
                                .selected_text(language.text(
                                    &intensity,
                                    if intensity == "None" {
                                        "Brak"
                                    } else {
                                        &intensity
                                    },
                                ))
                                .show_ui(ui, |ui| {
                                    for value in ["None", "+", "-"] {
                                        ui.selectable_value(
                                            &mut intensity,
                                            value.to_string(),
                                            language.text(
                                                value,
                                                if value == "None" { "Brak" } else { value },
                                            ),
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
                                .selected_text(language.text(
                                    &modifier,
                                    if modifier == "None" {
                                        "Brak"
                                    } else {
                                        &modifier
                                    },
                                ))
                                .show_ui(ui, |ui| {
                                    for value in [
                                        "None", "MI", "BC", "PR", "DR", "BL", "SH", "TS", "FZ",
                                        "VC",
                                    ] {
                                        ui.selectable_value(
                                            &mut modifier,
                                            value.to_string(),
                                            language.text(
                                                value,
                                                if value == "None" { "Brak" } else { value },
                                            ),
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
                ui.label(language.text("Temperature:", "Temperatura:"));
                ui.add(
                    egui::DragValue::new(&mut self.metar.temperature)
                        .suffix(" °C")
                        .range(-50..=50),
                );
                ui.label(language.text("Dew Point:", "Punkt rosy:"));
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
                if let Ok(results) = self.server_results.lock() {
                    for result in results.iter() {
                        ui.label(result);
                    }
                }
            });
            ui.separator();
            ui.horizontal(|ui| {
                if ui
                    .button("✅")
                    .on_hover_text(language.text(
                        "Send to servers and copy report",
                        "Wyślij do serwerów i kopiuj depeszę",
                    ))
                    .clicked()
                {
                    let report =
                        selected_report(&self.metar, self.overwrite, &self.overwrite_metar);
                    ui.ctx().copy_text(report.clone());
                    let servers: Vec<_> = self
                        .servers
                        .iter()
                        .enumerate()
                        .filter(|(_, server)| !server.url.trim().is_empty())
                        .map(|(index, server)| {
                            let name = if server.name.trim().is_empty() {
                                localized!(language, "Server {}", "Serwer {}", index + 1)
                            } else {
                                server.name.trim().to_owned()
                            };
                            (
                                name,
                                server.url.trim().to_owned(),
                                server.api_key.trim().to_owned(),
                            )
                        })
                        .collect();
                    if servers.is_empty() {
                        self.server_results.lock().unwrap().push(
                            language
                                .text(
                                    "Add at least one server URL in Settings → Servers.",
                                    "Dodaj co najmniej jeden adres serwera w Ustawienia → Serwery.",
                                )
                                .into(),
                        );
                    } else if report.is_empty() {
                        self.server_results.lock().unwrap().push(
                            language
                                .text(
                                    "METAR is empty; nothing was sent.",
                                    "Depesza METAR jest pusta; nic nie wysłano.",
                                )
                                .into(),
                        );
                    } else {
                        self.server_results.lock().unwrap().clear();
                        let results = Arc::clone(&self.server_results);
                        let context = ui.ctx().clone();
                        send_to_servers(servers, report, language, move |result| {
                            if let Ok(mut results) = results.lock() {
                                results.push(result);
                            }
                            context.request_repaint();
                        });
                    }
                }
                ui.add_space(10.0);
                if !self.overwrite {
                    self.overwrite_metar = self.metar.to_string();
                }
                ui.vertical(|ui| {
                    ui.label(self.metar.to_string());
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.overwrite, language.text("Overwrite", "Nadpisz"));
                        ui.add_enabled(
                            self.overwrite,
                            egui::TextEdit::singleline(&mut self.overwrite_metar)
                                .hint_text("METAR")
                                .clip_text(false),
                        );
                    });
                });
            });
            ui.separator();

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                if cfg!(debug_assertions) {
                    ui.colored_label(
                        ui.visuals().warn_fg_color,
                        language.text("Debug build", "Wersja debugowania"),
                    );
                }
            });
        });

        egui::Window::new(language.text("Sound settings", "Ustawienia dźwięku"))
            .id(egui::Id::unique("sound_settings"))
            .open(&mut self.sound_settings_window_open)
            .show(ui.ctx(), |ui| {
                let mut changed = false;
                changed |= ui
                    .checkbox(
                        &mut self.reminder_config.enabled,
                        language.text("Enabled", "Włączone"),
                    )
                    .changed();
                changed |= ui
                    .add(
                        egui::RangeSlider::new(
                            &mut self.reminder_config.start_hour,
                            &mut self.reminder_config.end_hour,
                            0..=23,
                        )
                        .text(language.text("Active hours", "Godziny aktywności")),
                    )
                    .changed();
                if changed {
                    self.reminder.update_config(self.reminder_config);
                }
                if ui
                    .button(language.text("Test sound", "Test dźwięku"))
                    .clicked()
                {
                    self.reminder.test_sound();
                }
            });

        egui::Window::new(language.text("Server settings", "Ustawienia serwerów")).id(egui::Id::unique("server_settings"))
            .open(&mut self.server_settings_window_open)
            .show(ui.ctx(), |ui| {
                ui.label(language.text("JSON config file (loading replaces the current server list):", "Plik konfiguracji JSON (wczytanie zastępuje bieżącą listę serwerów):"));
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.server_config_path)
                            .hint_text(language.text("Path to servers.json", "Ścieżka do servers.json")),
                    );
                    if ui.button(language.text("Load config", "Wczytaj konfigurację")).clicked() {
                        let result = std::fs::read_to_string(self.server_config_path.trim())
                            .map_err(|error| localized!(language, "Could not read config file: {error}", "Nie można odczytać pliku konfiguracji: {error}"))
                            .and_then(|contents| parse_server_config(&contents, language));
                        match result {
                            Ok(servers) => {
                                self.server_config_status = localized!(language, "Loaded {} server(s).", "Wczytano serwery: {}.", servers.len());
                                self.servers = servers;
                                if let Ok(mut results) = self.server_results.lock() {
                                    results.clear();
                                }
                            }
                            Err(error) => self.server_config_status = error,
                        }
                    }
                    if ui.button(language.text("Export config", "Eksportuj konfigurację")).clicked() {
                        let result = serde_json::to_string_pretty(&ServerConfigExport {
                            servers: &self.servers,
                        })
                        .map_err(|error| localized!(language, "Could not serialize config: {error}", "Nie można zakodować konfiguracji: {error}"))
                        .and_then(|contents| {
                            std::fs::write(self.server_config_path.trim(), contents)
                                .map_err(|error| localized!(language, "Could not write config file: {error}", "Nie można zapisać pliku konfiguracji: {error}"))
                        });
                        self.server_config_status = match result {
                            Ok(()) => localized!(language, "Exported {} server(s).", "Wyeksportowano serwery: {}.", self.servers.len()),
                            Err(error) => error,
                        };
                    }
                });
                ui.label(language.text("Export includes API keys and overwrites the file at the specified path.", "Eksport zawiera klucze API i nadpisuje plik pod wskazaną ścieżką."));
                if !self.server_config_status.is_empty() {
                    ui.label(&self.server_config_status);
                }
                ui.separator();
                ui.label(language.text("List of servers to which METARs are sent:", "Lista serwerów odbierających depesze METAR:"));
                ui.label(language.text("Each URL receives an HTTP POST with the report as plain text.", "Pod każdy adres URL wysyłane jest żądanie HTTP POST z depeszą jako zwykłym tekstem."));
                let mut remove = None;
                for (index, server) in self.servers.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        ui.add(egui::TextEdit::singleline(&mut server.name).hint_text(language.text("Name", "Nazwa")).desired_width(120.0));
                        ui.add(egui::TextEdit::singleline(&mut server.url).hint_text("https://server.example/metar"));
                        ui.add(
                            egui::TextEdit::singleline(&mut server.api_key)
                                .password(true)
                                .hint_text(language.text("Optional API key", "Opcjonalny klucz API")),
                        );
                        if ui.button(language.text("Remove", "Usuń")).clicked() {
                            remove = Some(index);
                        }
                    });
                }
                if let Some(index) = remove {
                    self.servers.remove(index);
                }
                if ui.button(language.text("Add server", "Dodaj serwer")).clicked() {
                    self.servers.push(ServerConfig::default());
                }
            });
    }

    fn save(&mut self, storage: &mut dyn Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}
