use std::{
    num::NonZeroUsize,
    ops::{Range, RangeInclusive},
    sync::{atomic::AtomicU32, Arc, RwLock},
};

use egui::remap;

#[derive(Clone, Debug)]
pub struct WaveformMipmap {
    pub points_per_second: f32,
    pub positive_peaks: Vec<egui::Vec2>,
    pub negative_peaks: Vec<egui::Vec2>,
}
impl WaveformMipmap {
    pub fn with_capacity(len: usize, points_per_second: f32) -> Self {
        Self {
            points_per_second,
            negative_peaks: Vec::with_capacity(len),
            positive_peaks: Vec::with_capacity(len),
        }
    }

    pub fn len(&self) -> usize {
        assert_eq!(self.positive_peaks.len(), self.negative_peaks.len());
        self.positive_peaks.len()
    }

    pub fn is_empty(&self) -> bool {
        assert_eq!(self.positive_peaks.len(), self.negative_peaks.len());
        self.positive_peaks.is_empty()
    }

    pub fn from_samples(data: &[f32], sample_rate: usize, shrink_factor: NonZeroUsize) -> Self {
        let shrink_factor = shrink_factor.get();

        assert!(
            shrink_factor < data.len(),
            "Shrink factor should be less than data length"
        );

        let mut result = Self::with_capacity(
            data.len().div_ceil(shrink_factor),
            sample_rate as f32 / shrink_factor as f32,
        );

        for i in (0..data.len()).step_by(shrink_factor) {
            let subrange = &data[i..(i + shrink_factor).min(data.len())];

            let (positive_peak_idx, positive_peak_value) = Self::positive_peak_in_samples(subrange);
            let (negative_peak_idx, negative_peak_value) = Self::negative_peak_in_samples(subrange);

            result.positive_peaks.push(egui::vec2(
                (i + positive_peak_idx) as f32 / sample_rate as f32,
                positive_peak_value,
            ));
            result.negative_peaks.push(egui::vec2(
                (i + negative_peak_idx) as f32 / sample_rate as f32,
                negative_peak_value,
            ));
        }

        result
    }

    pub fn shrink(&self, factor: NonZeroUsize) -> Self {
        let factor = factor.get();

        let mut result =
            Self::with_capacity(self.len() / factor, self.points_per_second / factor as f32);

        for i in (0..self.positive_peaks.len()).step_by(factor) {
            let subrange = i..(i + factor).min(self.positive_peaks.len());

            result.positive_peaks.push(Self::positive_peak_in_points(
                &self.positive_peaks[subrange.clone()],
            ));
            result.negative_peaks.push(Self::negative_peak_in_points(
                &self.negative_peaks[subrange.clone()],
            ));
        }

        result
    }

    fn positive_peak_in_samples(data: &[f32]) -> (usize, f32) {
        data.iter()
            .enumerate()
            .fold((0, f32::NEG_INFINITY), |acc, (i, &x)| {
                if x > acc.1 {
                    (i, x)
                }
                else {
                    acc
                }
            })
    }

    fn negative_peak_in_samples(data: &[f32]) -> (usize, f32) {
        data.iter()
            .enumerate()
            .fold((0, f32::INFINITY), |acc, (i, &x)| {
                if x < acc.1 {
                    (i, x)
                }
                else {
                    acc
                }
            })
    }

    fn positive_peak_in_points(data: &[egui::Vec2]) -> egui::Vec2 {
        data.iter()
            .fold(egui::vec2(0.0, f32::NEG_INFINITY), |acc, &x| {
                if x.y > acc.y {
                    x
                }
                else {
                    acc
                }
            })
    }

    fn negative_peak_in_points(data: &[egui::Vec2]) -> egui::Vec2 {
        data.iter().fold(egui::vec2(0.0, f32::INFINITY), |acc, &x| {
            if x.y < acc.y {
                x
            }
            else {
                acc
            }
        })
    }
}

static LAST_MIPMAP: AtomicU32 = AtomicU32::new(0);

