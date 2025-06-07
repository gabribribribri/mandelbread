use mandelbread::{fractal_engine::FractalEngine, sfml_engine::SfmlEngine};

use crate::ui::Mandelbread as GuiWrapper;

mod ui;

fn main() -> eframe::Result {
    // let options = eframe::NativeOptions {
    //     viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
    //     ..Default::default()
    // };
    // eframe::run_native(
    //     "My egui App",
    //     options,
    //     Box::new(|cc| Ok(Box::<GuiWrapper>::default())),
    // );

    let mut engine = SfmlEngine::new().unwrap();
    loop {
        match engine.reload() {
            Ok(d) => println!("{:?}", d),
            Err(e) => println!("{:?}", e.to_string()),
        }
        engine.render();
    }
}
