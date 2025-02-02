use super::transpose_text;

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Created with Rust, powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct ChordTransposerApp {
    half_steps: i32,
    original_chord_input: String,
    transposed_chords: String,
}

impl Default for ChordTransposerApp {
    fn default() -> Self {
        Self {
            half_steps: 0,
            original_chord_input: String::new(),
            transposed_chords: String::new(),
        }
    }
}

impl eframe::App for ChordTransposerApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Chord Transposer!");

            let slider_changed = ui
                .add(egui::Slider::new(&mut self.half_steps, -11..=11).text("Half Steps"))
                .changed();

            ui.separator();

            let mut text_changed = false;
            let available_width = ui.available_width();
            let column_width = available_width / 2.0 - 4.0;

            ui.with_layout(
                egui::Layout::left_to_right(egui::Align::Center).with_cross_justify(true),
                |ui| {
                    ui.vertical(|ui| {
                        ui.heading("Paste Chords And Lyrics Here");
                        egui::ScrollArea::vertical()
                            .id_salt("scroll_area_1")
                            .show(ui, |ui| {
                                text_changed |= ui
                                    .add(
                                        egui::TextEdit::multiline(&mut self.original_chord_input)
                                            .desired_width(column_width),
                                    )
                                    .changed();
                            });
                    });
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.heading("Transposed");
                        });
                        egui::ScrollArea::vertical()
                            .id_salt("scroll_area_2")
                            .show(ui, |ui| {
                                ui.add(
                                    egui::TextEdit::multiline(&mut self.transposed_chords)
                                        .desired_width(column_width),
                                );
                            });
                    });
                },
            );

            if slider_changed || text_changed {
                self.update_transposed_text();
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
                ui.hyperlink_to(
                    "Source code.",
                    "https://github.com/arincdemir/chord_transposer_rust",
                );
            });
        });
    }
}

impl ChordTransposerApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.

        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }

    fn update_transposed_text(&mut self) {
        self.transposed_chords = transpose_text(&self.original_chord_input, self.half_steps as i32);
    }
}
