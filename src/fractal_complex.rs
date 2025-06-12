use std::ops::{Add, Div, Mul, Sub};

use rug;
use sfml::graphics::Color;

#[derive(Clone, Copy)]
pub struct Complex<T> {
    pub re: T,
    pub im: T,
}

pub trait FractalComplex
where
    Self: Copy,
{
    type FloatType: PartialOrd
        + Copy
        + Add<Output = Self::FloatType>
        + Sub<Output = Self::FloatType>
        + Mul<Output = Self::FloatType>
        + Div<Output = Self::FloatType>;

    fn re(self) -> Self::FloatType;

    fn im(self) -> Self::FloatType;

    fn new(re: Self::FloatType, im: Self::FloatType) -> Self;

    fn float_val_0() -> Self::FloatType;

    fn float_val_1() -> Self::FloatType;

    fn float_val_100() -> Self::FloatType;

    fn float_val_255() -> Self::FloatType;

    fn half(n: Self::FloatType) -> Self::FloatType;

    fn clamp(n: Self::FloatType, min: Self::FloatType, max: Self::FloatType) -> Self::FloatType;

    fn round_to_u8(n: Self::FloatType) -> u8;

    fn fsq_add(&mut self, c: Self);

    fn distance_origin(self) -> Self::FloatType;

    fn from_cmplx(val: &rug::Complex) -> Self;

    fn from_u32_pair(val: (u32, u32)) -> Self;

    fn map_pixel_value(res: Self, center: Self, window: Self, value: Self) -> Self
// where
    //     Self::FloatType: Debug,
    {
        // let d =
        Self::new(
            center.re() - Self::half(window.re()) + (value.re() / res.re()) * window.re(),
            center.im() - Self::half(window.im()) + (value.im() / res.im()) * window.im(),
        )
        // print!("{:?}+{:?}i ", d.re(), d.im());
        // d
    }

    // TODO Make the color more generic
    fn distance_gradient(distance: Self::FloatType) -> Color {
        const START: u32 = 100;
        const END: u32 = 1500;

        // .re() is the start and .im() is the end... disgusting, I know
        let sne = Self::from_u32_pair((START, END));

        let clamped_value = Self::clamp(distance, sne.re(), sne.im());

        let red: Self::FloatType;
        let green: Self::FloatType;
        let blue: Self::FloatType;

        let half = Self::half(sne.im() - sne.re());

        if clamped_value <= half {
            let t = (clamped_value - sne.re()) / (half - sne.re());

            red = (Self::float_val_1() - t) * Self::float_val_255();
            green = t * Self::float_val_255();
            blue = Self::float_val_0();
        } else {
            let t = (clamped_value - half) / (half - sne.re());

            red = Self::float_val_0();
            green = (Self::float_val_1() - t) * Self::float_val_255();
            blue = t * Self::float_val_255();
        }

        Color::rgb(
            Self::round_to_u8(red),
            Self::round_to_u8(green),
            Self::round_to_u8(blue),
        )
    }
}

impl FractalComplex for Complex<f32> {
    type FloatType = f32;

    #[inline(always)]
    fn re(self) -> Self::FloatType {
        self.re
    }

    #[inline(always)]
    fn im(self) -> Self::FloatType {
        self.im
    }

    #[inline]
    fn new(re: Self::FloatType, im: Self::FloatType) -> Self {
        Self { re, im }
    }

    #[inline(always)]
    fn float_val_0() -> Self::FloatType {
        0.0
    }

    #[inline(always)]
    fn float_val_1() -> Self::FloatType {
        1.0
    }

    #[inline(always)]
    fn float_val_100() -> Self::FloatType {
        100.0
    }

    #[inline(always)]
    fn float_val_255() -> Self::FloatType {
        255.0
    }

    #[inline]
    fn fsq_add(&mut self, c: Self) {
        (self.re, self.im) = (
            self.re * self.re - self.im * self.im + c.re,
            2.0 * self.re * self.im + c.im,
        );
    }

    #[inline]
    fn half(n: Self::FloatType) -> Self::FloatType {
        n / 2.0
    }

    fn clamp(n: Self::FloatType, min: Self::FloatType, max: Self::FloatType) -> Self::FloatType {
        n.clamp(min, max)
    }

    fn round_to_u8(n: Self::FloatType) -> u8 {
        n.round() as u8
    }

    #[inline]
    fn distance_origin(self) -> Self::FloatType {
        self.re.abs() + self.im.abs()
    }

    #[inline]
    fn from_cmplx(val: &rug::Complex) -> Self {
        Complex {
            re: val.real().to_f32(),
            im: val.imag().to_f32(),
        }
    }

    #[inline]
    fn from_u32_pair(val: (u32, u32)) -> Self {
        Complex {
            re: val.0 as f32,
            im: val.1 as f32,
        }
    }
}
