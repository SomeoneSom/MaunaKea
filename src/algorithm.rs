//use std::time::{Duration, Instant};
use std::io::stdout;
use std::io::Write;
use std::num::ParseFloatError;

use crate::colliders::Collider;
use crate::colliders::Rect;
use crate::level::Level;
use crate::player::Player;
use crate::point::Point;

use argmin::core::{CostFunction, Error, Executor, Gradient, State};
use bitvec::prelude as bv;
use colored::Colorize;
use thiserror::Error;

//TODO: finish implementing these traits
//TODO: add useless input removal
//TODO: add hazard avoidance
#[derive(Debug)]
struct DistanceFn {
    start: Point,
    rects: Vec<Rect>,
}

impl CostFunction for DistanceFn {
    type Param = Vec<f32>;
    type Output = f32;

    fn cost(&self, param: &Self::Param) -> Result<Self::Output, Error> {
        let mut points = param
            .iter()
            .step_by(2)
            .zip(param.iter().skip(1).step_by(2))
            .zip(self.rects.iter())
            .map(|((x, y), r)| Point::new(*x, *y) * (r.dr - r.center()) + r.center())
            .collect::<Vec<_>>();
        points.insert(0, self.start);
        Ok(points
            .windows(2)
            .fold(0., |acc, p| acc + p[0].distance(p[1])))
    }
}

impl Gradient for DistanceFn {
    type Param = Vec<f32>;
    type Gradient = Vec<f32>;

    fn gradient(&self, param: &Self::Param) -> Result<Self::Gradient, Error> {
        let mut points = param
            .iter()
            .step_by(2)
            .zip(param.iter().skip(1).step_by(2))
            .zip(self.rects.iter())
            .map(|((x, y), r)| Point::new(*x, *y) * (r.dr - r.center()) + r.center())
            .collect::<Vec<_>>();
        points.insert(0, self.start);
        //println!("{points:#?}");
        let mut output = vec![0f32; 2];
        for p in points.windows(2) {
            let d = p[0].distance(p[1]);
            let dx = (p[0].x - p[1].x) / d;
            let dy = (p[0].y - p[1].y) / d;
            let len = output.len();
            output[len - 2] += dx;
            output[len - 1] += dy;
            output.push(-dx);
            output.push(-dy);
        }
        Ok(output[2..].to_vec())
    }
}

impl DistanceFn {
    fn from_str(data: &str) -> Result<Self, ParseFloatError> {
        let data_split = data.split('\n').collect::<Vec<_>>();
        let nums = data_split[0]
            .split(", ")
            .map(|s| s.trim().parse::<f32>())
            .collect::<Result<Vec<_>, _>>()?;
        let start = Point::new(nums[0], nums[1]);
        let mut rects: Vec<Rect> = Vec::new();
        for rect in data_split[1..].iter() {
            let mut temp: Vec<f32> = Vec::new();
            for r in rect.split(", ") {
                temp.push(r.trim().parse::<f32>()?);
            }
            rects.push(Rect::new(
                Point::new(temp[0], temp[1]),
                Point::new(temp[2], temp[3]),
            ));
        }
        Ok(Self { start, rects })
    }
}

fn test_descent(distance_fn: &DistanceFn, param: &mut Vec<f32>, change: f32) {
    let mut multiplier = 1.0;
    while multiplier > 0.000001 {
        let grad = distance_fn.gradient(param).unwrap();
        if grad.iter().any(|&x| x.is_nan()) {
            println!("NaN gradient due to overlapping points. Breaking.");
            return;
        }
        for j in 0..param.len() {
            param[j] -= grad[j] * multiplier;
            param[j] = param[j].clamp(-1., 1.);
        }
        multiplier *= change;
    }
}

type Input = (f32, i32);

#[derive(Error, Debug)]
pub enum DataParseError {
    //#[error("Malformed checkpoint on line {line}: {reason}")]
    //MalformedCheckpoint { line: i32, reason: String },

    #[error("Invalid float literal")]
    InvalidFloat(#[from] ParseFloatError),
}

fn parse_data(data: &str) -> Result<Vec<Rect>, DataParseError> {
    let data_split = data.split('\n').collect::<Vec<_>>();
    let mut rects = Vec::new();
    for rect in data_split {
        let mut temp = Vec::new();
        for r in rect.split(", ") {
            temp.push(r.trim().parse::<f32>()?);
        }
        rects.push(Rect::new(
            Point::new(temp[0], temp[1]),
            Point::new(temp[2], temp[3]),
        ));
    }
    Ok(rects)
}

fn weight_angle(angle: f32) -> f32 {
    unimplemented!();
}

fn initial_path() {
    unimplemented!();
}

fn add_turns() {
    unimplemented!();
}

fn adjust() {
    unimplemented!();
}

pub fn run() -> Vec<Input> {
    unimplemented!();
}

pub fn test_alg(data: &str) -> Result<Vec<f32>, ParseFloatError> {
    let distance_fn = DistanceFn::from_str(data)?;
    let mut init_param = vec![0f32; distance_fn.rects.len() * 2];
    println!("{distance_fn:?}");
    println!("{init_param:?}");
    println!("{:?}", distance_fn.gradient(&init_param));
    test_descent(&distance_fn, &mut init_param, 0.9999);
    println!("{init_param:?}");
    let points = init_param
        .iter()
        .step_by(2)
        .zip(init_param.iter().skip(1).step_by(2))
        .zip(distance_fn.rects.iter())
        .map(|((x, y), r)| Point::new(*x, *y) * (r.dr - r.center()) + r.center())
        .collect::<Vec<_>>();
    println!("{points:#?}");
    println!("{:?}", distance_fn.gradient(&init_param));
    Ok(vec![])
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
