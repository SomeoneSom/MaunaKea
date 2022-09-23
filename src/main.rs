mod gui;
mod globals;
mod algorithm;
mod simulator;
mod level;
mod colliders;

fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Waterline BETA",
        options,
        Box::new(|_cc| Box::new(gui::MaunaKea::default())),
    );
}