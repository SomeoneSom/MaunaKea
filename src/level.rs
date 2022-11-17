use std::io::stdout;
use std::io::Write;

use bitvec::prelude as bv;
use colored::Colorize;
use regex::Regex;

use crate::algorithm::Algorithm;
use crate::colliders::{Circle, Collider, Rect};
use crate::player::Player;
use image::{ImageBuffer, Rgb, RgbImage};

#[derive(Default)]
pub struct Level {
    bounds: Rect,
    static_death: Vec<bv::BitVec>,
    static_solids: Vec<bv::BitVec>,
    player: Player,
}

impl Level {
    pub fn new() -> Self {
        Self {
            bounds: Rect::default(),
            static_death: Vec::new(),
            static_solids: Vec::new(),
            player: Player::new((0., 0.), (0., 0.)),
        }
    }

    pub fn load(&mut self, info_path: String) -> () {
        let re = Regex::new(
            &(r"(.*)(Pos:\s*-?\d+\.?\d*, \s*-?\d+\.?\d*) (Speed:\s*-?\d+\.?\d*, \s*-?\d+\.?\d*)(.*)"
                .to_owned()
                + r"LightningUL:(.*)Bounds: \{(.*)\} Solids: (.*)"/* + r" LightningDR:(.*)" +
            r" SpikeUL:(.*) SpikeDR:(.*) SpikeDir:(.*)" +
            r" Wind:(.*) WTPos:(.*) WTPattern:(.*) WTWidth:(.*) WTHeight(.*)" +
            r" StarJumpUL:(.*) JThruUL:(.*?)"*/),
        )
        .unwrap();
        let data = &std::fs::read_to_string(info_path)
            .expect("Failed to read infodump file! Somehow uncaught");
        let caps = re.captures(data).unwrap();
        self.load_bounds(caps.get(6).unwrap().as_str().to_owned());
        self.static_death = vec![
            bv::bitvec![0; (self.bounds.dr.0 - self.bounds.ul.0) as usize];
            (self.bounds.dr.1 - self.bounds.ul.1) as usize
        ];
        self.static_solids = self.static_death.clone();
        self.load_solids(caps.get(7).unwrap().as_str().to_owned());
        self.load_spinners(caps.get(4).unwrap().as_str().to_owned());
        /*let mut img: RgbImage = ImageBuffer::new(
            self.static_death[0].len() as u32,
            self.static_death.len() as u32,
        );
        let (width, height) = img.dimensions();
        for y in 0..height {
            for x in 0..width {
                if self.static_solids[y as usize][x as usize] {
                    img.put_pixel(x, y, Rgb([255, 194, 11]));
                } else if self.static_death[y as usize][x as usize] {
                    img.put_pixel(x, y, Rgb([255, 0, 0]));
                } else {
                    img.put_pixel(x, y, Rgb([0, 0, 0]));
                }
            }
        }
        img.save("testimg.png").unwrap();*/
        self.load_player(
            caps.get(2).unwrap().as_str().to_owned(),
            caps.get(3).unwrap().as_str().to_owned(),
        );
        //println!("X:{} Y:{} Width:{} Height:{}", self.bounds.ul.0, self.bounds.ul.1, self.bounds.dr.0, self.bounds.dr.1);
    }

    #[inline(always)]
    fn parse_f32(caps: &regex::Captures, num: usize) -> f32 {
        return caps.get(num).unwrap().as_str().parse::<f32>().unwrap();
    }

    fn get_pair(string: &str) -> (f32, f32) {
        let re = Regex::new(r"(-?\d+\.?\d*), (-?\d+\.?\d*)").unwrap();
        let caps = re.captures(string).unwrap();
        return (Self::parse_f32(&caps, 1), Self::parse_f32(&caps, 2));
    }

