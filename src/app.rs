use std::sync::Arc;

use egui::{FontData, FontFamily};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct EguiApp {
    counter: i64,
}

impl EguiApp {
    pub const APP_NAME: &'static str = "logirecon";

    pub fn new(cc: &eframe::CreationContext) -> Self {
        Self::setup_fonts(&cc.egui_ctx);
        if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn setup_fonts(ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();
        // ********* simhei *********
        fonts.font_data.insert(
            "simhei".into(),
            Arc::new(FontData::from_static(include_bytes!(
                "../public/font/simhei.ttf"
            ))),
        );
        fonts
            .families
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .insert(0, "simhei".into());
        fonts
            .families
            .get_mut(&FontFamily::Monospace)
            .unwrap()
            .insert(0, "simhei".into());
        ctx.set_fonts(fonts);
    }

    fn counter(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("-").clicked() {
                self.counter -= 1;
            }
            ui.label(format!("{}", self.counter));
            if ui.button("+").clicked() {
                self.counter += 1;
            }
        });
    }
}

impl eframe::App for EguiApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.heading("你好世界");
            self.counter(ui);
        });
    }
}
