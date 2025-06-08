use std::time::Duration;

use crate::complex::Complex;

#[derive(Copy, Clone)]
pub struct FractalContext<T> {
    pub res: (u32, u32),
    pub start: Complex<T>, // up left
    pub end: Complex<T>,   // down right
}

impl Default for FractalContext<f32> {
    fn default() -> Self {
        Self {
            res: (800, 600),
            start: Complex::new(-1.66, 1.0),
            end: Complex::new(1.0, -1.0),
        }
    }
}

pub enum FractalAction {
    Shutdown,
    Reload,
    Move(Complex<f32>),
}

pub enum FractalInfoNotif {
    ReloadTime(Duration),
}

#[derive(Clone, Copy, Default)]
pub struct FractalInfos {
    pub reload_time: Option<Duration>,
}

impl FractalInfos {
    pub fn fuse_together(&mut self, other: &FractalInfos) {
        self.reload_time = other.reload_time.or(self.reload_time);
    }
}

pub trait FractalEngine {
    fn reload(&mut self);

    fn render(&mut self);

    fn get_ctx(&self) -> FractalContext<f64>;

    fn get_infos(&mut self) -> FractalInfos;

    fn move_view(&mut self, c: Complex<f32>);
}
