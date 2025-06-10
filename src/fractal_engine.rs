use std::fmt;

use rug::{Complex, Float};

const FRACTAL_CONTEXT_COMPLEX_PRECISION: u32 = 256;

#[derive(Clone)]
pub struct FractalContext {
    pub res: (u32, u32),
    pub center: Complex,
    pub window: Complex, // window.re : length window
    pub backend: FractalBackend,
}

#[derive(Copy, Clone)]
pub enum FractalBackend {
    F32,
    F64,
}

pub enum FractalNotif {
    Commence(FractalContext),
    Shutdown,
    Reload,
    Move(Complex),
    Zoom(f32),
}

pub enum FractalEngineError {
    SendError,
}

impl Default for FractalContext {
    fn default() -> Self {
        Self {
            res: (800, 600),
            center: Complex::with_val(FRACTAL_CONTEXT_COMPLEX_PRECISION, 0.0),
            window: Complex::with_val(FRACTAL_CONTEXT_COMPLEX_PRECISION, (2.66, 2.0)),
            backend: FractalBackend::F32,
        }
    }
}

pub trait FractalEngine {
    fn commence(&self) -> Result<(), FractalEngineError>;

    fn shutdown(&mut self) -> Result<(), FractalEngineError>;

    fn reload(&mut self) -> Result<(), FractalEngineError>;

    fn move_view(&mut self, translation: Complex) -> Result<(), FractalEngineError>;

    fn zoom_view(&mut self, zoom: f32) -> Result<(), FractalEngineError>;

    fn gui_bottom_panel(&mut self, ui: &mut egui::Ui);

    fn gui_central_panel(&mut self, ui: &mut egui::Ui);
}
