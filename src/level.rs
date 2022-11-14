use std::io::Write;
use std::io::stdout;

use colored::Colorize;
use regex::Regex;
use bitvec::prelude as bv;

use crate::colliders::{Death, Collider, Circle, Rect, Player};
use crate::algorithm::Algorithm;

#[derive(Default)]
pub struct Level {
    bounds:Rect,
    death2:Vec<bv::BitVec>,
    death:Vec<Death>,
    solids: Vec<bv::BitVec>,
    player:Player
}

impl Level {
    pub fn new() -> Self {
        Self {
            bounds: Rect::default(),
            death2: Vec::new(),
            death: Vec::new(),
            solids: Vec::new(),
            player: Player::new((0., 0.), (0., 0.))
        }
    }

    pub fn load(&mut self, info_path:String) -> () {
        let re = Regex::new(&(r"(.*)(Pos:\s*-?\d+\.?\d*, \s*-?\d+\.?\d*) (Speed:\s*-?\d+\.?\d*, \s*-?\d+\.?\d*)(.*)".to_owned() +
        r"LightningUL:(.*)Bounds: \{(.*)\}"/* + r" LightningDR:(.*)" +
        r" SpikeUL:(.*) SpikeDR:(.*) SpikeDir:(.*)" +
        r" Wind:(.*) WTPos:(.*) WTPattern:(.*) WTWidth:(.*) WTHeight(.*)" +
        r" StarJumpUL:(.*) JThruUL:(.*?)"*/)).unwrap();
        let data = &std::fs::read_to_string(info_path).expect("Failed to read infodump file! Somehow uncaught");
        let caps = re.captures(data).unwrap();
        self.load_bounds(caps.get(6).unwrap().as_str().to_owned());
        self.death2 = vec![bv::bitvec![0; (self.bounds.dr.0 - self.bounds.ul.0) as usize];
            (self.bounds.dr.1 - self.bounds.ul.1) as usize];
        self.solids = self.death2.clone();
        self.load_spinners(caps.get(4).unwrap().as_str().to_owned());
        self.load_player(caps.get(2).unwrap().as_str().to_owned(), caps.get(3).unwrap().as_str().to_owned());
        //println!("X:{} Y:{} Width:{} Height:{}", self.bounds.ul.0, self.bounds.ul.1, self.bounds.dr.0, self.bounds.dr.1);
    }

    #[inline]
    fn parse_f32(caps: &regex::Captures, num: usize) -> f32 {
        return caps.get(num).unwrap().as_str().parse::<f32>().unwrap();
    }

    fn get_pair(string:&str) -> (f32, f32) {
        let re = Regex::new(r"(-?\d+\.?\d*), (-?\d+\.?\d*)").unwrap();
        let caps = re.captures(string).unwrap();
        return (Self::parse_f32(&caps, 1), Self::parse_f32(&caps, 2));
    }

    fn load_bounds(&mut self, bounds:String) -> () {
        let re = Regex::new(r"X:(-*\d*) Y:(-*\d*) Width:(-*\d*) Height:(-*\d*)").unwrap();
        let caps = re.captures(&bounds).unwrap();
        self.bounds = Rect::new_xywh(Self::parse_f32(&caps, 1), Self::parse_f32(&caps, 2), 
            Self::parse_f32(&caps, 3), Self::parse_f32(&caps, 4));
    }

