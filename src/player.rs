use std::ops::Range;

use bitvec::prelude as bv;
use rayon::prelude::*;
use rstar::RTree;

use crate::colliders::{Axes, Collider, Direction, Rect};
use crate::level::Level;
use crate::point::Point;

const DELTATIME: f32 = 0.0166667;
const DELTATIME_RECIP: f32 = 1f32 / 0.0166667;

#[derive(Debug, Default)]
pub struct MovementPrecomputer {
    new_solids: Vec<u8>,
    solids: Vec<bool>,
    death: Vec<bool>,
    bounds: Rect,
}

// TODO: switch to using bitvecs probably, may be slower though
impl MovementPrecomputer {
    pub fn new(solids: &RTree<Collider>, death: &RTree<Collider>, bounds: Rect) -> Self {
        Self {
            new_solids: Self::precompute_solids_new(&bounds, solids),
            solids: Self::precompute_solids(&bounds, solids),
            death: Self::precompute_death(&bounds, death),
            bounds,
        }
    }

    // TODO: these two should prob be removed
    #[inline]
    fn get_solids_index(
        position: &Point, direction: Direction, amount: f32, bounds: &Rect,
    ) -> usize {
        let dir = match direction {
            Direction::Left => 0,
            Direction::Up => 1,
            Direction::Right => 2,
            Direction::Down => 3,
        };
        let point_i = (
            (position.x - bounds.ul.x).round() as i32,
            (position.y - bounds.ul.y).round() as i32,
        );
        let width = (bounds.dr.x - bounds.ul.x) as i32 + 1;
        let amount_i = amount.log2().floor() as i32;
        (((point_i.0 + point_i.1 * width) * 4 + dir) * 8 + amount_i) as usize
    }

    #[inline]
    fn get_index(&self, position: &Point, direction: Direction) -> usize {
        let dir = match direction {
            Direction::Left => 0,
            Direction::Up => 1,
            Direction::Right => 2,
            Direction::Down => 3,
        };
        let point_i = (
            (position.x - self.bounds.ul.x).round() as i32,
            (position.y - self.bounds.ul.y).round() as i32,
        );
        let width = (self.bounds.dr.x - self.bounds.ul.x) as i32 + 1;
        ((point_i.0 + point_i.1 * width) * 4 + dir) as usize
    }

    fn precompute_solids_new(bounds: &Rect, solids: &RTree<Collider>) -> Vec<u8> {
        let ul_i = (bounds.ul.x as i32, bounds.ul.y as i32);
        let dr_i = (bounds.dr.x as i32, bounds.dr.y as i32);
        let vals =
            itertools::iproduct!(ul_i.1..=dr_i.1, ul_i.0..=dr_i.0, 1..=4).collect::<Vec<_>>();
        vals.par_iter()
            .map(|(y, x, dir)| {
                let xf = *x as f32;
                let yf = *y as f32;
                let rect = match dir {
                    1 => {
                        Collider::Rectangular(Rect::new_xywh(xf - 255f32, yf, 8f32 + 255f32, 11f32))
                    }
                    2 => {
                        Collider::Rectangular(Rect::new_xywh(xf, yf - 255f32, 8f32, 11f32 + 255f32))
                    }
                    3 => Collider::Rectangular(Rect::new_xywh(xf, yf, 8f32 + 255f32, 11f32)),
                    4 => Collider::Rectangular(Rect::new_xywh(xf, yf, 8f32, 11f32 + 255f32)),
                    _ => unreachable!(),
                };
                // TODO: this is probably slow, needs to be optimized
                let mut intersected = solids
                    .locate_in_envelope_intersecting(&rect.to_aabb())
                    .collect::<Vec<_>>();
                match dir {
                    1 => intersected.sort_by(|ca, cb| cb.pos().x.partial_cmp(&ca.pos().x).unwrap()),
                    2 => intersected.sort_by(|ca, cb| cb.pos().y.partial_cmp(&ca.pos().y).unwrap()),
                    3 => intersected.sort_by(|ca, cb| ca.pos().x.partial_cmp(&cb.pos().x).unwrap()),
                    4 => intersected.sort_by(|ca, cb| ca.pos().y.partial_cmp(&cb.pos().y).unwrap()),
                    _ => unreachable!(),
                }
                match intersected.first() {
                    Some(c) => {
                        (match dir {
                            1 => xf - c.rect().unwrap().dr.x - 1f32,
                            2 => yf - c.rect().unwrap().dr.y - 1f32,
                            3 => c.rect().unwrap().ul.x - xf - 8f32,
                            4 => c.rect().unwrap().ul.y - yf - 11f32,
                            _ => unreachable!(),
                        }) as u8
                    }
                    None => 255u8,
                }
            })
            .collect::<Vec<_>>()
    }

