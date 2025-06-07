use std::time::Duration;

use crate::complex::Complex;

pub struct FractalContext<T> {
    pub resolution: (u32, u32),
    pub start: Complex<T>, // up top
    pub end: Complex<T>,   // down bottom
}

pub trait FractalEngine {
    fn reload(&mut self) -> Result<std::time::Duration, Box<dyn std::error::Error>>;

    fn render(&mut self);

    fn deinitialize(&mut self);
}
