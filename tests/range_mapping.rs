use mandelbread::map_range;

const EPSILON: f32 = 0.000001; // A small constant for floating-point comparisons

#[test]
fn test_basic_forward_mapping() {
    // Map 50 from [0, 100] to [0, 10] -> should be 5.0
    assert!((map_range(0, 100, 0.0, 10.0, 50) - 5.0).abs() < EPSILON);

    // Map 0 from [0, 100] to [0, 10] -> should be 0.0 (inr_x to outr_x)
    assert!((map_range(0, 100, 0.0, 10.0, 0) - 0.0).abs() < EPSILON);

    // Map 100 from [0, 100] to [0, 10] -> should be 10.0 (inr_y to outr_y)
    assert!((map_range(0, 100, 0.0, 10.0, 100) - 10.0).abs() < EPSILON);

    // Map 25 from [0, 100] to [0, 10] -> should be 2.5
    assert!((map_range(0, 100, 0.0, 10.0, 25) - 2.5).abs() < EPSILON);

    // Map 75 from [0, 100] to [0, 10] -> should be 7.5
    assert!((map_range(0, 100, 0.0, 10.0, 75) - 7.5).abs() < EPSILON);
}

#[test]
fn test_reversed_output_mapping() {
    // Map 50 from [0, 100] to [10.0, 0.0] -> should be 5.0
    assert!((map_range(0, 100, 10.0, 0.0, 50) - 5.0).abs() < EPSILON);

    // Map 0 from [0, 100] to [10.0, 0.0] -> should be 10.0 (inr_x to outr_x)
    assert!((map_range(0, 100, 10.0, 0.0, 0) - 10.0).abs() < EPSILON);

    // Map 100 from [0, 100] to [10.0, 0.0] -> should be 0.0 (inr_y to outr_y)
    assert!((map_range(0, 100, 10.0, 0.0, 100) - 0.0).abs() < EPSILON);

    // Map 25 from [0, 100] to [10.0, 0.0] -> should be 7.5
    assert!((map_range(0, 100, 10.0, 0.0, 25) - 7.5).abs() < EPSILON);
}

#[test]
fn test_mapping_with_negative_input_range() {
    // Map 0 from [-10, 10] to [0.0, 100.0] -> should be 50.0
    assert!((map_range(-10, 10, 0.0, 100.0, 0) - 50.0).abs() < EPSILON);

    // Map -10 from [-10, 10] to [0.0, 100.0] -> should be 0.0
    assert!((map_range(-10, 10, 0.0, 100.0, -10) - 0.0).abs() < EPSILON);

    // Map 10 from [-10, 10] to [0.0, 100.0] -> should be 100.0
    assert!((map_range(-10, 10, 0.0, 100.0, 10) - 100.0).abs() < EPSILON);

    // Map -5 from [-10, 10] to [0.0, 100.0] -> should be 25.0
    assert!((map_range(-10, 10, 0.0, 100.0, -5) - 25.0).abs() < EPSILON);
}

#[test]
fn test_mapping_with_negative_output_range() {
    // Map 50 from [0, 100] to [-10.0, 10.0] -> should be 0.0
    assert!((map_range(0, 100, -10.0, 10.0, 50) - 0.0).abs() < EPSILON);

    // Map 0 from [0, 100] to [-10.0, 10.0] -> should be -10.0
    assert!((map_range(0, 100, -10.0, 10.0, 0) - -10.0).abs() < EPSILON);

    // Map 100 from [0, 100] to [-10.0, 10.0] -> should be 10.0
    assert!((map_range(0, 100, -10.0, 10.0, 100) - 10.0).abs() < EPSILON);

    // Map 25 from [0, 100] to [-10.0, 10.0] -> should be -5.0
    assert!((map_range(0, 100, -10.0, 10.0, 25) - -5.0).abs() < EPSILON);
}

#[test]
fn test_mapping_with_both_negative_ranges() {
    // Map -75 from [-100, -50] to [-1000.0, -500.0] -> should be -750.0
    assert!((map_range(-100, -50, -1000.0, -500.0, -75) - -750.0).abs() < EPSILON);

    // Map -100 from [-100, -50] to [-1000.0, -500.0] -> should be -1000.0
    assert!((map_range(-100, -50, -1000.0, -500.0, -100) - -1000.0).abs() < EPSILON);

    // Map -50 from [-100, -50] to [-1000.0, -500.0] -> should be -500.0
    assert!((map_range(-100, -50, -1000.0, -500.0, -50) - -500.0).abs() < EPSILON);
}

#[test]
fn test_mapping_with_large_ranges() {
    // Map 50000 from [0, 100000] to [0.0, 1.0] -> should be 0.5
    assert!((map_range(0, 100000, 0.0, 1.0, 50000) - 0.5).abs() < EPSILON);

    // Map 1000 from [0, 2000] to [100.0, 200.0] -> should be 150.0
    assert!((map_range(0, 2000, 100.0, 200.0, 1000) - 150.0).abs() < EPSILON);
}

#[test]
fn test_mapping_with_fractional_results() {
    // Map 1 from [0, 3] to [0.0, 1.0] -> should be 0.33333...
    assert!((map_range(0, 3, 0.0, 1.0, 1) - 0.33333334).abs() < EPSILON);

    // Map 10 from [0, 20] to [0.0, 7.0] -> should be 3.5
    assert!((map_range(0, 20, 0.0, 7.0, 10) - 3.5).abs() < EPSILON);
}
