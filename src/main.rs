use crate::ui::GuiWrapper;

mod ui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|_cc| Ok(Box::<GuiWrapper>::default())),
    )?;

    Ok(())
}
