use crate::{fractal_engine::FractalEngine, sfml_engine::SfmlEngine};

#[derive(PartialEq)]
enum SelectedPage {
    Sfml,
}

pub struct GuiWrapper {
    selected_page: SelectedPage,
    sfml_engine: SfmlEngine,
}

impl Default for GuiWrapper {
    fn default() -> Self {
        Self {
            selected_page: SelectedPage::Sfml,
            sfml_engine: SfmlEngine::new(),
        }
    }
}

impl GuiWrapper {
    fn left_panel(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
            ui.heading("Engines");
            ui.separator();
            ui.selectable_value(&mut self.selected_page, SelectedPage::Sfml, "🐶 SFML");
        });
    }
}

impl eframe::App for GuiWrapper {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("left_panel")
            .default_width(100.0)
            .show(ctx, |ui| self.left_panel(ui));

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| match self.selected_page {
            SelectedPage::Sfml => self.sfml_engine.gui_bottom_panel(ui),
        });

        egui::CentralPanel::default().show(ctx, |ui| match self.selected_page {
            SelectedPage::Sfml => self.sfml_engine.gui_central_panel(ui),
        });
    }
}
