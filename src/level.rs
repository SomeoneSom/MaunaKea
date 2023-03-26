use bitvec::prelude as bv;
use colored::Colorize;
use image::{ImageBuffer, Rgb, RgbImage};
use regex::Regex;
use std::io::stdout;
use std::io::Write;

use crate::colliders::{Collider, Rect};
use crate::player::Player;
use crate::point::Point;

#[derive(Debug, Default)]
pub struct Level {
    pub bounds: Rect,
    pub static_death: Vec<bv::BitVec>,
    pub static_solids: Vec<bv::BitVec>,
    pub player: Player,
}

impl Level {
    pub fn load(&mut self, info_path: String) {
        let re = Regex::new(
            &(r"(.*)(Pos:\s*-?\d+\.?\d*, \s*-?\d+\.?\d*) (PosRemainder:\s*-?\d+\.?\d*, \s*-?\d+\.?\d*) (Speed:\s*-?\d+\.?\d*, \s*-?\d+\.?\d*)(.*)"
                .to_owned() + r"LightningUL:(.*)Bounds: \{(.*)\} Solids: (.*)"/* + r" LightningDR:(.*)" +
            r" SpikeUL:(.*) SpikeDR:(.*) SpikeDir:(.*)" +
            r" Wind:(.*) WTPos:(.*) WTPattern:(.*) WTWidth:(.*) WTHeight(.*)" +
            r" StarJumpUL:(.*) JThruUL:(.*?)"*/),
        )
        .unwrap();
        let data = &std::fs::read_to_string(info_path)
            .expect("Failed to read infodump file! Somehow uncaught");
        let caps = re.captures(data).unwrap();
        self.load_bounds(caps.get(7).unwrap().as_str().to_owned());
        self.static_death = vec![
            bv::bitvec![0; (self.bounds.dr.x - self.bounds.ul.x) as usize];
            (self.bounds.dr.y - self.bounds.ul.y) as usize
        ];
        self.static_solids = self.static_death.clone();
        self.load_solids(caps.get(8).unwrap().as_str().to_owned());
        self.load_spinners(caps.get(5).unwrap().as_str().to_owned());
        let mut img: RgbImage = ImageBuffer::new(
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
        img.save("testimg.png").unwrap();
        self.load_player(
            caps.get(2).unwrap().as_str().to_owned(),
            caps.get(3).unwrap().as_str().to_owned(),
            caps.get(4).unwrap().as_str().to_owned(),
        );
        //println!("X:{} Y:{} Width:{} Height:{}", self.bounds.ul.x, self.bounds.ul.y, self.bounds.dr.x, self.bounds.dr.y);
    }

    #[inline(always)]
    fn parse_f32(caps: &regex::Captures, num: usize) -> f32 {
        return caps.get(num).unwrap().as_str().parse::<f32>().unwrap();
    }

    fn get_pair(string: &str) -> Point {
        let re = Regex::new(r"(-?\d+\.?\d*), (-?\d+\.?\d*)").unwrap();
        let caps = re.captures(string).unwrap();
        Point::new(Self::parse_f32(&caps, 1), Self::parse_f32(&caps, 2))
    }

    fn grift_bv(dest: &mut Vec<bv::BitVec>, src: &Vec<bv::BitVec>, x: i32, y: i32) {
        if dest.is_empty() || src.is_empty() {
            return;
        }
        if y > dest.len() as i32 || x >= dest[0].len() as i32 || x + (src[0].len() as i32) < 0 {
            return;
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
    fn grift_line(dest: &mut Vec<bv::BitVec>, x: i32, y0: i32, y1: i32, switch: bool) {
        if switch {
            let temp: Vec<bv::BitVec> = vec![bv::bitvec![1; (y1 - y0).unsigned_abs() as usize]];
            Self::grift_bv(dest, &temp, if y0 < y1 { y0 } else { y1 }, x)
        } else {
            let temp: Vec<bv::BitVec> = vec![bv::bitvec![1; 1]; (y1 - y0).unsigned_abs() as usize];
            Self::grift_bv(dest, &temp, x, if y0 < y1 { y0 } else { y1 });
        }
    }

    fn circle_octant(
        dest: &mut Vec<bv::BitVec>, origin: Point, radius: f32, flip_x: i32, flip_y: i32,
        switch: bool,
    ) {
        let cx: f32;
        let cy: f32;

        if switch {
            cx = origin.y;
            cy = origin.x;
        } else {
            cx = origin.x;
            cy = origin.y;
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

        let mut start_y: f32 = y;
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
                start_y = y;
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

    fn grift_circle(dest: &mut Vec<bv::BitVec>, origin: Point, radius: f32) {
        Self::circle_octant(dest, origin, radius, 1, 1, false);
        Self::circle_octant(dest, origin, radius, 1, -1, false);
        Self::circle_octant(dest, origin, radius, -1, 1, false);
        Self::circle_octant(dest, origin, radius, -1, -1, false);
        Self::circle_octant(dest, origin, radius, 1, 1, true);
        Self::circle_octant(dest, origin, radius, 1, -1, true);
        Self::circle_octant(dest, origin, radius, -1, 1, true);
        Self::circle_octant(dest, origin, radius, -1, -1, true);
    }

    fn load_bounds(&mut self, bounds: String) {
        let re = Regex::new(r"X:(-*\d*) Y:(-*\d*) Width:(-*\d*) Height:(-*\d*)").unwrap();
        let caps = re.captures(&bounds).unwrap();
        self.bounds = Rect::new_xywh(
            Self::parse_f32(&caps, 1),
            Self::parse_f32(&caps, 2),
            Self::parse_f32(&caps, 3),
            Self::parse_f32(&caps, 4),
        );
    }

    //TODO: just use indicatif you fucking idiot
    fn load_spinners(&mut self, data: String) {
        let mut split: Vec<&str> = data.split('[').collect();
        split.remove(0);
        stdout().flush().unwrap();
        let mut circle: Vec<bv::BitVec> = vec![bv::bitvec![0; 12]; 12];
        Self::grift_circle(&mut circle, Point::new(6., 6.), 6.);
        for (i, p) in split.into_iter().enumerate() {
            let pair: Point = Self::get_pair(p);
            Self::grift_bv(
                &mut self.static_death,
                &circle,
                (pair.x - 6.) as i32,
                (pair.y - 6.) as i32,
            );
            Self::grift_bv(
                &mut self.static_death,
                &vec![bv::bitvec![1; 16]; 4],
                pair.x as i32 - 8 + self.bounds.ul.x as i32,
                pair.y as i32 + 5 + self.bounds.ul.y as i32,
            );
            stdout().flush().unwrap();
        }
    }

    fn load_solids(&mut self, data: String) {
        let rows: Vec<&str> = data.split(' ').collect();
        let tile: Vec<bv::BitVec> = vec![bv::bitvec![1; 8]; 8];
        for (y, row) in rows.iter().enumerate() {
            for x in 0..row.len() {
                if let Some(c) = row.chars().nth(x) {
                    if c != '0' && c != '\r' {
                        Self::grift_bv(&mut self.static_solids, &tile, x as i32 * 8, y as i32 * 8)
                    }
                }
            }
        }
    }

    fn load_player(&mut self, position: String, position_remainder: String, speed: String) {
        let pair1: Point = Self::get_pair(&position);
        let pair2: Point = Self::get_pair(&position_remainder);
        let pair3: Point = Self::get_pair(&speed);
        self.player = Player::new(pair3, pair1 + pair2);
    }
}