pub struct WaveformData {
    pub sample_rate: usize,
    pub num_samples: usize,
    pub mipmaps: Vec<WaveformMipmap>,
}
impl WaveformData {
    pub fn calculate(samples: &[f32], sample_rate: u32) -> Self {
        let mut mipmaps = Vec::new();
        let shrink_factor: NonZeroUsize = 2.try_into().unwrap();

        mipmaps.push(WaveformMipmap::from_samples(
            samples,
            sample_rate as usize,
            shrink_factor,
        ));
        loop {
            let last_mipmap = mipmaps.last().unwrap();
            if last_mipmap.len() / 2 <= shrink_factor.get() {
                break;
            }

            mipmaps.push(last_mipmap.shrink(shrink_factor));
        }

        Self {
            sample_rate: sample_rate as usize,
            num_samples: samples.len(),
            mipmaps,
        }
    }

    pub fn get_points(
        &self,
        desired_points: usize,
        time_range: std::ops::Range<f32>,
    ) -> [&[egui::Vec2]; 2] {
        let target_points_per_second = desired_points as f32 / (time_range.end - time_range.start);

        let (i, mipmap) = self
            .mipmaps
            .iter()
            .enumerate()
            .take_while(|(_, mip)| mip.points_per_second > target_points_per_second)
            .last()
            .unwrap_or((0, &self.mipmaps[0]));

        LAST_MIPMAP.store(i as u32, std::sync::atomic::Ordering::Relaxed);

        // println!(
        //     "Mipmap: {}pts, density: {}pts/s wanted, got {}pts/s ({:?})",
        //     mipmap.len(),
        //     target_points_per_second,
        //     mipmap.points_per_second,
        //     time_range
        // );

        [
            Self::point_range(&mipmap.positive_peaks, time_range.clone()),
            Self::point_range(&mipmap.negative_peaks, time_range.clone()),
        ]
    }

    pub fn get_outline(
        &self,
        pixels_per_point: f32,
        rect: egui::Rect,
        time_range: std::ops::Range<f32>,
        stroke: impl Into<egui::Stroke>,
    ) -> egui::epaint::PathShape {
        let desired_num_points = (rect.width() / pixels_per_point).ceil() as usize;

        let [max_points, min_points] = self.get_points(desired_num_points, time_range.clone());

        egui::epaint::PathShape::closed_line(
            max_points
                .iter()
                .copied()
                .chain(min_points.iter().rev().copied())
                .map(|p| {
                    egui::pos2(
                        egui::remap(p.x, time_range.start..=time_range.end, rect.x_range()),
                        egui::remap(p.y, 1.0..=-1.0, rect.y_range()),
                    )
                })
                .collect(),
            stroke,
        )
    }

    pub fn len_seconds(&self) -> f32 {
        self.num_samples as f32 / self.sample_rate as f32
    }

    pub fn calculate_into_async(
        output: Arc<RwLock<Option<Self>>>,
        samples: Vec<f32>,
        sample_rate: u32,
        ctx: Option<egui::Context>,
    ) {
        std::thread::spawn(move || {
            let result = Self::calculate(&samples, sample_rate);
            *output.write().unwrap() = Some(result);
            if let Some(ctx) = ctx {
                ctx.request_repaint();
            }
        });
    }

    fn point_range(points: &[egui::Vec2], range: std::ops::Range<f32>) -> &[egui::Vec2] {
        // TODO: Optimize this using binary search or point density
        let start = points
            .iter()
            .enumerate()
            .take_while(|(_, p)| p.x < range.start)
            .last()
            .map_or(0, |(i, _)| i);
        let end = points
            .iter()
            .enumerate()
            .skip(start)
            .find(|(_, p)| p.x > range.end)
            .map_or(points.len() - 1, |(i, _)| i);

        &points[start..=end]
    }
}

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

    pub fn time_range_inclusive(&self) -> RangeInclusive<f32> {
        self.time_range.start..=self.time_range.end
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

pub struct Waveform<'a> {
    pub data:   &'a WaveformData,
    pub cursor: Option<&'a mut TimeCursor>,
}
impl<'a> Waveform<'a> {
    pub fn new(data: &'a WaveformData) -> Self {
        Self { data, cursor: None }
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
        let waveform = self.data;

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

        painter.add(self.data.get_outline(
            10.0, // Pixels per point
            rect,
            cursor.time_range.clone(),
            ui.style().visuals.widgets.noninteractive.fg_stroke,
        ));

        response
    }
}
