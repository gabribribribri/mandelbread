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
