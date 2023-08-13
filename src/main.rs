use egui_audio::{fader::Fader, knob::Knob};

fn main() {
    let mut faders = [(0f32, 0f32); 8];

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
        });
    })
    .expect("Failed to open window");
}
