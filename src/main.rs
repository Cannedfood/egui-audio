use std::f32::consts::TAU;

use egui_audio::{Fader, Knob};

fn main() {
    let mut faders = [(0f32, 0f32); 8];

    let mut control_points = vec![
        egui_audio::ControlPoint::new(egui::vec2(1.0, 0.0)),
        egui_audio::ControlPoint::new(egui::vec2(0.5, 0.5)),
        egui_audio::ControlPoint::new(egui::vec2(0.0, 1.0)),
    ];

    let waveform =
        egui_audio::WaveformData::calculate(&generate_example_waveform(48000, 10.0), 48000);
    let mut cursor = egui_audio::TimeCursor::default();

    eframe::run_simple_native("audio_demo", Default::default(), move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Faders / Knobs");
            ui.horizontal(|ui| {
                for (volume, _pan) in &mut faders {
                    ui.vertical(|ui| {
                        ui.add(Knob::pan(_pan).label("pan"));
                        ui.add(Fader::volume(volume).range(-32.0..=0.0).label("volume"));
                    });
                }
            });
            ui.heading("Envelope");
            ui.add(egui_audio::Envelope::new(&mut control_points));
            ui.heading("Waveform");
            ui.add(
                egui_audio::Waveform::default()
                    .entry(egui_audio::Entry::from(&waveform))
                    .marker(egui_audio::Marker {
                        start: 0.5,
                        end: Some(1.0),
                        text: "Fun times".into(),
                        ..Default::default()
                    })
                    .cursor(&mut cursor),
            )
        });
    })
    .expect("Failed to open window");
}

pub fn generate_example_waveform(sample_rate: usize, seconds: f32) -> Vec<f32> {
    let length = (seconds * sample_rate as f32).ceil() as usize;

    let mut result = Vec::with_capacity(length);

    let phase_step = 1.0 / sample_rate as f32;
    let mut t = 0.0;

    let f1 = 5.0;
    let f2 = 100.0;
    let f3 = 400.0;
    while result.len() < length {
        let w = t * TAU;
        result.push(((f1 * w).sin() + (f2 * w).sin() + (f3 * w).sin()) / 4.0);
        t += phase_step;
    }

    result
}