    // TODO: use itertools iproduct macro here
    fn precompute_solids(bounds: &Rect, solids: &RTree<Collider>) -> Vec<bool> {
        let ul_i = (bounds.ul.x as i32, bounds.ul.y as i32);
        let dr_i = (bounds.dr.x as i32, bounds.dr.y as i32);
        let y_range = (ul_i.1..=dr_i.1).collect::<Vec<_>>();
        let x_range = (ul_i.0..=dr_i.0).collect::<Vec<_>>();
        let dir_range = &(1..=4).collect::<Vec<_>>();
        let amount_range = &(0..=7).collect::<Vec<_>>();
        y_range
            .par_iter()
            .flat_map(|y| {
                x_range.par_iter().flat_map(move |x| {
                    dir_range.par_iter().flat_map(move |dir| {
                        amount_range.par_iter().map(move |amount| {
                            let to_move = 2f32.powi(*amount);
                            let xf = *x as f32;
                            let yf = *y as f32;
                            let rect = match dir {
                                1 => Collider::Rectangular(Rect::new_xywh(
                                    xf - to_move,
                                    yf,
                                    8f32 + to_move,
                                    11f32,
                                )),
                                2 => Collider::Rectangular(Rect::new_xywh(
                                    xf,
                                    yf - to_move,
                                    8f32,
                                    11f32 + to_move,
                                )),
                                3 => Collider::Rectangular(Rect::new_xywh(
                                    xf,
                                    yf,
                                    8f32 + to_move,
                                    11f32,
                                )),
                                4 => Collider::Rectangular(Rect::new_xywh(
                                    xf,
                                    yf,
                                    8f32,
                                    11f32 + to_move,
                                )),
                                _ => unreachable!(),
                            };
                            solids
                                .locate_in_envelope_intersecting(&rect.to_aabb())
                                .next()
                                .is_some()
                        })
                    })
                })
            })
            .collect::<Vec<_>>()
    }

    // TODO: use itertools iproduct macro here
    fn precompute_death(bounds: &Rect, death: &RTree<Collider>) -> Vec<bool> {
        let ul_i = (bounds.ul.x as i32, bounds.ul.y as i32);
        let dr_i = (bounds.dr.x as i32, bounds.dr.y as i32);
        let y_range = (ul_i.1..=dr_i.1).collect::<Vec<_>>();
        let x_range = (ul_i.0..=dr_i.0).collect::<Vec<_>>();
        let dir_range = &(1..=4).collect::<Vec<_>>();
        y_range
            .par_iter()
            .flat_map(|y| {
                x_range.par_iter().flat_map(move |x| {
                    dir_range.par_iter().map(move |dir| {
                        // NOTE: dir will eventually be used for spikes
                        let rect =
                            Collider::Rectangular(Rect::new_xywh(*x as f32, *y as f32, 8f32, 9f32));
                        let result = death
                            .locate_in_envelope_intersecting(&rect.to_aabb())
                            .next();
                        match result {
                            None => false,
                            Some(Collider::Rectangular(_)) => true,
                            Some(circ) => circ.collide_check(&rect),
                        }
                    })
                })
            })
            .collect::<Vec<_>>()
    }

