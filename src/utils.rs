use num::{
    Complex, Float, FromPrimitive, One, ToPrimitive, Zero,
    complex::{Complex32, ComplexFloat},
};
use sfml::graphics::Color;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct ConversionColor(pub u8, pub u8, pub u8);

// TODO understand this shit
pub fn distance_gradient<T, const START: u32, const END: u32>(value: T) -> ConversionColor
where
    T: Float + FromPrimitive + ToPrimitive + Zero + One,
{
    let start = FromPrimitive::from_u32(START).unwrap();
    let end = FromPrimitive::from_u32(END).unwrap();

    assert!(end >= start);

    // Clamp the input value to the valid range [0.0, 100.0]
    let clamped_value = value.clamp(start, end);

    let red: T;
    let green: T;
    let blue: T;

    let t_zero = Zero::zero();
    let t_one: T = One::one();
    let t_two = FromPrimitive::from_f32(2.0).unwrap();
    let t_255 = FromPrimitive::from_f32(255.0).unwrap();

    let half = (end + start) / t_two;

    if clamped_value <= half {
        let t = (clamped_value - start) / (half - start); // t will be 0.0 at 0.0, 1.0 at 50.0

        red = (t_one - t) * t_255; // Red decreases from 255 to 0
        green = t * t_255; // Green increases from 0 to 255
        blue = t_zero; // Blue remains 0
    } else {
        let t = (clamped_value - half) / (half - start); // t will be 0.0 at 50.0, 1.0 at 100.0

        red = Zero::zero(); // Red remains 0
        green = (t_one - t) * t_255; // Green decreases from 255 to 0
        blue = t * t_255; // Blue increases from 0 to 255
    }

    // Convert f32 components to u8, rounding and clamping to 0-255
    let final_red = red.round().to_u8().unwrap();
    let final_green = green.round().to_u8().unwrap();
    let final_blue = blue.round().to_u8().unwrap();

    ConversionColor(final_red, final_green, final_blue)
}

pub fn distance_gradient_f32<const START: u32, const END: u32>(value: f32) -> ConversionColor {
    let start = START as f32;
    let end = END as f32;

    assert!(end >= start);

    // Clamp the input value to the valid range [0.0, 100.0]
    let clamped_value = value.clamp(start, end);

    let red: f32;
    let green: f32;
    let blue: f32;

    let half = (end + start) / 2.0;

    if clamped_value <= half {
        let t = (clamped_value - start) / (half - start); // t will be 0.0 at 0.0, 1.0 at 50.0

        red = (1.0 - t) * 255.0; // Red decreases from 255 to 0
        green = t * 255.0; // Green increases from 0 to 255
        blue = 0.0; // Blue remains 0
    } else {
        let t = (clamped_value - half) / (half - start); // t will be 0.0 at 50.0, 1.0 at 100.0

        red = 0.0; // Red remains 0
        green = (1.0 - t) * 255.0; // Green decreases from 255 to 0
        blue = t * 255.0; // Blue increases from 0 to 255
    }

    // Convert f32 components to u8, rounding and clamping to 0-255
    let final_red = red.round() as u8;
    let final_green = green.round() as u8;
    let final_blue = blue.round() as u8;

    ConversionColor(final_red, final_green, final_blue)
}

impl Into<sfml::graphics::Color> for ConversionColor {
    #[inline]
    fn into(self) -> sfml::graphics::Color {
        Color::rgb(self.0, self.1, self.2)
    }
}
pub fn map_between<T>(
    res: (u32, u32),
    start: Complex<T>,
    end: Complex<T>,
    value: (u32, u32),
) -> Complex<T>
where
    T: FromPrimitive + Float,
{
    let re = <T as FromPrimitive>::from_u32(value.0).unwrap() * (end.re - start.re)
        / FromPrimitive::from_u32(res.0).unwrap()
        + start.re;
    let im = <T as FromPrimitive>::from_u32(value.1).unwrap() * (end.im - start.im)
        / FromPrimitive::from_u32(res.1).unwrap()
        + start.im;
    Complex { re, im }
}
pub fn map_between_f32(
    res: (u32, u32),
    start: Complex32,
    end: Complex32,
    value: (u32, u32),
) -> Complex32 {
    let re = value.0 as f32 * (end.re() - start.re()) / res.0 as f32 + start.re();
    let im = value.1 as f32 * (end.im() - start.im()) / res.1 as f32 + start.im();
    Complex { re, im }
}

#[inline]
pub fn fsq_add<T>(slf: &mut Complex<T>, c: Complex<T>)
where
    T: Float + FromPrimitive,
{
    (slf.re, slf.im) = (
        slf.re * slf.re - slf.im * slf.im + c.re,
        <T as FromPrimitive>::from_f32(2.0).unwrap() * slf.re * slf.im + c.im,
    )
}
#[inline]
pub fn fsq_add_f32(slf: &mut Complex<f32>, c: Complex<f32>) {
    (slf.re, slf.im) = (
        slf.re * slf.re - slf.im * slf.im + c.re,
        2.0 * slf.re * slf.im + c.im,
    )
}