    fn load_spinners(&mut self, data:String) -> () {
        let re = Regex::new(r"(-?\d+\.?\d*), (-?\d+\.?\d*)").unwrap();
        let mut split:Vec<&str> = data.split("[").collect();
        split.remove(0);
        let to:i32 = split.len() as i32;
        let mut i = 0;
        let mut ret:Vec<Death> = Vec::new();
        print!("{}{}{}", "Loading spinners... ".bright_green().bold().italic(), "0/", to);
        stdout().flush();
        for p in split {
            print!("{}", "\u{8}".repeat((i).to_string().len() + to.to_string().len() + 1));
            let caps = re.captures(p).unwrap();
            let pair:(f32, f32) = (caps.get(1).unwrap().as_str().parse::<f32>().unwrap(),
                caps.get(2).unwrap().as_str().parse::<f32>().unwrap());
            ret.push(Death::new(vec![Collider::Circular(Circle::new(6., pair)),
                Collider::Rectangular(Rect::new((pair.0 - 8., pair.1 - 3.), (pair.0 + 8., pair.1 + 1.)))]));
            print!("{}/{}", i + 1, to);
            stdout().flush();
            i += 1;
        }
        /*let start = Instant::now();
        let mut next:f32 = 0.1;
        let to:i32 = 100000;
        print!("{}{}{}", "Loading spinners    ".green().bold(), "0/", to);
        stdout().flush();
        for i in 0..to {
            print!("{}", "\u{8}".repeat((i).to_string().len() + to.to_string().len() + 1));
            let dur:f32 = start.elapsed().as_secs_f32();
            if dur >= next {
                next += 0.1;
                print!("{}", "\u{8}".repeat(4));
                let dots:i32  = ((dur / 0.1) % 4.) as i32;
                print!("{}{}", ".".repeat(dots as usize).green().bold(), " ".repeat(4 - dots as usize));
            }
            print!("{}/{}", i + 1, to);
            stdout().flush();
        }*/
        println!("");
        self.death = ret;
    }

    fn load_player(&mut self, position:String, speed:String) -> () {
        let pair1:(f32, f32) = Self::get_pair(position.as_str());
        let pair2:(f32, f32) = Self::get_pair(speed.as_str());
        self.player = Player::new(pair2, pair1);
    }

    pub fn run_alg(&mut self, checkpoints:String) -> () {
        let mut inputs:Vec<i32> = Vec::new();
        let mut checks:Vec<Rect> = Vec::new();
        for check in checkpoints.split("\n") {
            let mut temp:Vec<f32> = Vec::new();
            for c in check.split(", ") {
                temp.push(c.parse::<f32>().unwrap());
            }
            checks.push(Rect::new((temp[0], temp[1]), (temp[2], temp[3])));
        }
        let mut i:i32 = 0;
        let mut flag:bool = false;
        let mut frame:i32 = 1;
        while i < checks.len() as i32 {
            println!("Frame: {}, Checkpoint: {}", frame, i);
            let inp:i32 = Algorithm::sim_frame(&mut self.player, &self.death, &checks[i as usize]);
            if inp == -1 {
                flag = true;
                break;
            } else {
                inputs.push(inp);
            }
            if Collider::Rectangular(checks[i as usize]).collide_check(&self.player.hurtbox) {
                i += 1;
            }
            frame += 1;
        }
        if !flag {
            println!("{}", "Done! Inputs:".bright_green());
        } else {
            println!("{}", "Inputs before aborting:".red());
        }
        let mut last_input:i32 = -1;
        let mut count:i32 = 0;
        for inp in inputs {
            if inp != last_input {
                if last_input != -1 {
                    println!("{},F,{}.{:0>3}", count, (last_input as f32 / 1000.) as i32, last_input % 1000);
                }
                last_input = inp;
                count = 1;
            } else {
                count += 1;
            }
        }
        println!("{},F,{}.{:0>3}", count, (last_input as f32 / 1000.) as i32, last_input % 1000);
        return
    }

    /*pub fn test_count(to:i32, interval:f32) {
        let start = Instant::now();
        let mut next:f32 = interval;
        print!("loading    0/{}", to);
        stdout().flush();
        for i in 1..=to {
            print!("{}", "\u{8}".repeat((i - 1).to_string().len() + to.to_string().len() + 1));
            let dur:f32 = start.elapsed().as_secs_f32();
            if dur >= next {
                next += interval;
                print!("{}", "\u{8}".repeat(4));
                let dots:i32  = ((dur / interval) % 4.) as i32;
                print!("{}{}", ".".repeat(dots as usize), " ".repeat(4 - dots as usize));
            }
            print!("{}/{}", i, to);
            stdout().flush();
        }
    }*/
}