    pub fn get_new_solid(&self, position: &Point, direction: Direction) -> u8 {
        self.new_solids[self.get_index(position, direction)]
    }

    pub fn get_solid(&self, position: &Point, direction: Direction, amount: f32) -> bool {
        self.solids[Self::get_solids_index(position, direction, amount, &self.bounds)]
    }

    pub fn get_death(&self, position: &Point, direction: Direction) -> bool {
        self.death[self.get_index(position, direction)]
    }
}

pub enum FrameResult {
    Nothing,
    CheckpointHit,
    Death,
}

#[derive(Clone, Debug, Default)]
pub struct Player {
    pub speed: Point,
    pub retained: f32,
    pub retained_timer: i32,
    pub alive: bool,
    pub hurtbox: Collider,
    pub hitbox: Collider,
}

impl Player {
    pub fn new(speed: Point, position: Point) -> Self {
        Self {
            speed,
            retained: 0f32,
            retained_timer: 0,
            alive: true,
            hurtbox: Collider::Rectangular(Rect::new(
                Point::new(position.x - 4f32, position.y - 11f32),
                Point::new(position.x + 3f32, position.y - 3f32),
            )),
            hitbox: Collider::Rectangular(Rect::new(
                Point::new(position.x - 4f32, position.y - 11f32),
                Point::new(position.x + 3f32, position.y - 1f32),
            )),
        }
    }

    pub fn pos(&self) -> Point {
        let ul = match self.hurtbox.rect() {
            Some(rect) => rect,
            None => unreachable!(),
        }
        .ul;
        Point::new(ul.x + 4f32, ul.y + 11f32)
    }

    // TODO: add in stuff for when speed is outside octagon and should be pulled back to it
    // TODO: make speed capping actually work how its meant to
    // TODO: water surface bs
    pub fn speed_calc(&mut self, angle: f64, level: &Level) {
        let truncated = f64::round(angle * 1000f64) / 1000f64;
        let adjusted = Point::new(
            truncated.to_radians().sin() as f32,
            truncated.to_radians().cos() as f32,
        );
        self.retained_timer -= 1;
        let target = Point::new(60f32 * adjusted.x, 80f32 * adjusted.y);
        if f32::abs(target.x - self.speed.x) < 10f32 {
            self.speed.x = target.x;
        } else {
            self.speed.x += f32::clamp(target.x - self.speed.x, -10f32, 10f32);
        }
        if self.speed.x.signum() == self.retained.signum()
            && self.retained_timer > 0
            && level.precomputed.get_solid(
                &self.pos(),
                if self.speed.x.signum() < 0f32 {
                    Direction::Left
                } else {
                    Direction::Right
                },
                1f32,
            )
        {
            self.speed.x = self.retained;
            self.retained = 0f32;
            self.retained_timer = 0;
        }
        self.speed.x = self.speed.x.clamp(-60f32, 60f32);
        if f32::abs(target.y - self.speed.y) < 10f32 {
            self.speed.y = target.y;
        } else {
            self.speed.y += f32::clamp(target.y - self.speed.y, -10f32, 10f32);
        }
        self.speed.y = self.speed.y.clamp(-80f32, 80f32);
    }

    pub fn speed_calc_restricted(&mut self) {
        todo!()
    }

