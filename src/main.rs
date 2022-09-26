mod algorithm;
mod colliders;
mod globals;
mod gui;
mod level;
mod simulator;

use colored::Colorize;

fn main() {
    if cfg!(windows) {
        unsafe {
            winapi::um::consoleapi::SetConsoleMode(
                kernel32::GetStdHandle(winapi::um::winbase::STD_OUTPUT_HANDLE), 0x4 | 0x1);
        }
    }
    println!("{}", "MaunaKea ALPHA, by atpx8".bright_cyan());
    println!("{}", "WARNING: This is unfinished, and likely will not fully work.".red());
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "MaunaKea ALPHA",
        options,
        Box::new(|_cc| Box::new(gui::MaunaKea::default())),
    );
}
