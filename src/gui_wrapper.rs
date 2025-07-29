use std::time::{Duration, Instant};

use crate::{fractal_engine::FractalEngine, sfml_engine::SfmlEngine};

const RELOAD_DUR: Duration = Duration::from_millis(17);

pub struct GuiWrapper {
    sfml_engine: SfmlEngine,
    last_update: Instant,
}

impl Default for GuiWrapper {
    fn default() -> Self {
        Self {
            sfml_engine: SfmlEngine::new(),
            last_update: Instant::now(),
        }
    }
}

impl eframe::App for GuiWrapper {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Continuous reloading
        let now = Instant::now();
        let delta = now - self.last_update;
        if delta > RELOAD_DUR {
            ctx.request_repaint();
            self.last_update = now;
        }

        egui::TopBottomPanel::bottom("bottom_panel")
            .show(ctx, |ui| self.sfml_engine.gui_bottom_panel(ui));

        egui::CentralPanel::default().show(ctx, |ui| self.sfml_engine.gui_central_panel(ui));
    }
}
