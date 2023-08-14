use std::num::NonZeroUsize;
use std::sync::Arc;
use std::sync::RwLock;

pub struct WaveformData {
    pub sample_rate: usize,
    pub num_samples: usize,
    pub mipmaps: Vec<super::WaveformMipmap>,
}

impl WaveformData {
    pub fn calculate(samples: &[f32], sample_rate: u32) -> Self {
        let mut mipmaps = Vec::new();
        let shrink_factor: NonZeroUsize = 2.try_into().unwrap();

        mipmaps.push(super::WaveformMipmap::from_samples(
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

        // Take the first mipmap that has at least target_points_per_second
        let mipmap = self
            .mipmaps
            .iter()
            .take_while(|mip| mip.points_per_second > target_points_per_second)
            .last()
            .unwrap_or(&self.mipmaps[0]);

        mipmap.point_range(time_range)
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
}
