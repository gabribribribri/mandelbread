use std::ops::{Add, Div, Mul, Sub};

#[derive(Default, Clone, Copy)]
pub struct Complex<T> {
    pub re: T,
    pub im: T,
}

impl<T> Complex<T>
where
    T: Copy + Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Div<Output = T> + FromLossyU32,
{
    pub fn new(re: T, im: T) -> Self {
        Self { re, im }
    }

    #[inline]
    pub fn sq_add(&mut self, c: Complex<T>) {
        (self.re, self.im) = (
            self.re * self.re - self.im * self.im + c.re,
            T::f_u32(2) * self.re * self.im + c.im,
        );
    }

    pub fn map_between(
        res: (u32, u32),
        start: Complex<T>,
        end: Complex<T>,
        value: (u32, u32),
    ) -> Complex<T> {
        let re = T::f_u32(value.0) * (end.re - start.re) / T::f_u32(res.0) + start.re;
        let im = T::f_u32(value.1) * (end.im - start.im) / T::f_u32(res.1) + start.im;
        Complex { re, im }
    }
}

impl Into<Complex<f64>> for Complex<f32> {
    fn into(self) -> Complex<f64> {
        Complex::new(self.re as f64, self.im as f64)
    }
}

pub trait FromLossyU32: Sized {
    fn f_u32(n: u32) -> Self;
}

impl FromLossyU32 for f32 {
    fn f_u32(n: u32) -> Self {
        n as Self
    }
}

impl FromLossyU32 for f64 {
    fn f_u32(n: u32) -> Self {
        n as Self
    }
}
