use std::time::Duration;

use eframe::egui;
use egui::{RichText, Vec2};
use mandelbread::{
    complex::Complex,
    engines::sfml_engine::SfmlEngine,
    fractal_engine::{FractalContext, FractalEngine, FractalInfos},
};

#[derive(PartialEq, Eq)]
enum SelectedEngine {
    Sfml,
}

pub struct GuiWrapper {
    selected_engine: SelectedEngine,
    sfml_engine: SfmlEngine,
}

impl Default for GuiWrapper {
    fn default() -> Self {
        Self {
            selected_engine: SelectedEngine::Sfml,
            sfml_engine: SfmlEngine::spawn(),
        }
    }
}

impl GuiWrapper {
    fn left_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Engines");
        ui.selectable_value(&mut self.selected_engine, SelectedEngine::Sfml, "SFML");
    }

    fn sfml_top_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("SFML Engine");

        if ui
            .button(RichText::new("Reload").size(32.0).strong())
            .clicked()
        {
            self.sfml_engine.reload();
        }

        ui.horizontal(|ui| {
            if ui.button("Left").clicked() {
                self.sfml_engine.move_view(Complex::new(-1.0, 0.0))
            }
            if ui.button("Up").clicked() {
                self.sfml_engine.move_view(Complex::new(0.0, 1.0));
            }
            if ui.button("Down").clicked() {
                self.sfml_engine.move_view(Complex::new(0.0, -1.0));
            }
            if ui.button("Right").clicked() {
                self.sfml_engine.move_view(Complex::new(1.0, 0.0));
            }
        });
    }

    fn sfml_bottom_panel(&mut self, ui: &mut egui::Ui) {
        let infos = self.sfml_engine.get_infos();
        let ctx = self.sfml_engine.get_ctx();

        ui.horizontal(|ui| {
            ui.label(RichText::new("Reload Time : ").strong());
            ui.label(format!("{:?}", infos.reload_time.unwrap_or(Duration::ZERO)));
        });

        ui.horizontal(|ui| {
            ui.label(RichText::new("Resolution : ").strong());
            ui.label(format!("{}x{}", ctx.res.0, ctx.res.1));
        });

        ui.horizontal(|ui| {
            ui.label(RichText::new("Range : ").strong());
            ui.label(format!("{}{:+}i", ctx.start.re, ctx.start.im));
            ui.label(RichText::new(" to ").strong());
            ui.label(format!("{}{:+}i", ctx.end.re, ctx.end.im));
        });
    }
}

impl eframe::App for GuiWrapper {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("left_panel")
            .resizable(true)
            .show(ctx, |ui| self.left_panel(ui));

        egui::CentralPanel::default().show(ctx, |ui| match self.selected_engine {
            SelectedEngine::Sfml => self.sfml_top_panel(ui),
        });

        egui::TopBottomPanel::bottom("bottom_panel")
            .resizable(true)
            .show(ctx, |ui| match self.selected_engine {
                SelectedEngine::Sfml => self.sfml_bottom_panel(ui),
            });
    }
}
