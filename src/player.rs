use std::ops::Range;

use bitvec::prelude as bv;
use rayon::prelude::*;
use rstar::RTree;

use crate::colliders::{Axes, Collider, Direction, Rect};
use crate::level::Level;
use crate::point::Point;

#[derive(Debug, Default)]
pub struct MovementPrecomputer {
    solids: Vec<bool>,
    death: Vec<bool>,
}

// TODO: switch to using bitvecs probably, may be slower though
impl MovementPrecomputer {
    pub fn new(bounds: &Rect, solids: &RTree<Collider>, death: &RTree<Collider>) -> Self {
        Self {
            solids: Self::precompute_solids(bounds, solids),
            death: Self::precompute_death(bounds, death),
        }
    }

    //TODO: these two should prob be removed
    #[inline]
    fn get_solids_index(
        position: &Point, direction: Direction, amount: f32, bounds: &Rect,
    ) -> usize {
        let dir = match direction {
            Direction::Left => 1,
            Direction::Up => 2,
            Direction::Right => 3,
            Direction::Down => 4,
        };
        let point_i = (
            (position.x - bounds.ul.x).round() as i32,
            (position.y - bounds.ul.y).round() as i32,
        );
        let width = (bounds.dr.x - bounds.ul.x) as i32;
        let amount_i = amount.log2().floor() as i32;
        ((point_i.0 + point_i.1 * width) * dir * amount_i) as usize
    }

    #[inline]
    fn get_death_index(position: &Point, direction: Direction, bounds: &Rect) -> usize {
        let dir = match direction {
            Direction::Left => 1,
            Direction::Up => 2,
            Direction::Right => 3,
            Direction::Down => 4,
        };
        let point_i = (
            (position.x - bounds.ul.x).round() as i32,
            (position.y - bounds.ul.y).round() as i32,
        );
        let width = (bounds.dr.x - bounds.ul.x) as i32;
        (point_i.0 + point_i.1 * width * dir) as usize
    }

    fn precompute_solids(bounds: &Rect, solids: &RTree<Collider>) -> Vec<bool> {
        let ul_i = (bounds.ul.x as i32, bounds.ul.y as i32);
        let dr_i = (bounds.dr.x as i32, bounds.dr.y as i32);
        let width = dr_i.0 - ul_i.0;
        let mut out = vec![];
        for x in ul_i.0..dr_i.0 {
            for y in ul_i.1..dr_i.1 {
                for dir in 1..=4 {
                    for amount in 0..=8 {
                        out.push((x, y, dir, amount));
                    }
                }
            }
        }
        out.par_iter()
            .map(|(x, y, dir, amount)| {
                let rect = Collider::Rectangular(Rect::new_xywh(*x as f32, *y as f32, 8f32, 9f32));
                true
            })
            .collect::<Vec<_>>()
    }

    fn precompute_death(bounds: &Rect, death: &RTree<Collider>) -> Vec<bool> {
        todo!()
    }

    pub fn get_solid(
        &self, position: &Point, direction: Direction, amount: f32, bounds: &Rect,
    ) -> bool {
        self.solids[Self::get_solids_index(position, direction, amount, bounds)]
    }

    pub fn get_death(&self, position: &Point, direction: Direction, bounds: &Rect) -> bool {
        self.solids[Self::get_death_index(position, direction, bounds)]
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

    pub fn move_self(&mut self, level: &Level) {
        let mut speed_x = self.speed.x;
        let mut speed_y = self.speed.y;
        let sign_x = speed_x.signum();
        let sign_y = speed_y.signum();
        loop {
            if speed_x * sign_x > 0f32 {
                let speed = if sign_x > 0f32 {
                    speed_x.min(60f32)
                } else {
                    speed_x.max(-60f32)
                };
                self.hurtbox.move_collider(speed, 0f32);
                self.hitbox.move_collider(speed, 0f32);
                if self.solids_collision(
                    level,
                    if sign_x < 0f32 {
                        Direction::Left
                    } else {
                        Direction::Right
                    },
                ) {
                    self.hurtbox.move_collider(-speed, 0f32);
                    self.hitbox.move_collider(-speed, 0f32);
                    self.retained = self.speed.x;
                    self.retained_timer = 4;
                    self.speed.x = 0f32;
                    speed_x = 0f32;
                }
                speed_x -= 60f32 * sign_x;
            } else {
                let speed = if sign_y > 0f32 {
                    speed_y.min(60f32)
                } else {
                    speed_y.max(-60f32)
                };
                self.hurtbox.move_collider(0f32, speed);
                self.hitbox.move_collider(0f32, speed);
                if self.solids_collision(
                    level,
                    if sign_y < 0f32 {
                        Direction::Up
                    } else {
                        Direction::Down
                    },
                ) {
                    self.hurtbox.move_collider(0f32, -speed);
                    self.hitbox.move_collider(0f32, -speed);
                    self.speed.y = 0f32;
                    break;
                }
                speed_y -= 60f32 * sign_y;
            }
            if speed_x * sign_x <= 0f32 && speed_y * sign_y <= 0f32 {
                break;
            }
        }
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
        if self.speed.x.signum() == self.retained.signum() && self.retained_timer > 0 {
            let temp_hitbox = self.hitbox;
            self.hitbox
                .move_collider(self.speed.x.signum() * 60f32, 0f32);
            if self.solids_collision(
                level,
                if self.speed.x.signum() < 0f32 {
                    Direction::Left
                } else {
                    Direction::Right
                },
            ) {
                self.speed.x = self.retained;
                self.retained_timer = 0;
            }
            self.hitbox = temp_hitbox;
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

    pub fn collide(&mut self, level: &Level, checkpoint: &Rect) -> FrameResult {
        for collider in level
            .death
            .locate_in_envelope_intersecting(&self.hurtbox.to_aabb())
        {
            match collider {
                Collider::Rectangular(_) => return FrameResult::Death,
                Collider::Circular(_) => {
                    if self.hurtbox.collide_check(collider) {
                        return FrameResult::Death;
                    }
                }
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
mod tests {}
