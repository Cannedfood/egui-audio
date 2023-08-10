use std::{f32::consts::TAU, ops::RangeInclusive};

pub struct Knob<'a> {
    pub value: &'a mut f32,
    pub default: f32,
    pub range: RangeInclusive<f32>,
    pub size: f32,
}
impl<'a> Knob<'a> {
    pub fn pan(value: &'a mut f32) -> Self {
        Self {
            value,
            default: 0.5,
            range: 0.0..=1.0,
            size: 50.0,
        }
    }
}
impl<'a> egui::Widget for Knob<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (rect, mut res) = ui.allocate_at_least(
            egui::vec2(self.size, self.size),
            egui::Sense::click_and_drag(),
        );

        let visuals = ui.style().interact(&res);

        let center = rect.center();
        let outer_radius = rect.size().min_elem() / 2.0;
        let inner_radius = rect.size().min_elem() / 3.0;

        ui.painter()
            .circle(center, outer_radius, visuals.bg_fill, visuals.bg_stroke);
        ui.painter().circle_filled(
            center + egui::vec2(0.0, -2.0),
            inner_radius,
            visuals.weak_bg_fill,
        );
        ui.painter().circle(
            center + egui::vec2(0.0, -2.0),
            inner_radius,
            visuals.bg_fill,
            visuals.fg_stroke,
        );

        // Handle input
        if res.double_clicked() {
            *self.value = self.default;
            res.mark_changed();
        }
        else if res.dragged() {
            let drag_position = res.interact_pointer_pos().unwrap() + res.drag_delta();
            let drag_direction = drag_position - center;

            if drag_direction.length_sq() > 5.0 {
                let angle = drag_direction.y.atan2(drag_direction.x);
                *self.value = remap_range(angle, -TAU..=TAU, self.range.clone());
                res.mark_changed();
            }
        }

        // Draw value line
        let angle = remap_range(*self.value, self.range.clone(), -TAU..=TAU);
        let dir = egui::vec2(angle.cos(), angle.sin());
        let line_start = center + dir * (inner_radius + 2.0);
        let line_end = center + dir * outer_radius;
        ui.painter()
            .line_segment([line_start, line_end], visuals.fg_stroke);

        res
    }
}

fn remap_range(value: f32, from_range: RangeInclusive<f32>, to_range: RangeInclusive<f32>) -> f32 {
    let from_min = *from_range.start();
    let from_max = *from_range.end();
    let to_min = *to_range.start();
    let to_max = *to_range.end();

    (value - from_min) * (to_max - to_min) / (from_max - from_min) + to_min
}
