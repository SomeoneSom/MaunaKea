//use crate::globals::{ANGLES};
//use std::time::{Duration, Instant};
use std::io::stdout;
use std::io::Write;

use crate::colliders::{Point, Rect};
use crate::level::Level;
use crate::player::Player;

use bitvec::prelude as bv;
use colored::Colorize;

pub struct Pathfinder {}

impl Pathfinder {
    pub fn intersection_point(
        line1: (Point, Point), line2: (Point, Point),
    ) -> Option<(bool, Point)> {
        let c1s = line1.1 - line1.0;
        let c1m = c1s * line1.0;
        let c1 = c1m.x - c1m.y;

        let c2s = line2.1 - line2.0;
        let c2m = c2s * line2.0;
        let c2 = c2m.x - c2m.y;

        let determinant = c1s.x * -c2s.y - c2s.x * -c1s.y;

        if determinant == 0.0 {
            None
        } else {
            let x = (c1s.y * c2 - c2s.y * c1) / determinant;
            let y = (c1s.x * c2 - c2s.x * c1) / determinant;
            Some((
                (f32::min(line1.0.x, line1.1.x) <= x
                    && x <= f32::max(line1.0.x, line1.1.x)
                    && f32::min(line1.0.y, line1.1.y) <= y
                    && y <= f32::max(line1.0.y, line1.1.y))
                    && (f32::min(line2.0.x, line2.1.x) <= x
                        && x <= f32::max(line2.0.x, line2.1.x)
                        && f32::min(line2.0.y, line2.1.y) <= y
                        && y <= f32::max(line2.0.y, line2.1.y)),
                Point::new(x, y),
            ))
        }
    }

    pub fn initial_path(checkpoints: &Vec<Rect>, starting: Point) -> Vec<Point> {
        let mut path: Vec<Point> = vec![];
        path
    }
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
        println!("Position: ({}, {})", rect.ul.x + 4., rect.ul.y + 11.);
        println!("Speed: ({}, {})", player.speed.x, player.speed.y);
        best.0
    }
}
