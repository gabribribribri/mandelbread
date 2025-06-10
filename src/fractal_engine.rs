use std::{fmt, time::Duration};

use rug;

const FRACTAL_CONTEXT_COMPLEX_PRECISION: u32 = 256;

#[derive(Clone)]
pub struct FractalContext {
    pub res: (u32, u32),
    pub center: rug::Complex,
    pub window: rug::Complex, // window.re : length window
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
    ReloadTime(Duration),
    Move(rug::Complex),
    Zoom(f32),
    Resize(u32, u32),
}

pub enum FractalEngineError {
    SendError,
}

impl Default for FractalContext {
    fn default() -> Self {
        Self {
            res: (800, 600),
            center: rug::Complex::with_val(FRACTAL_CONTEXT_COMPLEX_PRECISION, 0.0),
            window: rug::Complex::with_val(FRACTAL_CONTEXT_COMPLEX_PRECISION, (2.66, 2.0)),
            backend: FractalBackend::F32,
        }
    }
}

pub trait FractalEngine {
    fn commence(&self) -> Result<(), FractalEngineError>;

    fn shutdown(&mut self) -> Result<(), FractalEngineError>;

    fn reload(&mut self) -> Result<(), FractalEngineError>;

    fn move_view(&mut self, translation: rug::Complex) -> Result<(), FractalEngineError>;

    fn zoom_view(&mut self, zoom: f32) -> Result<(), FractalEngineError>;

    fn gui_bottom_panel(&mut self, ui: &mut egui::Ui);

    fn gui_central_panel(&mut self, ui: &mut egui::Ui);
}
