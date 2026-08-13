use tokimu_math_study_provider_backed::Vec3;

fn bits(values: Vec3) -> [u32; 3] {
    values.to_array().map(f32::to_bits)
}

fn main() {
    let nan = f32::from_bits(0x7fc0_0042);
    let left = Vec3::new(nan, 1.0, -2.0);
    let right = Vec3::new(4.0, nan, -3.0);

    println!(
        "left_min_right={:08x?}; right_min_left={:08x?}; left_max_right={:08x?}; right_max_left={:08x?}",
        bits(left.min(right)),
        bits(right.min(left)),
        bits(left.max(right)),
        bits(right.max(left)),
    );
}
