use rustfft::num_complex::Complex;

pub struct WaveformSpectrum {}
impl WaveformSpectrum {
    pub fn calculate(samples: &[f32], sample_rate: usize) -> Self {
        let frequencies = {
            let steps = 128;
            let start_frequency = 30.0f32;
            let end_frequency = 20000.0f32;
            let step_factor = (end_frequency / start_frequency).powf(1.0 / steps as f32);

            let mut f = start_frequency;
            todo!()
        };

        let planner = rustfft::FftPlanner::new();
        let fft_forward = planner.plan_fft_forward(sample_rate);
        let fft_backward = planner.plan_fft_inverse(sample_rate);

        let mut scratch: Vec<Complex<f32>> = {
            let required_scratch = usize::max(
                fft_backward.get_inplace_scratch_len(),
                fft_forward.get_inplace_scratch_len(),
            );

            (0..required_scratch)
                .map(|_| Complex::new(0.0, 0.0))
                .collect()
        };

        let mut input: Vec<Complex<f32>> = samples
            .iter()
            .map(|&sample| Complex::new(sample, 0.0))
            .collect();

        fft_forward.process_with_scratch(&mut input, &mut scratch);
    }
}
