use bitvec::prelude as bv;
use colored::Colorize;
use image::{ImageBuffer, Rgb, RgbImage};
use quadtree_rs::{area::AreaBuilder, point::Point as QTPoint, Quadtree};
use regex::Regex;
use rstar::RTree;

use std::io::stdout;
use std::io::Write;

use crate::colliders::Circle;
use crate::colliders::{Collider, Rect};
use crate::player::Player;
use crate::point::Point;

#[derive(Debug)]
pub struct Level {
    pub bounds: Rect,
    pub qt_solids: Quadtree<i32, Collider>,
    pub qt_death: Quadtree<i32, Collider>,
    pub solids: RTree<Collider>,
    pub death: RTree<Collider>,
    temp_solids: Vec<Collider>,
    temp_death: Vec<Collider>,
    pub static_death: Vec<bv::BitVec>,
    pub static_solids: Vec<bv::BitVec>,
}

impl Default for Level {
    fn default() -> Self {
        Self {
            bounds: Rect::default(),
            qt_solids: Quadtree::new(1),
            qt_death: Quadtree::new(1),
            solids: RTree::default(),
            death: RTree::default(),
            temp_solids: vec![],
            temp_death: vec![],
            static_death: vec![],
            static_solids: vec![],
        }
    }
}

impl Level {
    pub fn load(info_path: &str) -> (Level, Player) {
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
        // TODO: make this not have to be mutable
        let mut level = Self::default();
        level.load_bounds(caps.get(7).unwrap().as_str().to_owned());
        let anchor = QTPoint {
            x: level.bounds.ul.x as i32,
            y: level.bounds.ul.y as i32,
        };
        let depth_x = f32::ceil(f32::log2(level.bounds.dr.x - level.bounds.ul.x)) as usize;
        let depth_y = f32::ceil(f32::log2(level.bounds.dr.y - level.bounds.ul.y)) as usize;
        level.qt_solids = Quadtree::new_with_anchor(anchor, usize::max(depth_x, depth_y));
        level.qt_death = Quadtree::new_with_anchor(anchor, usize::max(depth_x, depth_y));
        level.static_death = vec![
            bv::bitvec![0; (level.bounds.dr.x - level.bounds.ul.x) as usize];
            (level.bounds.dr.y - level.bounds.ul.y) as usize
        ];
        level.static_solids = level.static_death.clone();
        level.load_solids(caps.get(8).unwrap().as_str().to_owned());
        level.load_spinners(caps.get(5).unwrap().as_str().to_owned());
        level.solids = RTree::bulk_load(level.temp_solids.clone());
        level.death = RTree::bulk_load(level.temp_death.clone());
        level.temp_solids = vec![];
        level.temp_death = vec![];
        /*let mut img = ImageBuffer::new(
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
        (
            level,
            Self::load_player(
                caps.get(2).unwrap().as_str().to_owned(),
                caps.get(3).unwrap().as_str().to_owned(),
                caps.get(4).unwrap().as_str().to_owned(),
            ),
        )
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
                    let start = f32::clamp(x as f32, 0f32, dest.len() as f32 - 1f32) as usize;
                    let end = f32::clamp((x + src[0].len() as i32) as f32, 0f32, dest.len() as f32)
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
            let temp = vec![bv::bitvec![1; (y1 - y0).unsigned_abs() as usize]];
            Self::grift_bv(dest, &temp, if y0 < y1 { y0 } else { y1 }, x)
        } else {
            let temp = vec![bv::bitvec![1; 1]; (y1 - y0).unsigned_abs() as usize];
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
            x = f32::ceil(cx + radius - 1f32);
        } else {
            x = f32::floor(cx - radius + 1f32);
        }

        let mut y: f32;
        if flip_y > 0 {
            y = cy.floor();
        } else {
            y = cy.ceil();
        }

        let mut start_y = y;
        let mut e = (x - cx).powi(2) + (y - cy).powi(2) - radius.powi(2);
        let mut yc = flip_y as f32 * 2f32 * (y - cy) + 1f32;
        let mut xc = flip_x as f32 * -2f32 * (x - cx) + 1f32;

        while flip_y as f32 * (y - cy) <= flip_x as f32 * (x - cx) {
            e += yc;
            y += flip_y as f32;
            yc += 2f32;
            if e >= 0f32 {
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
                xc += 2f32;
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

    // TODO: just use indicatif you fucking idiot
    fn load_spinners(&mut self, data: String) {
        let mut split = data.split('[').collect::<Vec<_>>();
        split.remove(0);
        for (i, p) in split.into_iter().enumerate() {
            let pair = Self::get_pair(p);
            self.temp_death.push(Collider::Circular(Circle::new(6f32, pair)));
            self.temp_death.push(
                Collider::Rectangular(Rect::new_xywh(pair.x - 8f32, pair.y + 5f32, 16f32, 4f32))
            );
        }
    }

    fn load_solids(&mut self, data: String) {
        let rows = data.split(' ').collect::<Vec<_>>();
        for (y, row) in rows.iter().enumerate() {
            for x in 0..row.len() {
                if let Some(c) = row.chars().nth(x) {
                    if c != '0' && c != '\r' {
                        let tile = Collider::Rectangular(Rect::new_xywh(
                            (x as f32 + self.bounds.ul.x) * 8f32,
                            (y as f32 + self.bounds.ul.y) * 8f32,
                            8f32,
                            8f32,
                        ));
                        self.temp_solids.push(tile);
                    }
                }
            }
        }
    }

    fn load_player(position: String, position_remainder: String, speed: String) -> Player {
        let pair1 = Self::get_pair(&position);
        let pair2 = Self::get_pair(&position_remainder);
        let pair3 = Self::get_pair(&speed);
        Player::new(pair3, pair1 + pair2)
    }
}

#[cfg(test)]
mod tests {}
