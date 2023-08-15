use std::num::NonZeroUsize;

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

            let (positive_peak_idx, positive_peak_value) = positive_peak_in_samples(subrange);
            let (negative_peak_idx, negative_peak_value) = negative_peak_in_samples(subrange);

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

            result.positive_peaks.push(positive_peak_in_points(
                &self.positive_peaks[subrange.clone()],
            ));
            result.negative_peaks.push(negative_peak_in_points(
                &self.negative_peaks[subrange.clone()],
            ));
        }

        result
    }

    pub fn point_range(&self, time_range: std::ops::Range<f32>) -> [&[egui::Vec2]; 2] {
        [
            point_range_helper(&self.positive_peaks, time_range.clone()),
            point_range_helper(&self.negative_peaks, time_range.clone()),
        ]
    }
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

fn point_range_helper(points: &[egui::Vec2], range: std::ops::Range<f32>) -> &[egui::Vec2] {
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
