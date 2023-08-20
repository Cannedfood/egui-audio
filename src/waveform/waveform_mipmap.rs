use std::num::NonZeroUsize;

#[derive(Clone, Debug)]
pub struct WaveformMipmap {
    pub points_per_second: f32,
    pub positive_peaks: Vec<egui::Vec2>,
    pub negative_peaks: Vec<egui::Vec2>,
}

impl WaveformMipmap {
    pub fn simplify(&mut self, max_cost: f32) {
        simplify_path(&mut self.positive_peaks, -max_cost / self.points_per_second);
        simplify_path(&mut self.negative_peaks, max_cost / self.points_per_second);
    }

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

    pub(crate) fn positive_peak_in_samples(data: &[f32]) -> (usize, f32) {
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

    pub(crate) fn negative_peak_in_samples(data: &[f32]) -> (usize, f32) {
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

    pub(crate) fn positive_peak_in_points(data: &[egui::Vec2]) -> egui::Vec2 {
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

    pub(crate) fn negative_peak_in_points(data: &[egui::Vec2]) -> egui::Vec2 {
        data.iter().fold(egui::vec2(0.0, f32::INFINITY), |acc, &x| {
            if x.y < acc.y {
                x
            }
            else {
                acc
            }
        })
    }

    pub(crate) fn point_range(&self, time_range: std::ops::Range<f32>) -> [&[egui::Vec2]; 2] {
        [
            point_range_helper(&self.positive_peaks, time_range.clone()),
            point_range_helper(&self.negative_peaks, time_range.clone()),
        ]
    }
}

fn point_range_helper(points: &[egui::Vec2], range: std::ops::Range<f32>) -> &[egui::Vec2] {
    // TODO: Optimize this using binary search or point density
    let start = match points.binary_search_by(|p| p.x.partial_cmp(&range.start).unwrap()) {
        Err(a) => a,
        Ok(a) => a,
    }
    .saturating_sub(1);

    let end = points[start..]
        .iter()
        .position(|p| p.x > range.end)
        .map(|v| v + start)
        .unwrap_or(points.len() - 1);

    &points[start..=end]
}

fn cross(a: egui::Vec2, b: egui::Vec2) -> f32 {
    a.x * b.y - a.y * b.x
}

fn triangle_area_2(p: egui::Vec2, left: egui::Vec2, right: egui::Vec2) -> f32 {
    cross(left - p, right - p)
}

fn simplify_path(points: &mut Vec<egui::Vec2>, max_area_error: f32) {
    assert!(points.len() > 3);

    let max_cost_2 = max_area_error * 2.0;

    loop {
        let mut last_kept_point = points[0];
        let mut current_index = 1;
        for i in 1..points.len() - 1 {
            let keep = if max_cost_2 < 0.0 {
                max_cost_2 < triangle_area_2(points[i], last_kept_point, points[i + 1])
            }
            else {
                max_cost_2 > triangle_area_2(points[i], last_kept_point, points[i + 1])
            };

            if keep {
                last_kept_point = points[i];
                points[current_index] = points[i];
                current_index += 1;
            }
        }
        points[current_index] = *points.last().unwrap();
        current_index += 1;

        println!("Simplified from {} to {}", points.len(), current_index);

        if points.len() == current_index {
            break;
        }

        points.resize(current_index, egui::Vec2::ZERO);
    }
    points.shrink_to_fit();
}
