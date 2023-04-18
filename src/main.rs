#![warn(clippy::unwrap_used, clippy::expect_used)]

mod algorithm;
mod colliders;
mod gui;
mod level;
mod player;
mod point;

use colored::Colorize;

#[cfg(windows)]
fn fix_conhost() {
    unsafe {
        winapi::um::consoleapi::SetConsoleMode(
            winapi::um::processenv::GetStdHandle(winapi::um::winbase::STD_OUTPUT_HANDLE),
            0x4 | 0x1,
        );
    }
}

#[cfg(not(windows))]
const fn fix_conhost() {}

fn main() {
    if cfg!(windows) {
        fix_conhost();
    }
    println!("{}", "MaunaKea ALPHA, by atpx8".bright_cyan());
    println!(
        "{}",
        "WARNING: This is unfinished, and likely will not fully work.".red()
    );
    //let options = eframe::NativeOptions::default();
    //eframe::run_native(
    //    "MaunaKea ALPHA v0.0.3",
    //    options,
    //    Box::new(|_cc| Box::<gui::MaunaKea>::default()),
    //);
    println!("current running headless for testing");
    let (level, player) = level::Level::load("/home/atpx8/Celeste/infodump.txt");
    algorithm::run_alg(level, player, "-860, -265, -840, -185\n-802, -232, -778, -186\n-750, -279, -714, -236\n-726, -319, -697, -305");
}
