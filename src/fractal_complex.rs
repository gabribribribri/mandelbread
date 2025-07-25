use rug::{
    Assign,
    float::Round,
    ops::{AddFrom, DivAssignRound, MulAssignRound, MulFrom},
};
use sfml::system::Vector2;

// I don't know why but this is faster than `core::f64::<impl f64>::abs`
#[inline]
fn f_abs(n: f64) -> f64 {
    f64::from_bits(0x7FFF_FFFF_FFFF_FFFF & n.to_bits())
}

#[derive(Clone, Copy)]
pub struct Complex<T> {
    pub re: T,
    pub im: T,
}

impl<T> Complex<T> {
    pub fn new(re: T, im: T) -> Self {
        Self { re, im }
    }
}

impl Complex<f64> {
    pub fn map_pixel_value_f64(res: Self, center: Self, window: Self, coord: Self) -> Self {
        Self::new(
            center.re - (window.re / 2.0) + (coord.re / res.re) * window.re,
            center.im - (window.im / 2.0) + ((res.im - coord.im) / res.im) * window.im,
        )
    }

    pub fn f_sq_add_f64(&mut self, c: Self) {
        (self.re, self.im) = (
            self.re * self.re - self.im * self.im + c.re,
            2.0 * self.re * self.im + c.im,
        );
    }

    pub fn abs_sum_f64(&self) -> f64 {
        f_abs(self.re) + f_abs(self.im)
    }
}

pub fn iter_gradient(iter: u32, seq_iter: u32) -> [u8; 4] {
    let iter_norm = iter as f64 / seq_iter as f64;
    let mid = 0.35;
    let (red, green, blue);

    if iter_norm <= mid {
        let t = iter_norm / mid;
        red = (1. - t) * 255.;
        green = t * 255.;
        blue = 0.;
    } else {
        let t = (iter_norm - mid) / (1. - mid);
        red = 0.;
        green = (1. - t) * 255.;
        blue = t * 255.;
    }

    [red as u8, green as u8, blue as u8, 255]
}

pub fn map_pixel_value_rug(
    res: Vector2<u32>,
    center: &rug::Complex,
    window: &rug::Complex,
    coord: (i32, i32),
) -> rug::Complex {
    let mut result = window.clone();
    result
        .mut_real()
        .mul_assign_round(-(res.x as i32) + 2 * coord.0, Round::Nearest);
    result.mut_imag().mul_assign_round(
        -(res.y as i32) + 2 * (res.y as i32 - coord.1),
        Round::Nearest,
    );
    result
        .mut_real()
        .div_assign_round(2 * res.x, Round::Nearest);
    result
        .mut_imag()
        .div_assign_round(2 * res.y, Round::Nearest);
    result.add_from(center);
    result
}

pub fn f_sq_add_rug(n: &rug::Complex, c: &rug::Complex) -> rug::Complex {
    let mut r = rug::Complex::new(n.prec());
    r.mut_real()
        .assign(n.real() * n.real() - n.imag() * n.imag());
    r.mut_imag().assign(n.real() * n.imag());
    r.mut_imag().mul_from(2.0);
    r.add_from(c);
    r
}

pub fn abs_sum_rug(n: &rug::Complex) -> f64 {
    n.real().to_f64().abs() + n.imag().to_f64().abs()
}
