use std::ops::RangeInclusive;

use egui::Vec2;

use crate::util::{from_db_deadzone, remap_range, to_db_deadzone};

#[derive(Debug)]
pub struct Fader<'a> {
    value: &'a mut f32,
    default: f32,
    range: RangeInclusive<f32>,
    size: Vec2,
    convert_to_db: bool,
    show_value: bool,
    label: Option<String>,
}
impl<'a> Fader<'a> {
    pub fn volume(value: &'a mut f32) -> Self {
        Self {
            value,
            default: 0.5,
            range: -32.0..=0.0,
            size: Vec2::new(50.0, 150.0),
            convert_to_db: true,
            show_value: true,
            label: None,
        }
    }

    pub fn default(self, default: f32) -> Self {
        Self { default, ..self }
    }

    pub fn range(self, range: RangeInclusive<f32>) -> Self {
        Self { range, ..self }
    }

    pub fn size(self, size: Vec2) -> Self {
        Self { size, ..self }
    }

    pub fn convert_to_db(self, convert_to_db: bool) -> Self {
        Self {
            convert_to_db,
            ..self
        }
    }

    pub fn show_value(self, show_value: bool) -> Self {
        Self { show_value, ..self }
    }

    pub fn label(self, label: impl Into<String>) -> Self {
        Self {
            label: Some(label.into()),
            ..self
        }
    }
}
impl<'a> egui::Widget for Fader<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.with_layout(egui::Layout::top_down_justified(egui::Align::Min), |ui| {
            ui.set_width(self.size.x);

            if self.convert_to_db {
                *self.value = to_db_deadzone(*self.value, *self.range.start());
            }

            let (rect, mut res) = ui.allocate_at_least(self.size, egui::Sense::click_and_drag());
            let handle_height = 20.0;

            // Handle input
            if res.double_clicked() {
                *self.value = self.default;
                res.mark_changed();
            }
            else if res.dragged() {
                let delta = res.drag_delta().y / (rect.height() - handle_height)
                    * (*self.range.end() - *self.range.start());
                *self.value = (*self.value - delta).clamp(*self.range.start(), *self.range.end());
                res.mark_changed();
            }

            let visuals = ui.style().interact(&res);

            ui.painter().rect_filled(
                egui::Rect::from_center_size(rect.center(), egui::vec2(5.0, rect.height())),
                visuals.rounding,
                ui.style().visuals.extreme_bg_color,
            );

            let handle_rect = egui::Rect::from_center_size(
                rect.center_top()
                    + egui::vec2(
                        0.0,
                        remap_range(
                            *self.value,
                            self.range.clone(),
                            rect.height() - handle_height..=0.0,
                        ) + handle_height * 0.5,
                    ),
                egui::vec2(rect.width(), handle_height),
            );

            ui.painter().rect(
                handle_rect,
                visuals.rounding,
                visuals.bg_fill,
                visuals.fg_stroke,
            );

            if self.show_value {
                ui.add(
                    egui::DragValue::new(self.value)
                        .speed(0.1)
                        .clamp_range(self.range.clone())
                        .max_decimals(1)
                        .min_decimals(1)
                        .suffix("db"),
                );
            }

            if let Some(label) = self.label {
                ui.label(label);
            }

            if self.convert_to_db {
                *self.value = from_db_deadzone(*self.value, *self.range.start());
            }

            res
        })
        .inner
    }
}
