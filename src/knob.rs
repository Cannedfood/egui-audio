use std::{f32::consts::PI, ops::RangeInclusive};

use crate::util::remap_range;

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
            range: -1.0..=1.0,
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

        let offset_3d = egui::vec2(0.0, -2.0);
        let center = rect.center();
        let outer_radius = rect.size().min_elem() / 2.0;
        let inner_radius = rect.size().min_elem() / 3.0;

        ui.painter()
            .circle(center, outer_radius, visuals.bg_fill, visuals.bg_stroke);
        ui.painter()
            .circle_filled(center + offset_3d, inner_radius, visuals.weak_bg_fill);
        ui.painter().circle(
            center + offset_3d,
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

            if drag_direction.length() > inner_radius {
                let angle = drag_direction.y.atan2(drag_direction.x);
                *self.value = remap_range(angle, -PI..=PI, self.range.clone());
                res.mark_changed();
            }
        }

        // Draw value line
        let angle = remap_range(*self.value, self.range.clone(), -PI..=PI);
        let dir = egui::vec2(angle.cos(), angle.sin());
        let line_start = center + dir * (inner_radius + offset_3d.abs().max_elem());
        let line_end = center + dir * outer_radius;
        ui.painter()
            .line_segment([line_start, line_end], visuals.fg_stroke);

        ui.painter().text(
            center + offset_3d,
            egui::Align2::CENTER_CENTER,
            format!("{:.2}", *self.value),
            egui::FontId::proportional(inner_radius * 0.5),
            visuals.text_color(),
        );

        res
    }
}