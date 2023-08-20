use std::ops::{Range, RangeInclusive};

#[derive(Clone, Debug)]
pub struct TimeCursor {
    pub time_range: Range<f32>,
    pub min_size:   f32,
}
impl Default for TimeCursor {
    fn default() -> Self {
        Self {
            time_range: Default::default(),
            min_size:   1.0 / 48000.0,
        }
    }
}
impl TimeCursor {
    pub fn try_initialize(&mut self, range: Range<f32>) {
        if self.time_range.is_empty() {
            self.time_range = range;
        }
    }

    pub fn clamp(&self, range: Range<f32>) -> Range<f32> {
        let start = range
            .start
            .clamp(self.time_range.start, self.time_range.end);
        let end = range.end.clamp(self.time_range.start, self.time_range.end);

        start..end
    }

    pub fn clamp_with_offset(&self, range: Range<f32>, offset: f32) -> Range<f32> {
        let r = self.clamp((range.start + offset)..(range.end + offset));
        (r.start - offset)..(r.end - offset)
    }

    pub fn overlaps(&self, range: Range<f32>) -> bool {
        range.start < self.time_range.end && range.end > self.time_range.start
    }

    pub fn time_range_inclusive(&self) -> RangeInclusive<f32> {
        self.time_range.start..=self.time_range.end
    }

    pub fn time_range_rect(&self, rect: egui::Rect, time_range: Range<f32>) -> egui::Rect {
        let x_start = egui::remap(
            time_range.start,
            self.time_range_inclusive(),
            rect.x_range(),
        );
        let x_end = egui::remap(time_range.end, self.time_range_inclusive(), rect.x_range());

        egui::Rect::from_x_y_ranges(x_start..=x_end, rect.y_range())
    }

    pub fn time_range_rect_clamped(&self, rect: egui::Rect, time_range: Range<f32>) -> egui::Rect {
        let x_start = egui::remap_clamp(
            time_range.start,
            self.time_range_inclusive(),
            rect.x_range(),
        );
        let x_end = egui::remap_clamp(time_range.end, self.time_range_inclusive(), rect.x_range());

        egui::Rect::from_x_y_ranges(x_start..=x_end, rect.y_range())
    }

    pub fn zoom_to(&mut self, to: f32, amount: f32) {
        let factor = 2.0f32.powf(amount);
        self.time_range.start = ((self.time_range.start - to) * factor) + to;
        self.time_range.end = ((self.time_range.end - to) * factor) + to;

        // Clamp range to prevent collapsing to duration = 0
        self.time_range.start = self.time_range.start.min(to - self.min_size / 2.0);
        self.time_range.end = self.time_range.end.max(to + self.min_size / 2.0);
    }

    pub fn shift(&mut self, by: f32) {
        self.time_range.start += by;
        self.time_range.end += by;
    }

    pub fn move_into_range(&mut self, range: Range<f32>) {
        assert!(range.start < range.end);

        if self.time_range.start > range.start {
            self.shift(range.start - self.time_range.start);
            self.time_range.end = self
                .time_range
                .end
                .clamp(self.time_range.start + self.min_size, range.end);
        }
        else if self.time_range.end < range.end {
            self.shift(range.end - self.time_range.end);
            self.time_range.start = self
                .time_range
                .start
                .clamp(range.start, self.time_range.end - self.min_size);
        }
    }
}
impl From<std::ops::Range<f32>> for TimeCursor {
    fn from(time_range: std::ops::Range<f32>) -> Self {
        Self {
            time_range,
            ..Default::default()
        }
    }
}
