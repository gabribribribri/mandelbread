use std::time::Duration;

use eframe::egui;
use egui::RichText;
use mandelbread::{
    engines::sfml_engine::SfmlEngine,
    fractal_engine::{FractalContext, FractalEngine, FractalInfos},
};

#[derive(PartialEq, Eq)]
enum SelectedEngine {
    Sfml,
}

pub struct GuiWrapper {
    selected_engine: SelectedEngine,
    current_engine: Box<dyn FractalEngine>,
    fractal_infos: FractalInfos,
}

impl Default for GuiWrapper {
    fn default() -> Self {
        Self {
            selected_engine: SelectedEngine::Sfml,
            current_engine: SfmlEngine::spawn(),
            fractal_infos: FractalInfos::new(),
        }
    }
}

impl GuiWrapper {
    fn infos(&mut self, ui: &mut egui::Ui) {
        self.fractal_infos
            .fuse_together(&self.current_engine.get_infos());

        ui.horizontal(|ui| {
            ui.label(RichText::new("Reload Time : ").strong());
            ui.label(format!(
                "{:?}",
                self.fractal_infos.reload_time.unwrap_or(Duration::ZERO)
            ));
        });
    }

    fn left_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Engines");
        ui.selectable_value(&mut self.selected_engine, SelectedEngine::Sfml, "SFML");
    }

    fn sfml_frontend(&mut self, ui: &mut egui::Ui) {
        ui.heading("SFML Engine");

        if ui.button("Reload").clicked() {
            self.current_engine.reload();
        }
    }
}

impl eframe::App for GuiWrapper {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("left_panel")
            .resizable(true)
            .show(ctx, |ui| self.left_panel(ui));

        egui::CentralPanel::default().show(ctx, |ui| match self.selected_engine {
            SelectedEngine::Sfml => self.sfml_frontend(ui),
        });

        egui::TopBottomPanel::bottom("bottom_panel")
            .resizable(true)
            .show(ctx, |ui| self.infos(ui));
    }
}
