//use crate::globals::{ANGLES};
//use std::time::{Duration, Instant};
use std::io::stdout;
use std::io::Write;

use crate::colliders::Rect;
use crate::player::Player;
use crate::level::Level;

use bitvec::prelude as bv;
use colored::Colorize;

//this will be a genetic alg, similar to what featherline uses. this will be used if it outperforms
//the deterministic alg.
pub struct AlgGenetic {}

impl AlgGenetic {
}

//this will be a deterministic alg that i will make myself. this will be used if it outperforms the
//genetic alg.
pub struct AlgDeterministic {}

impl AlgDeterministic {
}

pub struct Algorithm {}

//current approach will be brute force just to test the simulation. this is horrible but it will be improved later
impl Algorithm {
    pub fn sim_frame(
        player: &mut Player, bounds: &Rect, static_death: &Vec<bv::BitVec>,
        static_solids: &Vec<bv::BitVec>, checkpoint: &Rect,
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
        println!("Position: ({}, {})", rect.ul.0 + 4., rect.ul.1 + 11.);
        println!("Speed: ({}, {})", player.speed.0, player.speed.1);
        return best.0;
    }
}
