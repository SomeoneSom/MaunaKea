//use std::time::{Duration, Instant};
use std::io::stdout;
use std::io::Write;

use crate::colliders::Rect;
use crate::player::Player;
use crate::point::Point;

use bitvec::prelude as bv;
use colored::Colorize;

pub fn initial_path(checkpoints: &[Rect], starting: Point) -> Vec<Point> {
    let mut path = vec![];
    path
}

//current approach will be brute force just to test the simulation. this is horrible but it will be improved later
pub fn sim_frame(
    player: &mut Player, bounds: &Rect, static_death: &[bv::BitVec],
    static_solids: &[bv::BitVec], checkpoint: &Rect,
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