    #[inline]
    fn grift_bv(dest: &mut Vec<bv::BitVec>, src: &Vec<bv::BitVec>, x: i32, y: i32) -> () {
        if dest.len() == 0 || src.len() == 0 {
            return ();
        }
        if y > dest.len() as i32 || x >= dest[0].len() as i32 || x + (src[0].len() as i32) < 0 {
            return ();
        }
        for h in 0..src.len() as i32 {
            if h + y < dest.len() as i32 && h + y >= 0 {
                if src[0].len() as i32 + x <= dest[0].len() as i32 && x >= 0 {
                    dest[(h + y) as usize][x as usize..(src[0].len() as i32 + x) as usize]
                        .clone_from_bitslice(&src[h as usize][..]);
                } else {
                    let start: usize = f32::clamp(x as f32, 0., dest.len() as f32 - 1.) as usize;
                    let end: usize =
                        f32::clamp((x + src[0].len() as i32) as f32, 0., dest.len() as f32)
                            as usize;
                    if start != end {
                        dest[(h + y) as usize][start..end].clone_from_bitslice(
                            &src[h as usize]
                                [(start as i32 - x) as usize..(end as i32 - x) as usize],
                        );
                    }
                }
            }
        }
    }

    #[inline]
    fn grift_line(dest: &mut Vec<bv::BitVec>, x: i32, y0: i32, y1: i32, switch: bool) -> () {
        if switch {
            let temp: Vec<bv::BitVec> = vec![bv::bitvec![1; ((y1 - y0).abs()) as usize]];
            Self::grift_bv(dest, &temp, if y0 < y1 { y0 } else { y1 }, x);
        } else {
            let temp: Vec<bv::BitVec> = vec![bv::bitvec![1; 1]; ((y1 - y0).abs()) as usize];
            Self::grift_bv(dest, &temp, x, if y0 < y1 { y0 } else { y1 });
        }
    }

    fn circle_octant(
        dest: &mut Vec<bv::BitVec>, origin: (f32, f32), radius: f32, flip_x: i32, flip_y: i32,
        switch: bool,
    ) -> () {
        let cx: f32;
        let cy: f32;

        if switch {
            cx = origin.1;
            cy = origin.0;
        } else {
            cx = origin.0;
            cy = origin.1;
        }

        let mut x: f32;
        if flip_x > 0 {
            x = f32::ceil(cx + radius - 1.);
        } else {
            x = f32::floor(cx - radius + 1.);
        }

        let mut y: f32;
        if flip_y > 0 {
            y = cy.floor();
        } else {
            y = cy.ceil();
        }

        let mut start_y: f32 = y.clone();
        let mut e: f32 = (x - cx) * (x - cx) + (y - cy) * (y - cy) - radius * radius;
        let mut yc: f32 = flip_y as f32 * 2. * (y - cy) + 1.;
        let mut xc: f32 = flip_x as f32 * -2. * (x - cx) + 1.;

        while flip_y as f32 * (y - cy) <= flip_x as f32 * (x - cx) {
            e += yc;
            y += flip_y as f32;
            yc += 2.;
            if e >= 0. {
                Self::grift_line(
                    dest,
                    x as i32 + (if flip_x < 0 { -1 } else { 0 }),
                    start_y as i32,
                    y as i32,
                    switch,
                );
                start_y = y.clone();
                e += xc;
                x -= flip_x as f32;
                xc += 2.;
            }
        }
        Self::grift_line(
            dest,
            x as i32 + (if flip_x < 0 { -1 } else { 0 }),
            start_y as i32,
            y as i32,
            switch,
        );
    }

    fn grift_circle(dest: &mut Vec<bv::BitVec>, origin: (f32, f32), radius: f32) -> () {
        Self::circle_octant(dest, origin, radius, 1, 1, false);
        Self::circle_octant(dest, origin, radius, 1, -1, false);
        Self::circle_octant(dest, origin, radius, -1, 1, false);
        Self::circle_octant(dest, origin, radius, -1, -1, false);
        Self::circle_octant(dest, origin, radius, 1, 1, true);
        Self::circle_octant(dest, origin, radius, 1, -1, true);
        Self::circle_octant(dest, origin, radius, -1, 1, true);
        Self::circle_octant(dest, origin, radius, -1, -1, true);
    }

    fn load_bounds(&mut self, bounds: String) -> () {
        let re = Regex::new(r"X:(-*\d*) Y:(-*\d*) Width:(-*\d*) Height:(-*\d*)").unwrap();
        let caps = re.captures(&bounds).unwrap();
        self.bounds = Rect::new_xywh(
            Self::parse_f32(&caps, 1),
            Self::parse_f32(&caps, 2),
            Self::parse_f32(&caps, 3),
            Self::parse_f32(&caps, 4),
        );
    }

