use std::f32::consts::{PI, TAU};
use std::ops::RangeInclusive;

pub struct Knob<'a> {
    value: &'a mut f32,
    default: f32,
    range: RangeInclusive<f32>,
    size: f32,
    angle_offset: f32,
    label: Option<String>,
}
impl<'a> Knob<'a> {
    pub fn pan(value: &'a mut f32) -> Self {
        Self {
            value,
            default: 0.0,
            range: -1.0..=1.0,
            size: 50.0,
            angle_offset: -PI / 2.0, // 0.0 is at the top
            label: None,
        }
    }

    pub fn default(self, default: f32) -> Self { Self { default, ..self } }

    pub fn range(self, range: RangeInclusive<f32>) -> Self { Self { range, ..self } }

    pub fn size(self, size: f32) -> Self { Self { size, ..self } }

    pub fn angle_offset(self, angle_offset: f32) -> Self {
        Self {
            angle_offset,
            ..self
        }
    }

    pub fn label(self, label: impl Into<String>) -> Self {
        Self {
            label: Some(label.into()),
            ..self
        }
    }
}
impl<'a> egui::Widget for Knob<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.with_layout(egui::Layout::top_down_justified(egui::Align::Min), |ui| {
            ui.set_width(self.size);

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
                let start_position = res.interact_pointer_pos().unwrap() - res.drag_delta();
                let close_to_center = (start_position - center).length() < outer_radius * 0.5;

                if close_to_center {
                    let delta = res.drag_delta().y / rect.height()
                        * (*self.range.end() - *self.range.start()).abs();
                    *self.value =
                        (*self.value - delta).clamp(*self.range.start(), *self.range.end());
                    res.mark_changed();
                }
                else {
                    let drag_position = res.interact_pointer_pos().unwrap() + res.drag_delta();
                    let drag_direction = drag_position - center;

                    if drag_direction.length() > inner_radius {
                        let angle =
                            (drag_direction.y.atan2(drag_direction.x) + TAU + self.angle_offset)
                                % TAU;
                        *self.value = egui::remap(angle, 0.0..=TAU, self.range.clone());
                        res.mark_changed();
                    }
                }
            }

            // Draw value line
            let angle = egui::remap(*self.value, self.range.clone(), 0.0..=TAU);
            let dir = egui::vec2(
                (angle - self.angle_offset).cos(),
                (angle - self.angle_offset).sin(),
            );
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

            if let Some(label) = self.label {
                ui.label(label);
            }

            res
        })
        .inner
    }
}
