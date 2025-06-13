use std::time::Duration;

use rug;
use sfml::system::Vector2u;

use crate::fractal_complex::Complex;

pub const FRCTL_CTX_CMPLX_PREC: u32 = 256;

pub mod lodiv {
    pub const HIGHEST: u32 = 1;
    pub const FAST: u32 = 2;
    pub const FASTER: u32 = 3;
    pub const FASTEST: u32 = 5;
}

#[derive(Clone)]
pub struct FractalContext {
    pub res: Vector2u,
    pub center: rug::Complex,
    pub window: rug::Complex, // window.re : length window
    pub backend: FractalBackend,
    pub lodiv: u32,
    pub reload_dur: Duration,
    pub engine_enabled: bool,
}

#[derive(Copy, Clone, PartialEq)]
pub enum FractalBackend {
    F32,
    F64,
}

pub enum FractalNotif {
    Commence,
    Shutdown,
    Reload,
}

#[derive(Debug)]
pub enum FractalEngineError {
    SendError,
}

impl Default for FractalContext {
    fn default() -> Self {
        Self {
            res: (800, 600).into(),
            center: rug::Complex::with_val(FRCTL_CTX_CMPLX_PREC, -0.5),
            window: rug::Complex::with_val(FRCTL_CTX_CMPLX_PREC, (2.66, 2.0)),
            backend: FractalBackend::F32,
            lodiv: lodiv::HIGHEST,
            reload_dur: Duration::ZERO,
            engine_enabled: true,
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