    // NOTE: this will replace the existing function when it's done
    fn move_in_direction_new(&mut self, level: &Level, speed: f32, dir: Direction) -> bool {
        let pixels_f = speed * DELTATIME;
        let pixels_i = match dir {
            Direction::Left | Direction::Right => {
                if self.pos().x.round() == f32::round(self.pos().x + pixels_f) {
                    pixels_f.floor()
                } else {
                    pixels_f.ceil()
                }
            }
            Direction::Up | Direction::Down => {
                if self.pos().y.round() == f32::round(self.pos().y + pixels_f) {
                    pixels_f.floor()
                } else {
                    pixels_f.ceil()
                }
            }
        }
        .abs();
        let mut to_move = level.precomputed.get_new_solid(&self.pos(), dir) as f32;
        if to_move > pixels_i {
            to_move = pixels_i
                + match dir {
                    Direction::Left | Direction::Right => self.pos().x.round() - self.pos().x,
                    Direction::Up | Direction::Down => self.pos().y.round() - self.pos().y,
                };
        }
        let (x, y) = match dir {
            Direction::Left => (-to_move * DELTATIME_RECIP, 0f32),
            Direction::Up => (0f32, -to_move * DELTATIME_RECIP),
            Direction::Right => (to_move * DELTATIME_RECIP, 0f32),
            Direction::Down => (0f32, to_move * DELTATIME_RECIP),
        };
        self.hitbox.move_collider(x, y);
        self.hurtbox.move_collider(x, y);
        to_move <= pixels_i
    }

    // NOTE: again, fallback prob needed, might implement later
    // NOTE: yes this uses both DELTATIME_RECIP and DELTATIME, but it works
    // TODO: this code is absolutely awful and needs to be cleaned up
    // TODO: ok actually i just have to redo this entire function
    fn move_in_direction(&mut self, level: &Level, speed: f32, dir: Direction) -> bool {
        if speed == 0f32 {
            return false;
        }
        let mut ret = false;
        let pixels = speed * DELTATIME;
        let pixels_f = pixels.fract();
        let pixels_l = pixels.log2() as i32;
        let mut pixels_max = pixels as i32;
        for a in (0..=pixels_l).rev() {
            let to_move = 2f32.powi(a);
            if level.precomputed.get_solid(&self.pos(), dir, to_move) {
                ret = true;
            } else {
                pixels_max -= 2i32.pow(a as u32);
                let (x, y) = match dir {
                    Direction::Left => (-to_move * DELTATIME_RECIP, 0f32),
                    Direction::Up => (0f32, -to_move * DELTATIME_RECIP),
                    Direction::Right => (to_move * DELTATIME_RECIP, 0f32),
                    Direction::Down => (0f32, to_move * DELTATIME_RECIP),
                };
                self.hurtbox.move_collider(x, y);
                self.hitbox.move_collider(x, y);
            }
            if pixels_max <= 0 {
                break;
            }
        }
        let to_move_temp = match dir {
            Direction::Left => Point::new(-pixels_f, 0f32),
            Direction::Up => Point::new(0f32, -pixels_f),
            Direction::Right => Point::new(pixels_f, 0f32),
            Direction::Down => Point::new(0f32, pixels_f),
        };
        let changed = self.pos() + to_move_temp;
        let to_move = if self.pos().x.round() != changed.x.round()
            || self.pos().y.round() != changed.y.round()
        {
            if level.precomputed.get_solid(&self.pos(), dir, 1f32) {
                ret = true;
                0f32
            } else {
                pixels_f
            }
        } else {
            pixels_f
        };
        let (x, y) = match dir {
            Direction::Left => (-to_move * DELTATIME_RECIP, 0f32),
            Direction::Up => (0f32, -to_move * DELTATIME_RECIP),
            Direction::Right => (to_move * DELTATIME_RECIP, 0f32),
            Direction::Down => (0f32, to_move * DELTATIME_RECIP),
        };
        self.hurtbox.move_collider(x, y);
        self.hitbox.move_collider(x, y);
        ret
    }

    pub fn move_self(&mut self, level: &Level) {
        if self.move_in_direction_new(
            level,
            self.speed.x,
            if self.speed.x <= 0f32 {
                Direction::Left
            } else {
                Direction::Right
            },
        ) {
            self.retained = self.speed.x;
            self.retained_timer = 4;
            self.speed.x = 0f32;
        }
        if self.move_in_direction_new(
            level,
            self.speed.y,
            if self.speed.y <= 0f32 {
                Direction::Up
            } else {
                Direction::Down
            },
        ) {
            self.speed.y = 0f32;
        }
    }

