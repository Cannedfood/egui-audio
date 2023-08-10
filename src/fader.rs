use std::ops::RangeInclusive;

#[derive(Debug)]
pub struct Fader<'a> {
    pub value:   &'a mut f32,
    pub default: f32,
    pub range:   RangeInclusive<f32>,
    pub scale:   f32,
}
impl<'a> Fader<'a> {
    pub fn volume(value: &'a mut f32) -> Self {
        Self {
            value,
            default: 0.5,
            range: 0.0..=1.0,
            scale: 1.0,
        }
    }
}
impl<'a> egui::Widget for Fader<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (rect, mut res) = ui.allocate_at_least(
            egui::vec2(50.0, 150.0) * self.scale,
            egui::Sense::click_and_drag(),
        );

        // Handle input
        if res.double_clicked() {
            *self.value = self.default;
            res.mark_changed();
        }
        else if res.dragged() {
            let delta = res.drag_delta().y / rect.height();
            *self.value = (*self.value - delta).clamp(*self.range.start(), *self.range.end());
            res.mark_changed();
        }

        let visuals = ui.style().interact(&res);

        ui.painter().rect_filled(
            egui::Rect::from_center_size(rect.center(), egui::vec2(5.0, rect.height())),
            visuals.rounding,
            ui.style().visuals.extreme_bg_color,
        );

        let handle_height = 20.0;
        let handle_rect = egui::Rect::from_min_size(
            rect.min + egui::vec2(0.0, (1.0 - *self.value) * (rect.height() - handle_height)),
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
            format!("{:.1}", self.value),
            egui::FontId::proportional(handle_rect.height() - 4.0),
            visuals.text_color(),
        );

        res
    }
}
