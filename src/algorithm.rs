mod geneticalg;
mod waterspeed;
//use std::time::{Duration, Instant};
use std::io::stdout;
use std::io::Write;
use std::num::ParseFloatError;

use crate::colliders::Collider;
use crate::colliders::Rect;
use crate::level::Level;
use crate::player::Player;
use crate::point::Point;

use bitvec::prelude as bv;
use colored::Colorize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataParseError {
    #[error("Invalid number of rectangle corners on line {0}: expected 4, got {1}")]
    InvalidNumRectCorners(usize, usize),

    #[error("Invalid float literal")]
    InvalidFloat(#[from] ParseFloatError),
}

fn parse_checkpoint(data: &str) -> Result<Vec<Rect>, DataParseError> {
    let data_split = data.split('\n').collect::<Vec<_>>();
    let mut rects = Vec::new();
    for (line, rect) in data_split.iter().enumerate() {
        let temp_split = rect.split(", ").collect::<Vec<_>>();
        if temp_split.len() != 4 {
            return Err(DataParseError::InvalidNumRectCorners(
                line,
                temp_split.len(),
            ));
        }
        let temp_nums = temp_split
            .iter()
            .map(|x| x.parse::<f32>())
            .collect::<Result<Vec<_>, _>>()?;
        rects.push(Rect::new(
            Point::new(temp_nums[0], temp_nums[1]),
            Point::new(temp_nums[2], temp_nums[3]),
        ));
    }
    Ok(rects)
}

pub fn run_alg(level: &mut Level, checkpoints: String) {
    let mut inputs: Vec<i32> = Vec::new();
    let mut checks: Vec<Rect> = Vec::new();
    for check in checkpoints.split('\n') {
        let mut temp: Vec<f32> = Vec::new();
        for c in check.split(", ") {
            temp.push(c.parse::<f32>().unwrap());
        }
        checks.push(Rect::new(
            Point::new(temp[0], temp[1]),
            Point::new(temp[2], temp[3]),
        ));
    }
    let mut i: i32 = 0;
    let mut flag: bool = false;
    let mut frame: i32 = 1;
    while i < checks.len() as i32 {
        println!("Frame: {}, Checkpoint: {}", frame, i);
        let inp: i32 = sim_frame(
            &mut level.player,
            &level.bounds,
            &level.static_death,
            &level.static_solids,
            &checks[i as usize],
        );
        if inp == -1 {
            flag = true;
            break;
        } else {
            inputs.push(inp);
        }
        if Collider::Rectangular(checks[i as usize]).collide_check(&level.player.hurtbox) {
            i += 1;
        }
        frame += 1;
    }
    if !flag {
        println!("{}", "Done! Inputs:".bright_green());
    } else {
        println!("{}", "Inputs before aborting:".red());
    }
    let mut last_input: i32 = -1;
    let mut count: i32 = 0;
    let mut file = std::fs::File::create("pain.txt").unwrap();
    for inp in inputs {
        if inp != last_input {
            if last_input != -1 {
                file.write_all(
                    format!(
                        "{},F,{}.{:0>3}\n",
                        count,
                        (last_input as f32 / 1000.) as i32,
                        last_input % 1000
                    )
                    .as_bytes(),
                )
                .unwrap();
                println!(
                    "{},F,{}.{:0>3}",
                    count,
                    (last_input as f32 / 1000.) as i32,
                    last_input % 1000
                );
            }
            last_input = inp;
            count = 1;
        } else {
            count += 1;
        }
    }
    file.write_all(
        format!(
            "{},F,{}.{:0>3}\n",
            count,
            (last_input as f32 / 1000.) as i32,
            last_input % 1000
        )
        .as_bytes(),
    )
    .unwrap();
    println!(
        "{},F,{}.{:0>3}",
        count,
        (last_input as f32 / 1000.) as i32,
        last_input % 1000
    );
}

//current approach will be brute force just to test the simulation. this is horrible but it will be improved later
pub fn sim_frame(
    player: &mut Player, bounds: &Rect, static_death: &[bv::BitVec], static_solids: &[bv::BitVec],
    checkpoint: &Rect,
) -> i32 {
    let mut best: (i32, f32) = (360000, 9999999.);
    //print!("Round 1/4");
    //stdout().flush();
    for i in (0..360000).step_by(1000) {
        let result: f32 = player.sim_frame(i, bounds, static_death, static_solids, checkpoint);
        if result < best.1 {
            best = (i, result);
        }
    }
    //println!("Winner: {},{}", best.0, best.1);
    //print!("\u{8}\u{8}\u{8}2");
    //stdout().flush();
    for i in ((best.0 - 1000)..(best.0 + 1000)).step_by(100) {
        let result: f32 = player.sim_frame(i, bounds, static_death, static_solids, checkpoint);
        if result < best.1 {
            best = (i, result);
        }
    }
    //print!("\u{8}3");
    //stdout().flush();
    //println!("Winner: {},{}", best.0, best.1);
    for i in ((best.0 - 100)..(best.0 + 100)).step_by(10) {
        let result: f32 = player.sim_frame(i, bounds, static_death, static_solids, checkpoint);
        if result < best.1 {
            best = (i, result);
        }
    }
    //print!("\u{8}4");
    //stdout().flush();
    //println!("Winner: {},{}", best.0, best.1);
    for i in ((best.0 - 10)..(best.0 + 10)).step_by(1) {
        let result: f32 = player.sim_frame(i, bounds, static_death, static_solids, checkpoint);
        if result < best.1 {
            best = (i, result);
        }
    }
    //println!("Winner: {},{}", best.0, best.1);
    //println!("");
    if best.0 == 360000 {
        println!("{}", "ERROR: can't find usable angle! Aborting.".red());
        return -1;
    }
    println!("Winner: {} -> {}", best.0, best.1);
    player.move_self(best.0, bounds, static_solids);
    let rect: &Rect = player.hurtbox.rect().unwrap();
    println!("Position: ({}, {})", rect.ul.x + 4., rect.ul.y + 11.);
    println!("Speed: ({}, {})", player.speed.x, player.speed.y);
    best.0
}
