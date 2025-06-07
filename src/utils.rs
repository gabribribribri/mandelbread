use sfml::graphics::Color;

// TODO make this function generic and cool
pub fn map_range(inr_x: i32, inr_y: i32, outr_x: f32, outr_y: f32, value: i32) -> f32 {
    assert!(inr_x <= value && value <= inr_y);
    assert!(inr_x != inr_y);
    assert!(outr_x != outr_y);

    let inr_x_f32 = inr_x as f32;
    let inr_y_f32 = inr_y as f32;
    let value_f32 = value as f32;

    (value_f32 - inr_x_f32) * (outr_y - outr_x) / (inr_y_f32 - inr_x_f32) + outr_x
}

// TODO understand this shit
pub fn distance_gradient<const START: u32, const END: u32>(value: f32) -> (u8, u8, u8) {
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

    (final_red, final_green, final_blue)
}

pub fn tuple_to_sfml_color(t: (u8, u8, u8)) -> Color {
    Color::rgb(t.0, t.1, t.2)
}
