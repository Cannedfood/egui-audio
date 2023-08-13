use std::ops::RangeInclusive;

use egui::Vec2;

use crate::util::{from_db_deadzone, remap_range, to_db_deadzone};

#[derive(Debug)]
pub struct Fader<'a> {
    pub value: &'a mut f32,
    pub default: f32,
    pub range: RangeInclusive<f32>,
    pub size: Vec2,
    pub convert_to_db: bool,
}
impl<'a> Fader<'a> {
    pub fn volume(value: &'a mut f32) -> Self {
        Self {
            value,
            default: 0.5,
            range: 0.0..=1.0,
            size: Vec2::new(50.0, 150.0),
            convert_to_db: true,
        }
    }

    pub fn with_default(self, default: f32) -> Self {
        Self { default, ..self }
    }

    pub fn with_range(self, range: RangeInclusive<f32>) -> Self {
        Self { range, ..self }
    }

    pub fn with_size(self, size: Vec2) -> Self {
        Self { size, ..self }
    }
}
impl<'a> egui::Widget for Fader<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
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
        ui.painter().text(
            handle_rect.center(),
            egui::Align2::CENTER_CENTER,
            if self.convert_to_db {
                format!("{:.1}db", self.value)
            }
            else {
                format!("{:.1}", self.value)
            },
            egui::FontId::proportional(handle_rect.height() - 4.0),
            visuals.text_color(),
        );

        if self.convert_to_db {
            *self.value = from_db_deadzone(*self.value, *self.range.start());
        }

        res
    }
}
