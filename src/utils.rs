use sfml::graphics::Color;

struct ConversionColor(u8, u8, u8);

// TODO understand this shit
pub fn distance_gradient<const START: u32, const END: u32>(value: f32) -> ConversionColor {
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
