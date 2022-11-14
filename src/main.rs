mod algorithm;
mod colliders;
mod globals;
mod gui;
mod level;

use colored::Colorize;

//C:\Users\kidss\Desktop\Celeste\infodump.txt

/*
131, 77, 139, 89
120, 68, 133, 79
126, 45, 139, 64
144, 32, 159, 47
154, 61, 164, 74
157, 89, 164, 97
*/

/*
159, 48, 173, 62
194, 68, 212, 91
152, 83, 171, 100
*/

/*
158, 45, 165, 54
201, 43, 209, 53
202, 86, 210, 98
157, 89, 164, 98
*/

/*
155, 61, 163, 71
144, 32, 159, 47
200, 33, 213, 48
200, 56, 215, 71
204, 90, 210, 101
192, 128, 207, 143
176, 103, 186, 118
157, 89, 164, 99
*/

/*
173, 78, 177, 83
*/

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
