use std::time::Duration;

use rug;
use sfml::{graphics::glsl::Vec4, system::Vector2u};

use crate::fractal_complex::Complex;

// Some Constants
pub const FRCTL_CTX_CMPLX_PREC: u32 = 128;
pub const INIT_SEQ_ITER: u32 = 75;
pub const SEQ_ITER_FACT_BASE: f64 = 50.;
pub const BASE_CENTER: f64 = -0.72;
pub const BASE_WINDOW: (f64, f64) = (3.3, 0.0);
pub const BASE_WORKER_COUNT: usize = 1;
pub const BASE_CONV_DIST: f64 = 2.;

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
    pub auto_seq_iter: bool,
    pub auto_seq_iter_fact: f64,
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
    Reload(FractalBackend),
}

impl Default for FractalContext {
    fn default() -> Self {
        Self {
            res: (800, 600).into(),
            center: rug::Complex::with_val(FRCTL_CTX_CMPLX_PREC, BASE_CENTER),
            window: rug::Complex::with_val(FRCTL_CTX_CMPLX_PREC, BASE_WINDOW),
            backend: FractalBackend::Shader,
            lodiv: lodiv::HIGHEST,
            seq_iter: INIT_SEQ_ITER,
            auto_seq_iter: true,
            auto_seq_iter_fact: SEQ_ITER_FACT_BASE,
            reload_durs: vec![Duration::ZERO],
            engine_enabled: true,
            worker_count: BASE_WORKER_COUNT,
            converge_distance: BASE_CONV_DIST,
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

    fn set_auto_seq_iter(&mut self, auto_seq_iter: bool);

    fn set_auto_seq_iter_fact(&mut self, auto_seq_iter_fact: f64);

    fn set_workers(&mut self, workers: usize);

    fn set_backend(&mut self, backend: FractalBackend);

    fn set_converge_distance(&mut self, converge_distance: f64);

    fn gui_bottom_panel(&mut self, ui: &mut egui::Ui);

    fn gui_central_panel(&mut self, ui: &mut egui::Ui);
}

pub fn two_f64_to_vec4(a: f64, b: f64) -> Vec4 {
    let x = f32::from_bits((a.to_bits() >> 32) as u32);
    let y = f32::from_bits((a.to_bits() & 0x0000_0000_FFFF_FFFF) as u32);
    let z = f32::from_bits((b.to_bits() >> 32) as u32);
    let w = f32::from_bits((b.to_bits() & 0x0000_0000_FFFF_FFFF) as u32);
    Vec4 { x, y, z, w }
}

pub fn seq_iters_formula(window: &rug::Complex, factor: f64) -> u32 {
    let window_size = f64::max(window.real().to_f64(), window.imag().to_f64());
    INIT_SEQ_ITER + (factor * -window_size.log2()) as u32
}