    // NOTE: there still needs to probably be a fallback here but that can be dealt with later
    pub fn collide(&mut self, level: &Level, checkpoint: &Rect) -> FrameResult {
        // looks messy, avoids allocations though
        if self.speed.x <= 0f32 {
            if level.precomputed.get_death(&self.hurtbox.pos(), Direction::Left) {
                return FrameResult::Death;
            }
        }
        if self.speed.x >= 0f32 {
            if level.precomputed.get_death(&self.hurtbox.pos(), Direction::Right) {
                return FrameResult::Death;
            }
        }
        if self.speed.y <= 0f32 {
            if level.precomputed.get_death(&self.hurtbox.pos(), Direction::Up) {
                return FrameResult::Death;
            }
        }
        if self.speed.y >= 0f32 {
            if level.precomputed.get_death(&self.hurtbox.pos(), Direction::Down) {
                return FrameResult::Death;
            }
        }
        if self
            .hitbox
            .collide_check(&Collider::Rectangular(*checkpoint))
        {
            FrameResult::CheckpointHit
        } else {
            FrameResult::Nothing
        }
    }

    // NOTE: this is really just here for when i need to implement a fallback
    // this might just get removed entirely
    pub fn solids_collision(&mut self, level: &Level, direction: Direction) -> bool {
        let hitbox_rect = match self.hitbox.rect() {
            Some(rect) => rect,
            None => unreachable!(),
        };
        let to_check = Collider::Rectangular(match direction {
            Direction::Left => Rect::new_xywh(hitbox_rect.ul.x, hitbox_rect.ul.y, 1f32, 11f32),
            Direction::Right => Rect::new_xywh(hitbox_rect.ur.x, hitbox_rect.ur.y, 1f32, 11f32),
            Direction::Up => Rect::new_xywh(hitbox_rect.ul.x, hitbox_rect.ul.y, 8f32, 1f32),
            Direction::Down => Rect::new_xywh(hitbox_rect.dl.x, hitbox_rect.dl.y, 8f32, 1f32),
        });
        level
            .solids
            .locate_in_envelope_intersecting(&to_check.to_aabb())
            .next()
            .is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn precompute_test_death() {
        let solids = RTree::bulk_load(vec![]);
        let death = RTree::bulk_load(vec![
            Collider::Rectangular(Rect::new_xywh(8f32, 0f32, 8f32, 8f32)),
            Collider::Rectangular(Rect::new_xywh(0f32, 8f32, 8f32, 8f32)),
        ]);
        let bounds = Rect::new_xywh(0f32, 0f32, 16f32, 16f32);
        let precomputer = MovementPrecomputer::new(&solids, &death, bounds);
        for y in 0..=15 {
            for x in 0..=15 {
                let expected = !(x >= 8 && y >= 8);
                assert_eq!(
                    precomputer.get_death(&Point::new(x as f32, y as f32), Direction::Left),
                    expected
                );
                assert_eq!(
                    precomputer.get_death(&Point::new(x as f32, y as f32), Direction::Up),
                    expected
                );
                assert_eq!(
                    precomputer.get_death(&Point::new(x as f32, y as f32), Direction::Right),
                    expected
                );
                assert_eq!(
                    precomputer.get_death(&Point::new(x as f32, y as f32), Direction::Down),
                    expected
                );
            }
        }
    }

    #[test]
    fn precompute_test_new_solids() {
        let death = RTree::bulk_load(vec![]);
        let solids = RTree::bulk_load(vec![
            Collider::Rectangular(Rect::new_xywh(-8f32, 0f32, 8f32, 8f32)),
            Collider::Rectangular(Rect::new_xywh(0f32, -9f32, 8f32, 8f32)),
            Collider::Rectangular(Rect::new_xywh(10f32, 0f32, 8f32, 8f32)),
            Collider::Rectangular(Rect::new_xywh(0f32, 15f32, 8f32, 8f32)),
        ]);
        let bounds = Rect::new_xywh(-8f32, -9f32, 27f32, 33f32);
        let precomputer = MovementPrecomputer::new(&solids, &death, bounds);
        for d in 0..=3 {
            let dir = match d {
                0 => Direction::Left,
                1 => Direction::Up,
                2 => Direction::Right,
                3 => Direction::Down,
                _ => unreachable!(),
            };
            assert_eq!(
                precomputer.get_new_solid(&Point::new(0f32, 0f32), dir),
                if d == 0 { 0 } else { 2u8.pow(d - 1) }
            );
        }
    }

    #[test]
    fn precompute_test_solids() {
        let death = RTree::bulk_load(vec![]);
        let solids = RTree::bulk_load(vec![
            Collider::Rectangular(Rect::new_xywh(-8f32, 0f32, 8f32, 8f32)),
            Collider::Rectangular(Rect::new_xywh(0f32, -9f32, 8f32, 8f32)),
            Collider::Rectangular(Rect::new_xywh(10f32, 0f32, 8f32, 8f32)),
            Collider::Rectangular(Rect::new_xywh(0f32, 15f32, 8f32, 8f32)),
        ]);
        let bounds = Rect::new_xywh(-8f32, -9f32, 27f32, 33f32);
        let precomputer = MovementPrecomputer::new(&solids, &death, bounds);
        for be_true in 0..=3 {
            for amount in 0..=7 {
                let dir = match be_true {
                    0 => Direction::Left,
                    1 => Direction::Up,
                    2 => Direction::Right,
                    3 => Direction::Down,
                    _ => unreachable!(),
                };
                assert_eq!(
                    precomputer.get_solid(&Point::new(0f32, 0f32), dir, f32::powi(2f32, amount)),
                    amount >= be_true
                );
            }
        }
    }

    #[test]
    fn precompute_test_solids_player() {
        let death = RTree::bulk_load(vec![]);
        let solids = RTree::bulk_load(vec![
            Collider::Rectangular(Rect::new_xywh(-8f32, 0f32, 8f32, 8f32)),
            Collider::Rectangular(Rect::new_xywh(0f32, -9f32, 8f32, 8f32)),
            Collider::Rectangular(Rect::new_xywh(10f32, 0f32, 8f32, 8f32)),
            Collider::Rectangular(Rect::new_xywh(0f32, 15f32, 8f32, 8f32)),
        ]);
        let bounds = Rect::new_xywh(-8f32, -9f32, 27f32, 33f32);
        let precomputer = MovementPrecomputer::new(&solids, &death, bounds);
        let mut level = Level::default();
        level.bounds = bounds;
        level.precomputed = precomputer;
        let mut player = Player::new(Point::new(0f32, 0f32), Point::new(0f32, 0f32));
        for be_true in 0..=3 {
            for amount in 0..=128 {
                let to_move = amount as f32 * DELTATIME_RECIP;
                match be_true {
                    0 => player.speed = Point::new(-to_move, 0f32),
                    1 => player.speed = Point::new(0f32, -to_move),
                    2 => player.speed = Point::new(to_move, 0f32),
                    3 => player.speed = Point::new(0f32, to_move),
                    _ => unreachable!(),
                }
                let to_move_expected = if be_true == 0 {
                    0f32
                } else {
                    amount.min(i32::pow(2, be_true - 1)) as f32
                };
                let expected_pos = match be_true {
                    0 => Point::new(-to_move_expected, 0f32),
                    1 => Point::new(0f32, -to_move_expected),
                    2 => Point::new(to_move_expected, 0f32),
                    3 => Point::new(0f32, to_move_expected),
                    _ => unreachable!(),
                };
                player.move_self(&level);
                assert_eq!(player.pos(), expected_pos);
                player = Player::new(Point::new(0f32, 0f32), Point::new(0f32, 0f32));
            }
        }
    }
}
