use std::ops::Range;

use egui::{remap, vec2, Modifiers};

mod waveform_data;
mod waveform_mipmap;

pub use waveform_data::WaveformShape;
pub use waveform_mipmap::WaveformMipmap;

use crate::TimeCursor;

// Backwards compatibility
#[deprecated = "Use WaveformShape instead"]
pub type WaveformData = WaveformShape;
#[deprecated = "Use WaveformItem instead"]
pub type Entry<'a> = WaveformItem<'a>;
#[deprecated = "Use WaveformMarker instead"]
pub type Marker = WaveformMarker;

#[derive(Clone, Copy)]
pub struct WaveformItem<'a> {
    pub position: f32,
    pub gain:     f32,
    pub waveform: &'a WaveformShape,
    pub stroke:   Option<egui::Stroke>,
}
impl<'a> WaveformItem<'a> {
    pub fn new(waveform: &'a WaveformShape) -> Self {
        Self {
            position: 0.0,
            gain: 1.0,
            waveform,
            stroke: None,
        }
    }

    pub fn duration(&self) -> f32 { self.waveform.len_seconds() }

    pub fn time_range(&self) -> std::ops::Range<f32> {
        self.position..(self.duration() + self.position)
    }

    pub fn with_position(self, seconds: f32) -> Self {
        Self {
            position: seconds,
            ..self
        }
    }

    pub fn with_stroke(self, stroke: impl Into<egui::Stroke>) -> Self {
        Self {
            stroke: Some(stroke.into()),
            ..self
        }
    }

    pub fn with_gain(self, gain: f32) -> Self { Self { gain, ..self } }
}

#[derive(Debug, Clone)]
pub struct WaveformMarker {
    pub start:  f32,
    pub end:    Option<f32>,
    pub stroke: egui::Stroke,
    pub fill:   egui::Color32,
    pub text:   String,
}
impl Default for WaveformMarker {
    fn default() -> Self {
        Self {
            start:  0.0,
            end:    None,
            stroke: (1.0, egui::Color32::RED).into(),
            fill:   egui::Color32::from_rgba_unmultiplied(0x22, 0x8, 0x8, 0x22),
            text:   String::new(),
        }
    }
}
impl WaveformMarker {
    pub fn from_position(start: f32) -> Self {
        Self {
            start,
            ..Default::default()
        }
    }

    pub fn from_start_end(start: f32, end: f32) -> Self {
        Self {
            start,
            end: Some(end),
            ..Default::default()
        }
    }

    pub fn from_range(range: Range<f32>) -> Self { Self::from_start_end(range.start, range.end) }

    pub fn from_tuple(range: (f32, f32)) -> Self { Self::from_start_end(range.0, range.1) }

    pub fn with_label(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    pub fn with_color(mut self, color: impl Into<egui::Color32>) -> Self {
        let c: egui::Color32 = color.into();
        self.stroke.color = c;
        self.fill = egui::Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), 0x1);
        self
    }

    pub fn with_fill(mut self, color: impl Into<egui::Color32>) -> Self {
        self.fill = color.into();
        self
    }

    pub fn with_stroke(mut self, stroke: impl Into<egui::Stroke>) -> Self {
        self.stroke = stroke.into();
        self
    }

    pub fn end(&self) -> f32 { self.end.unwrap_or(self.start) }

    fn time_range(&self) -> std::ops::Range<f32> { self.start..self.end() }
}

#[derive(Default, Clone, Copy)]
pub struct WaveformResponse {
    pub clicked: Option<egui::Vec2>,
    pub dragged_to: Option<egui::Vec2>,
    pub dragged_delta: Option<egui::Vec2>,
}

pub struct Waveform<'a> {
    pub data: Vec<WaveformItem<'a>>,
    pub markers: Vec<WaveformMarker>,
    pub cursor: Option<&'a mut TimeCursor>,
    pub pixels_per_point: f32,
    pub zoom_modifier: egui::Modifiers,
    pub height: f32,
}
impl<'a> Default for Waveform<'a> {
    fn default() -> Self {
        Self {
            data: Vec::new(),
            markers: Vec::new(),
            cursor: None,
            pixels_per_point: 10.0,
            zoom_modifier: Modifiers::NONE,
            height: 200.0,
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

    pub fn zoom_modifier(self, zoom_modifier: egui::Modifiers) -> Self {
        Self {
            zoom_modifier,
            ..self
        }
    }

    pub fn cursor(self, cursor: &'a mut TimeCursor) -> Self {
        Self {
            cursor: Some(cursor),
            ..self
        }
    }

    pub fn entry(mut self, e: WaveformItem<'a>) -> Self {
        self.data.push(e);
        self
    }

    pub fn entries(mut self, e: impl IntoIterator<Item = WaveformItem<'a>>) -> Self {
        self.data.extend(e);
        self
    }

    pub fn marker(mut self, m: WaveformMarker) -> Self {
        self.markers.push(m);
        self
    }

    pub fn markers(mut self, m: impl IntoIterator<Item = WaveformMarker>) -> Self {
        self.markers.extend(m);
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    pub fn show(self, ui: &mut egui::Ui) -> egui::InnerResponse<WaveformResponse> {
        // Set up parameters
        let entries_range = Iterator::chain(
            self.data.iter().map(WaveformItem::time_range),
            self.markers.iter().map(WaveformMarker::time_range),
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
            egui::vec2(ui.available_width(), self.height),
            egui::Sense::click_and_drag(),
        );
        let painter = ui.painter_at(rect);

        if response.dragged_by(egui::PointerButton::Middle) {
            let dx1 = ui.input(|i| i.pointer.delta().x);
            let dx = dx1 / rect.width() * (cursor.time_range.end - cursor.time_range.start);

            cursor.shift(-dx);
        }

        if let Some(hover_pos) = response.hover_pos() {
            let zoomed = ui.input(|i| i.zoom_delta());
            let zoom_target = remap(hover_pos.x, rect.x_range(), cursor.time_range_inclusive());
            cursor.zoom_to(zoom_target, zoomed);
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
                ui.painter_at(entry_rect).add(
                    e.waveform.get_outline(
                        self.pixels_per_point,
                        entry_rect,
                        cursor.clamp_with_offset(0.0..e.duration(), e.position),
                        e.stroke
                            .unwrap_or(ui.style().visuals.widgets.noninteractive.fg_stroke),
                        e.gain,
                    ),
                );
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

        let mut ret = WaveformResponse::default();
        if response.clicked() {
            if let Some(p) = response.interact_pointer_pos() {
                let y = egui::remap(p.y, rect.y_range(), -1.0..=1.0);
                let x = egui::remap(p.x, rect.x_range(), cursor.time_range_inclusive());
                ret.clicked = Some(egui::vec2(x, y));
            }
        }
        if response.dragged_by(egui::PointerButton::Primary) {
            if let Some(p) = response.interact_pointer_pos() {
                let y = egui::remap(p.y, rect.y_range(), -1.0..=1.0);
                let x = egui::remap(p.x, rect.x_range(), cursor.time_range_inclusive());
                ret.dragged_to = Some(egui::vec2(x, y));
                ret.dragged_delta = Some(
                    response.drag_delta() / rect.size()
                        * vec2(1.0, cursor.time_range.end - cursor.time_range.start),
                );
            }
        }
        egui::InnerResponse::new(ret, response)
    }
}