    fn load_spinners(&mut self, data: String) -> () {
        let mut split: Vec<&str> = data.split("[").collect();
        split.remove(0);
        let to: i32 = split.len() as i32;
        let mut i = 0;
        print!(
            "{}{}{}",
            "Loading spinners... ".bright_green().bold().italic(),
            "0/",
            to
        );
        stdout().flush();
        //this lets us just reuse the same circle bitvec over and over and over
        //instead of generating new ones each time
        let mut circle: Vec<bv::BitVec> = vec![bv::bitvec![0; 12]; 12];
        Self::grift_circle(&mut circle, (6., 6.), 6.);
        for p in split {
            print!(
                "{}",
                "\u{8}".repeat((i).to_string().len() + to.to_string().len() + 1)
            );
            let pair: (f32, f32) = Self::get_pair(p);
            Self::grift_bv(
                &mut self.static_death,
                &circle,
                (pair.0 - 6.) as i32,
                (pair.1 - 6.) as i32,
            );
            Self::grift_bv(
                &mut self.static_death,
                &vec![bv::bitvec![1; 16]; 4],
                pair.0 as i32 - 8 + self.bounds.ul.0 as i32,
                pair.1 as i32 + 5 + self.bounds.ul.1 as i32,
            );
            print!("{}/{}", i + 1, to);
            stdout().flush();
            i += 1;
        }
        println!("");
    }

    fn load_barriers(&mut self, data_ul: String, data_dr: String) -> () {
        let mut ul: Vec<&str> = data_ul.split("[").collect();
        let mut dr: Vec<&str> = data_dr.split("[").collect();
        ul.remove(0);
        dr.remove(0);
        for i in 0..ul.len() {
            let ul_pair: (f32, f32) = Self::get_pair(ul[i]);
            let dr_pair: (f32, f32) = Self::get_pair(dr[i]);
            let width: usize = (dr_pair.0 - ul_pair.0) as usize;
            let height: usize = (dr_pair.1 - ul_pair.1) as usize;
            println!("{} {} {} {}", width, height, (ul_pair.0 - self.bounds.ul.0) as i32, (ul_pair.1 - self.bounds.ul.1) as i32);
            Self::grift_bv(
                &mut self.static_solids,
                &vec![bv::bitvec![1; width]; height],
                (ul_pair.0 - self.bounds.ul.0) as i32,
                (ul_pair.1 - self.bounds.ul.1) as i32,
            );
        }
    }

    fn load_solids(&mut self, data: String) -> () {
        let rows: Vec<&str> = data.split(" ").collect();
        let tile: Vec<bv::BitVec> = vec![bv::bitvec![1; 8]; 8];
        for y in 0..rows.len() {
            for x in 0..rows[0].len() {
                if rows[y].chars().nth(x).unwrap() != '0' {
                    Self::grift_bv(&mut self.static_solids, &tile, x as i32 * 8, y as i32 * 8);
                }
            }
        }
    }

    fn load_player(&mut self, position: String, speed: String) -> () {
        let pair1: (f32, f32) = Self::get_pair(position.as_str());
        let pair2: (f32, f32) = Self::get_pair(speed.as_str());
        self.player = Player::new(pair2, pair1);
    }

    pub fn run_alg(&mut self, checkpoints: String) -> () {
        let mut inputs: Vec<i32> = Vec::new();
        let mut checks: Vec<Rect> = Vec::new();
        for check in checkpoints.split("\n") {
            let mut temp: Vec<f32> = Vec::new();
            for c in check.split(", ") {
                temp.push(c.parse::<f32>().unwrap());
            }
            checks.push(Rect::new((temp[0], temp[1]), (temp[2], temp[3])));
        }
        let mut i: i32 = 0;
        let mut flag: bool = false;
        let mut frame: i32 = 1;
        while i < checks.len() as i32 {
            println!("Frame: {}, Checkpoint: {}", frame, i);
            let inp: i32 = Algorithm::sim_frame(
                &mut self.player,
                &self.bounds,
                &self.static_death,
                &checks[i as usize],
            );
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
        let mut last_input: i32 = -1;
        let mut count: i32 = 0;
        for inp in inputs {
            if inp != last_input {
                if last_input != -1 {
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
        println!(
            "{},F,{}.{:0>3}",
            count,
            (last_input as f32 / 1000.) as i32,
            last_input % 1000
        );
        return;
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
