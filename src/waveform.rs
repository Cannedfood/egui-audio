use egui::remap;

mod waveform_data;
mod waveform_mipmap;
mod waveform_spectrum;

pub use waveform_data::WaveformData;
pub use waveform_mipmap::WaveformMipmap;

use crate::TimeCursor;

pub struct Waveform<'a> {
    pub waveform: &'a waveform_data::WaveformData,
    pub spectrum: Option<&'a waveform_spectrum::WaveformSpectrum>,
    pub cursor:   Option<&'a mut TimeCursor>,
}
impl<'a> Waveform<'a> {
    pub fn new(data: &'a waveform_data::WaveformData) -> Self {
        Self {
            waveform: data,
            spectrum: None,
            cursor:   None,
        }
    }

    pub fn cursor(self, cursor: &'a mut TimeCursor) -> Self {
        Self {
            cursor: Some(cursor),
            ..self
        }
    }
}
impl<'a> egui::Widget for Waveform<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        // Set up parameters
        let waveform = self.waveform;

        let mut fallback_cursor = TimeCursor::from(0.0..waveform.len_seconds());
        let cursor: &mut TimeCursor = self.cursor.unwrap_or(&mut fallback_cursor);
        cursor.try_initialize(0.0..waveform.len_seconds());

        let (rect, response) = ui.allocate_at_least(
            egui::vec2(ui.available_width(), 200.0),
            egui::Sense::click_and_drag(),
        );
        let painter = ui.painter_at(rect);

        if response.dragged_by(egui::PointerButton::Middle) {
            let dx1 = ui.input(|i| i.pointer.delta().x);
            let dx = dx1 / rect.width() * (cursor.time_range.end - cursor.time_range.start);

            cursor.shift(-dx);
        }

        if let Some(hover_pos) = response.hover_pos() {
            let scrolled = ui.input_mut(|i| std::mem::replace(&mut i.scroll_delta.y, 0.0)) / 100.0;
            let zoom_target = remap(hover_pos.x, rect.x_range(), cursor.time_range_inclusive());
            cursor.zoom_to(zoom_target, scrolled);
        }

        // cursor.move_into_range(0.0..waveform.len_seconds());

        painter.rect(
            rect,
            ui.style().visuals.widgets.noninteractive.rounding,
            ui.style().visuals.widgets.noninteractive.bg_fill,
            ui.style().visuals.widgets.noninteractive.bg_stroke,
        );

        {
            // Draw waveform item
            let start_x = egui::remap(0.0, cursor.time_range_inclusive(), rect.x_range());
            let end_x = egui::remap(
                waveform.len_seconds(),
                cursor.time_range_inclusive(),
                rect.x_range(),
            );
            painter.rect_filled(
                egui::Rect::from_x_y_ranges(start_x..=end_x, rect.y_range()),
                3.0,
                ui.style().visuals.extreme_bg_color,
            );
        }

        painter.add(self.waveform.get_outline(
            10.0, // Pixels per point
            rect,
            cursor.time_range.clone(),
            ui.style().visuals.widgets.noninteractive.fg_stroke,
        ));

        response
    }
}
