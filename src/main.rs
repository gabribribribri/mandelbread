mod fractal_engine;
mod gui_wrapper;
mod sfml_engine;

use gui_wrapper::GuiWrapper;

fn main() -> eframe::Result {
    eframe::run_native(
        "Mandelbread",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::<GuiWrapper>::default())),
    )
}
