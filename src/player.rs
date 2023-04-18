use std::ops::Range;

use bitvec::prelude as bv;

use crate::colliders::{Axes, Collider, Direction, Rect};
use crate::level::Level;
use crate::point::Point;

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
                    speed_x.min(480f32)
                } else {
                    speed_x.max(-480f32)
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
                    speed_x = 0f32;
                }
                speed_x -= 480f32 * sign_x;
            } else {
                let speed = if sign_y > 0f32 {
                    speed_y.min(480f32)
                } else {
                    speed_y.max(-480f32)
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
                    break;
                }
                speed_y -= 480f32 * sign_y;
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
        // TODO: truncate to fit within valid inputs, precise_fix is too costly
        let adjusted = Point::new(
            angle.to_radians().sin() as f32,
            angle.to_radians().cos() as f32,
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
            self.hitbox.move_collider(self.speed.x.signum(), 0f32);
            if self.solids_collision(
                level,
                if self.speed.x.signum() < 0f32 {
                    Direction::Left
                } else {
                    Direction::Right
                },
            ) {
                self.speed.x = self.retained;
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
        let hurtbox_rect = match self.hurtbox.rect() {
            Some(rect) => rect,
            None => unreachable!(),
        };
        let left = (hurtbox_rect.ul.x - level.bounds.ul.x).round() as i32;
        let right = (hurtbox_rect.dr.x - level.bounds.ul.x).round() as i32 + 1;
        let up = (hurtbox_rect.ul.y - level.bounds.ul.y).round() as i32;
        let down = (hurtbox_rect.dr.y - level.bounds.ul.y).round() as i32 + 1;
        // println!("{} {} {} {}", left, right, up, down);
        for x in left..right {
            for y in up..down {
                if level.static_death[y as usize][x as usize] {
                    return FrameResult::Death;
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
        todo!()
    }

    pub fn solids_collision_old(
        &mut self, bounds: &Rect, static_solids: &[bv::BitVec], switch_xy: bool, switch_lr: bool,
    ) -> bool {
        let hitbox_rect = match self.hitbox.rect() {
            Some(rect) => rect,
            None => unreachable!(),
        };
        let left = (hitbox_rect.ul.x - bounds.ul.x).round() as usize;
        let right = (hitbox_rect.dr.x - bounds.ul.x).round() as usize + 1;
        let up = (hitbox_rect.ul.y - bounds.ul.y).round() as usize;
        let down = (hitbox_rect.dr.y - bounds.ul.y).round() as usize + 1;
        let first: Range<usize>;
        let second: Range<usize>;
        if switch_xy {
            first = up..down;
            second = left..right;
        } else {
            first = left..right;
            second = up..down;
        }
        let first_v = if switch_lr {
            first.rev().collect::<Vec<_>>()
        } else {
            first.collect::<Vec<_>>()
        };
        let second_v = if switch_lr {
            second.rev().collect::<Vec<_>>()
        } else {
            second.collect::<Vec<_>>()
        };
        let mut last_seen = -1;
        for f in &first_v {
            for s in &second_v {
                let b = if switch_xy {
                    static_solids[*f][*s]
                } else {
                    static_solids[*s][*f]
                };
                if b {
                    last_seen = *f as i32 - if switch_xy { up as i32 } else { left as i32 };
                }
            }
        }
        if last_seen == -1 {
            return false;
        }
        if !switch_lr {
            last_seen = first_v.len() as i32 - last_seen;
        } else {
            last_seen += 1;
        }
        if self.retained_timer == 0 && !switch_xy {
            self.retained = self.speed.x;
            self.retained_timer = 4;
        }
        let multiplier = if switch_lr { 60f32 } else { -60f32 };
        if switch_xy {
            self.hitbox
                .move_collider(0f32, last_seen as f32 * multiplier);
            self.hurtbox
                .move_collider(0f32, last_seen as f32 * multiplier);
            self.speed.y = 0f32;
        } else {
            self.hitbox
                .move_collider(last_seen as f32 * multiplier, 0f32);
            self.hurtbox
                .move_collider(last_seen as f32 * multiplier, 0f32);
            self.speed.x = 0f32;
        }
        if !switch_xy {
            self.hitbox.reset_subpixels(Axes::Horizontal);
            self.hurtbox.reset_subpixels(Axes::Horizontal);
        } else {
            self.hitbox.reset_subpixels(Axes::Vertical);
            self.hurtbox.reset_subpixels(Axes::Vertical);
        }
        true
    }
}

#[cfg(test)]
mod tests {}
