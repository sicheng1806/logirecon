use std::sync::Arc;

use egui::{FontData, FontFamily};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct EguiApp {}

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
}

impl eframe::App for EguiApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("title").show_inside(ui, |ui| {
            ui.heading("核对你的物流账单");
        });
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.add_space(12.0);
            ui.strong("主内容区");
        });
    }
}
