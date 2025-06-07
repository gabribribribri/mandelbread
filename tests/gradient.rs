use mandelbread::utils::distance_gradient;
#[test]
fn test_gradient_start_red() {
    // Value 0.0 should be pure Red (255, 0, 0)
    assert_eq!(distance_gradient::<0, 100>(0.0), (255, 0, 0));
}

#[test]
fn test_gradient_midpoint_green() {
    // Value 50.0 should be pure Green (0, 255, 0)
    assert_eq!(distance_gradient::<0, 100>(50.0), (0, 255, 0));
}

#[test]
fn test_gradient_end_blue() {
    // Value 100.0 should be pure Blue (0, 0, 255)
    assert_eq!(distance_gradient::<0, 100>(100.0), (0, 0, 255));
    assert_eq!(distance_gradient::<0, 1000>(1000.0), (0, 0, 255));
    assert_eq!(distance_gradient::<300, 1000>(1000.0), (0, 0, 255));
}

#[test]
fn test_first_segment_quarter_point() {
    // Value 25.0 should be 50% Red, 50% Green
    // Red: 255 * (1 - 0.5) = 127.5 -> 128
    // Green: 255 * 0.5 = 127.5 -> 128
    assert_eq!(distance_gradient::<0, 100>(25.0), (128, 128, 0));
    assert_eq!(distance_gradient::<0, 1000>(250.0), (128, 128, 0));
    assert_eq!(distance_gradient::<500, 1500>(750.0), (128, 128, 0));
}

#[test]
fn test_second_segment_quarter_point() {
    // Value 75.0 should be 50% Green, 50% Blue
    // Green: 255 * (1 - 0.5) = 127.5 -> 128
    // Blue: 255 * 0.5 = 127.5 -> 128
    assert_eq!(distance_gradient::<0, 100>(75.0), (0, 128, 128));
}

#[test]
fn test_value_just_before_midpoint() {
    // Slightly before 50.0, should be mostly green, some red
    // (1.0 - 49.9/50.0) * 255 = (1.0 - 0.998) * 255 = 0.002 * 255 = 0.51 -> 1
    // (49.9/50.0) * 255 = 0.998 * 255 = 254.49 -> 254
    assert_eq!(distance_gradient::<0, 100>(49.9), (1, 254, 0));
}

#[test]
fn test_value_just_after_midpoint() {
    // Slightly after 50.0, should be mostly green, some blue
    // ((50.1 - 50.0) / 50.0) = 0.1 / 50 = 0.002
    // Green: (1.0 - 0.002) * 255 = 0.998 * 255 = 254.49 -> 254
    // Blue: 0.002 * 255 = 0.51 -> 1
    assert_eq!(distance_gradient::<0, 100>(50.1), (0, 254, 1));
}

#[test]
fn test_clamping_below_zero() {
    // Value -10.0 should be clamped to 0.0, resulting in pure Red
    assert_eq!(distance_gradient::<0, 100>(-10.0), (255, 0, 0));
}

#[test]
fn test_clamping_above_hundred() {
    // Value 110.0 should be clamped to 100.0, resulting in pure Blue
    assert_eq!(distance_gradient::<0, 100>(110.0), (0, 0, 255));
}

#[test]
fn test_floating_point_precision() {
    // Test a value that might hit rounding exactly
    assert_eq!(distance_gradient::<0, 100>(1.0), (250, 5, 0));
    assert_eq!(distance_gradient::<0, 100>(99.0), (0, 5, 250));
}
