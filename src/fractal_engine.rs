use std::time::Duration;

use rug;

use crate::fractal_complex::Complex;

const FRCTL_CTX_CMPLX_PREC: u32 = 256;

pub mod lodiv {
    pub const HIGHEST: u32 = 1;
    pub const FAST: u32 = 2;
    pub const FASTER: u32 = 3;
    pub const FASTEST: u32 = 5;
}

#[derive(Clone)]
pub struct FractalContext {
    pub res: (u32, u32),
    pub center: rug::Complex,
    pub window: rug::Complex, // window.re : length window
    pub backend: FractalBackend,
    pub lodiv: u32,
}

#[derive(Copy, Clone, PartialEq)]
pub enum FractalBackend {
    F32,
    F64,
}

pub enum FractalNotif {
    Commence(FractalContext),
    Shutdown,
    ResetView,
    Reload,
    ReloadTime(Duration),
    Move(Complex<f32>),
    ChangeWindow(rug::Complex),
    Zoom(f32),
    ChangeResolution(u32, u32),
    ChangeLodiv(u32),
    ChangeBackend(FractalBackend),
}

#[derive(Debug)]
pub enum FractalEngineError {
    SendError,
}

impl Default for FractalContext {
    fn default() -> Self {
        Self {
            res: (800, 600),
            center: rug::Complex::with_val(FRCTL_CTX_CMPLX_PREC, -0.5),
            window: rug::Complex::with_val(FRCTL_CTX_CMPLX_PREC, (2.66, 2.0)),
            backend: FractalBackend::F32,
            lodiv: lodiv::HIGHEST,
        }
    }
}

pub trait FractalEngine {
    fn commence(&self) -> Result<(), FractalEngineError>;

    fn shutdown(&mut self) -> Result<(), FractalEngineError>;

    fn reset_view(&mut self) -> Result<(), FractalEngineError>;

    fn reload(&mut self) -> Result<(), FractalEngineError>;

    fn move_window(&mut self, translation: Complex<f32>) -> Result<(), FractalEngineError>;

    fn zoom_view(&mut self, zoom: f32) -> Result<(), FractalEngineError>;

    fn change_lodiv(&mut self, lodiv: u32) -> Result<(), FractalEngineError>;

    fn change_backend(&mut self, backend: FractalBackend) -> Result<(), FractalEngineError>;
    fn gui_bottom_panel(&mut self, ui: &mut egui::Ui);

    fn gui_central_panel(&mut self, ui: &mut egui::Ui);
}
