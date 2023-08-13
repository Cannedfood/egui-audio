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

pub fn to_db(value: f32) -> f32 {
    20.0 * value.log10()
}
pub fn from_db(value: f32) -> f32 {
    10.0f32.powf(value / 20.0)
}
pub fn from_db_deadzone(value: f32, deadzone_db: f32) -> f32 {
    if value <= deadzone_db {
        0.0
    }
    else {
        from_db(value)
    }
}
pub fn to_db_deadzone(value: f32, deadzone_db: f32) -> f32 {
    if value <= 0.0 {
        deadzone_db
    }
    else {
        to_db(value)
    }
}
