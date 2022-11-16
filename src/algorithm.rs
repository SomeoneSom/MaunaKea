//use crate::globals::{ANGLES};
//use std::time::{Duration, Instant};
use std::io::stdout;
use std::io::Write;

use crate::colliders::{Player, Rect};

use bitvec::prelude as bv;
use colored::Colorize;

pub struct Initializer {}

impl Initializer {
    /*pub fn comp_angles() -> () {
        let start = Instant::now();
        let mut vec:Vec<(f32, f32)> = Vec::new();
        for i in 0..360000 {
            let x:f32 =  (60. * 80.) / ((640. + 360. * ((i as f32 / 1000.).to_radians().tan()).powi(2)).sqrt())
                * if i < 90000 || i > 270000 {1.} else {-1.};
            let y:f32 = (60. * 80.) / ((360. + 640. / ((i as f32 / 1000.).to_radians().tan()).powi(2)).sqrt())
                * if i < 180000 {1.} else {-1.};
            vec.push((x, y));
        }
        let boxes:Box<[(f32, f32)]> = vec.clone().into_boxed_slice();
        vec.clear();
        unsafe {
            for i in 0..360000 {
                ANGLES[i] = boxes[i];
            }
            PRE_COMPING = false;
        }
        let duration = start.elapsed();
        println!("Precomputing done in {:?}!", duration);
    }*/
}

pub struct Algorithm {}

//current approach will be brute force just to test the simulation. this is horrible but it will be improved later
impl Algorithm {
    pub fn sim_frame(
        player: &mut Player, bounds: &Rect, static_death: &Vec<bv::BitVec>, checkpoint: &Rect,
    ) -> i32 {
        let mut best: (i32, f32) = (360000, 9999999.);
        //print!("Round 1/4");
        //stdout().flush();
        for i in (0..360000).step_by(1000) {
            let result: f32 = player.sim_frame(i, bounds, static_death, checkpoint);
            if result < best.1 {
                best = (i, result);
            }
        }
        //println!("Winner: {},{}", best.0, best.1);
        //print!("\u{8}\u{8}\u{8}2");
        //stdout().flush();
        for i in ((best.0 - 1000)..(best.0 + 1000)).step_by(100) {
            let result: f32 = player.sim_frame(i, bounds, static_death, checkpoint);
            if result < best.1 {
                best = (i, result);
            }
        }
        //print!("\u{8}3");
        //stdout().flush();
        //println!("Winner: {},{}", best.0, best.1);
        for i in ((best.0 - 100)..(best.0 + 100)).step_by(10) {
            let result: f32 = player.sim_frame(i, bounds, static_death, checkpoint);
            if result < best.1 {
                best = (i, result);
            }
        }
        //print!("\u{8}4");
        //stdout().flush();
        //println!("Winner: {},{}", best.0, best.1);
        for i in ((best.0 - 10)..(best.0 + 10)).step_by(1) {
            let result: f32 = player.sim_frame(i, bounds, static_death, checkpoint);
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
        player.move_self(best.0);
        return best.0;
    }
}
