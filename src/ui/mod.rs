use std::time::Duration;

use eframe::egui;
use mandelbread::{engines::sfml_engine::SfmlEngine, fractal_engine::FractalEngine};

#[derive(PartialEq, Eq)]
enum SelectedEngine {
    Sfml,
}

pub struct GuiWrapper {
    selected_engine: SelectedEngine,
    current_engine: Box<dyn FractalEngine>,
    engine_enabled: bool,
    reload_duration: Duration,
}

impl Default for GuiWrapper {
    fn default() -> Self {
        Self {
            selected_engine: SelectedEngine::Sfml,
            current_engine: Box::new(SfmlEngine::new().unwrap()),
            engine_enabled: true,
            reload_duration: Duration::default(),
        }
    }
}

impl GuiWrapper {
    fn left_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Engines");
        ui.selectable_value(&mut self.selected_engine, SelectedEngine::Sfml, "SFML");
    }

    fn sfml_frontend(&mut self, ui: &mut egui::Ui) {
        ui.heading("SFML Engine");

        if ui.checkbox(&mut self.engine_enabled, "Enabled").clicked() {
            match self.engine_enabled {
                true => self.current_engine = Box::new(SfmlEngine::new().unwrap()),
                false => self.current_engine.deinitialize(),
            }
        };

        if ui.button("Reload").clicked() {
            match self.current_engine.reload() {
                Ok(d) => self.reload_duration = d,
                Err(e) => println!("UNABLE TO RENDER: {}", (*e).to_string()),
            }
        }
    }
}

impl eframe::App for GuiWrapper {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("left_panel")
            .resizable(true)
            .exact_width(300.0)
            .show(ctx, |ui| self.left_panel(ui));

        egui::CentralPanel::default().show(ctx, |ui| match self.selected_engine {
            SelectedEngine::Sfml => self.sfml_frontend(ui),
        });
    }
}
