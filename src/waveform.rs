use std::ops::Range;

use egui::remap;

mod waveform_data;
mod waveform_mipmap;

pub use waveform_data::WaveformData;
pub use waveform_mipmap::WaveformMipmap;

use crate::TimeCursor;

pub struct Entry<'a> {
    pub offset:   f32,
    pub waveform: &'a WaveformData,
    pub stroke:   egui::Stroke,
}
impl<'a> Entry<'a> {
    pub fn duration(&self) -> f32 {
        self.waveform.len_seconds()
    }

    pub fn start(&self) -> f32 {
        self.offset
    }

    pub fn end(&self) -> f32 {
        self.offset + self.duration()
    }

    pub fn time_range(&self) -> std::ops::Range<f32> {
        self.start()..self.end()
    }
}
impl<'a> From<&'a WaveformData> for Entry<'a> {
    fn from(waveform: &'a WaveformData) -> Self {
        Self {
            offset: 0.0,
            waveform,
            stroke: (1.0, egui::Color32::WHITE).into(),
        }
    }
}

pub struct Marker {
    pub start: f32,
    pub end: Option<f32>,
    pub stroke: egui::Stroke,
    pub fill: egui::Color32,
    pub text: String,
}
impl Default for Marker {
    fn default() -> Self {
        Self {
            start: 0.0,
            end: None,
            stroke: (1.0, egui::Color32::RED).into(),
            fill: egui::Color32::from_rgba_unmultiplied(0x22, 0x8, 0x8, 0x22),
            text: String::new(),
        }
    }
}
impl Marker {
    pub fn from_position(start: f32) -> Self {
        Self {
            start,
            ..Default::default()
        }
    }

    pub fn from_range(range: Range<f32>) -> Self {
        Self {
            start: range.start,
            end: Some(range.end),
            ..Default::default()
        }
    }

    pub fn label(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    pub fn end(&self) -> f32 {
        self.end.unwrap_or(self.start)
    }

    fn time_range(&self) -> std::ops::Range<f32> {
        self.start..self.end()
    }
}

pub struct Waveform<'a> {
    pub data: Vec<Entry<'a>>,
    pub markers: Vec<Marker>,
    pub cursor: Option<&'a mut TimeCursor>,
    pub pixels_per_point: f32,
}
impl<'a> Default for Waveform<'a> {
    fn default() -> Self {
        Self {
            data: Vec::new(),
            markers: Vec::new(),
            cursor: None,
            pixels_per_point: 10.0,
        }
    }
}
impl<'a> Waveform<'a> {
    pub fn pixels_per_point(self, pixels_per_point: f32) -> Self {
        Self {
            pixels_per_point,
            ..self
        }
    }

    pub fn cursor(self, cursor: &'a mut TimeCursor) -> Self {
        Self {
            cursor: Some(cursor),
            ..self
        }
    }

    pub fn entry(mut self, e: Entry<'a>) -> Self {
        self.data.push(e);
        self
    }

    pub fn entries(mut self, e: impl IntoIterator<Item = Entry<'a>>) -> Self {
        self.data.extend(e);
        self
    }

    pub fn marker(mut self, m: Marker) -> Self {
        self.markers.push(m);
        self
    }

    pub fn markers(mut self, m: impl IntoIterator<Item = Marker>) -> Self {
        self.markers.extend(m);
        self
    }
}

impl<'a> egui::Widget for Waveform<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        // Set up parameters
        let entries_range = Iterator::chain(
            self.data.iter().map(|e| e.time_range()),
            self.markers.iter().map(|m| m.time_range()),
        )
        .fold(None, |a: Option<std::ops::Range<f32>>, b| {
            Some(match a {
                None => b,
                Some(a) => f32::min(a.start, b.start)..f32::max(a.end, b.end),
            })
        })
        .unwrap_or(0.0..1.0);

        let mut fallback_cursor = TimeCursor::from(entries_range.clone());
        let cursor: &mut TimeCursor = self.cursor.unwrap_or(&mut fallback_cursor);
        cursor.try_initialize(entries_range.clone());

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

        // Draw entry backgrounds
        for e in self.data.iter() {
            painter.rect_filled(
                cursor.time_range_rect(rect, e.time_range()), // Don't use the _clamped variant - we want the rounding to be correct, the renderer will handle clipping
                3.0,
                ui.style().visuals.extreme_bg_color,
            );
        }

        // Draw entry waveforms
        for e in self.data.iter() {
            if cursor.overlaps(e.time_range()) {
                let entry_rect = cursor.time_range_rect_clamped(rect, e.time_range());
                ui.painter_at(entry_rect).add(e.waveform.get_outline(
                    self.pixels_per_point,
                    entry_rect,
                    cursor.clamp(e.time_range()),
                    ui.style().visuals.widgets.noninteractive.fg_stroke,
                ));
            }
        }

        // Draw markers
        for m in self.markers.iter() {
            if cursor.overlaps(m.time_range()) {
                let rect = cursor.time_range_rect(rect, m.time_range());
                painter.rect_filled(rect, 0.0, m.fill);
                painter.line_segment([rect.left_top(), rect.left_bottom()], m.stroke);
                painter.line_segment([rect.right_top(), rect.right_bottom()], m.stroke);

                if !m.text.is_empty() {
                    painter.text(
                        rect.left_top(),
                        egui::Align2::LEFT_TOP,
                        &m.text,
                        egui::FontId::proportional(10.0),
                        egui::Color32::WHITE,
                    );
                }
            }
        }

        response
    }
}
