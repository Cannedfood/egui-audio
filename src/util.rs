use std::ops::RangeInclusive;

pub fn remap_range(
    value: f32,
    from_range: RangeInclusive<f32>,
    to_range: RangeInclusive<f32>,
) -> f32 {
    let from_min = *from_range.start();
    let from_max = *from_range.end();
    let to_min = *to_range.start();
    let to_max = *to_range.end();

    (value - from_min) * (to_max - to_min) / (from_max - from_min) + to_min
}
