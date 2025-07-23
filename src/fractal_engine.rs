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
    pub seq_iter: u32,
    pub reload_durs: Vec<Duration>,
    pub engine_enabled: bool,
    pub worker_count: usize,
    pub converge_distance: f64,
    pub has_resized: bool,
}

#[derive(Copy, Clone, PartialEq)]
pub enum FractalBackend {
    F64,
    Rug,
    Shader,
}

pub enum FractalNotif {
    Commence,
    Shutdown,
    Reload,
    ChangeBackend(FractalBackend),
}

impl Default for FractalContext {
    fn default() -> Self {
        Self {
            res: (800, 600).into(),
            center: rug::Complex::with_val(FRCTL_CTX_CMPLX_PREC, -0.5),
            window: rug::Complex::with_val(FRCTL_CTX_CMPLX_PREC, (2.66, 2.0)),
            backend: FractalBackend::F64,
            lodiv: lodiv::HIGHEST,
            seq_iter: 100,
            reload_durs: vec![Duration::ZERO],
            engine_enabled: true,
            worker_count: 1,
            converge_distance: 2.0,
            has_resized: true,
        }
    }
}

pub trait FractalEngine {
    fn commence(&self);

    fn shutdown(&mut self);

    fn reset_window(&mut self);

    fn reset_view(&mut self);

    fn reload(&mut self);

    fn move_window(&mut self, translation: Complex<f32>);

    fn zoom_view(&mut self, zoom: f32);

    fn set_lodiv(&mut self, lodiv: u32);

    fn set_seq_iter(&mut self, seq_iter: u32);

    fn set_workers(&mut self, workers: usize);

    fn set_backend(&mut self, backend: FractalBackend);

    fn set_converge_distance(&mut self, converge_distance: f64);

    fn gui_bottom_panel(&mut self, ui: &mut egui::Ui);

    fn gui_central_panel(&mut self, ui: &mut egui::Ui);
}
