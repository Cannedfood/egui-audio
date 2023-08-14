use egui_audio::{Fader, Knob};

fn main() {
    let mut faders = [(0f32, 0f32); 8];

    let mut control_points = vec![
        egui_audio::ControlPoint::new(egui::vec2(1.0, 0.0)),
        egui_audio::ControlPoint::new(egui::vec2(0.5, 0.5)),
        egui_audio::ControlPoint::new(egui::vec2(0.0, 1.0)),
    ];

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
        });
    })
    .expect("Failed to open window");
}
