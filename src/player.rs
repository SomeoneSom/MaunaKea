use std::ops::Range;

use bitvec::prelude as bv;

use crate::colliders::{Axes, Collider, Rect};
use crate::level::Level;
use crate::point::Point;

fn precise_fix(angle: f32, magnitude: f32) -> Point {
    const DEADZONE: f64 = 0.239532471;
    const AMPLOWERBOUND: f32 = (0.25 * (1f64 - DEADZONE) * (1f64 - DEADZONE)) as f32;
    const LOWERBOUND: i16 = 7849;
    let raw_angle = Point::new(angle.cos(), angle.sin());
    let upper_bound = i16::max((magnitude * 32767f32) as i16, LOWERBOUND);
    let approx = (raw_angle.x as f64) / (raw_angle.y as f64);
    let multip = (raw_angle.y as f64) / (raw_angle.x as f64);
    let upperl = (upper_bound / 32767) as f64;
    let mut least_error = approx;
    let mut short_x = 32767i16;
    let mut short_y = 0i16;
    let mut y = LOWERBOUND;
    loop {
        let ys = y as f64 / 32767f64 - DEADZONE;
        let xx = f64::min(DEADZONE + multip * ys, upperl);
        let mut x = f64::floor(xx * 32767f64) as i16;
        let mut xs = x as f64 / 32767f64 - DEADZONE;
        let mut error = f64::abs(ys / xs - approx);
        if xs * xs + ys * ys >= AMPLOWERBOUND as f64 && (error < least_error || error <= 0.5e-10) {
            least_error = error;
            short_x = x;
            short_y = y;
        }
        if x < upper_bound {
            x += 1;
            xs = x as f64 / 32767f64 - DEADZONE;
            error = f64::abs(ys / xs - approx);
            if xs * xs + ys * ys >= AMPLOWERBOUND as f64
                && (error < least_error || error <= 0.5e-10)
            {
                least_error = error;
                short_x = x;
                short_y = y;
            }
        }
        if xx >= upperl {
            break;
        }
        y += 1;
    }
    let final_x = short_x.signum() as f32
        * (f64::max(f64::abs(short_x as f64) / 32767f64 - DEADZONE, 0f64) / (1f64 - DEADZONE))
            as f32;
    let final_y = short_y.signum() as f32
        * (f64::max(f64::abs(short_y as f64) / 32767f64 - DEADZONE, 0f64) / (1f64 - DEADZONE))
            as f32;
    Point::new(final_x, final_y)
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

    pub fn move_self(&mut self, angle: f32, level: &Level) {
        let mut ang = angle - 90f32;
        if ang < 0f32 {
            ang += 360f32;
        }
        ang = 360f32 - ang;
        let rang = ang.to_radians();
        let adjusted = precise_fix(rang, 1f32);
        self.speed_calc(adjusted, level);
        let mut speed_x: f32 = self.speed.x;
        let mut speed_y: f32 = self.speed.y;
        let sign_x: f32 = speed_x.signum();
        let sign_y: f32 = speed_y.signum();
        loop {
            if speed_x * sign_x > 0f32 {
                let speed: f32 = if sign_x > 0f32 {
                    speed_x.min(480f32)
                } else {
                    speed_x.max(-480f32)
                };
                self.hurtbox.move_collider(speed, 0f32);
                self.hitbox.move_collider(speed, 0f32);
                if self.solids_collision(&level.bounds, &level.static_solids, false, sign_x < 0f32)
                {
                    speed_x = 0f32;
                }
                speed_x -= 480f32 * sign_x;
            } else {
                let speed: f32 = if sign_y > 0f32 {
                    speed_y.min(480f32)
                } else {
                    speed_y.max(-480f32)
                };
                self.hurtbox.move_collider(0f32, speed);
                self.hitbox.move_collider(0f32, speed);
                if self.solids_collision(&level.bounds, &level.static_solids, true, sign_y < 0f32) {
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
    pub fn speed_calc(&mut self, vector: Point, level: &Level) {
        self.retained_timer -= 1;
        let target: Point = Point::new(60f32 * vector.x, 80f32 * vector.y);
        if f32::abs(target.x - self.speed.x) < 10f32 {
            self.speed.x = target.x;
        } else {
            self.speed.x += f32::clamp(target.x - self.speed.x, -10f32, 10f32);
        }
        if self.speed.x.signum() == self.retained.signum() && self.retained_timer > 0 {
            let temp_hitbox: Collider = self.hitbox;
            self.hitbox.move_collider(self.speed.x.signum(), 0f32);
            if self.solids_collision(
                &level.bounds,
                &level.static_solids,
                false,
                self.speed.x.signum() < 0f32,
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

    pub fn collision_check() -> bool {
        todo!()
    }

    // this function is getting removed soon im pretty sure
    pub fn sim_frame_legacy(
        &mut self, angle: f32, bounds: &Rect, static_death: &[bv::BitVec],
        static_solids: &[bv::BitVec], checkpoint: &Rect, level: &Level,
    ) -> f32 {
        self.move_self(angle, level);
        let hurtbox_rect: &Rect = self.hurtbox.rect().unwrap();
        let left: i32 = (hurtbox_rect.ul.x - bounds.ul.x).round() as i32;
        let right: i32 = (hurtbox_rect.dr.x - bounds.ul.x).round() as i32 + 1;
        let up: i32 = (hurtbox_rect.ul.y - bounds.ul.y).round() as i32;
        let down: i32 = (hurtbox_rect.dr.y - bounds.ul.y).round() as i32 + 1;
        // println!("{} {} {} {}", left, right, up, down);
        for x in left..right {
            for y in up..down {
                if static_death[y as usize][x as usize] {
                    return 9999999f32;
                }
            }
        }
        // TODO: make it so this actually calcs distance properly
        let rect = self.hurtbox.rect().unwrap();
        let player_cent: Point = (rect.ul + rect.dr) / 2f32;
        let check_cent: Point = Point::new(
            (checkpoint.ul.x + checkpoint.dr.x) / 2f32,
            (checkpoint.ul.y + checkpoint.dr.y) / 2f32,
        );
        f32::sqrt((player_cent.x - check_cent.x).powi(2) + (player_cent.y - check_cent.y).powi(2))
    }

    pub fn solids_collision(
        &mut self, bounds: &Rect, static_solids: &[bv::BitVec], switch_xy: bool, switch_lr: bool,
    ) -> bool {
        let hitbox_rect = self.hitbox.rect().unwrap();
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
                let b: bool = if switch_xy {
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
mod tests {
    #[test]
    fn precise_fix_test() {
        assert_eq!(2 + 2, 4);
    }
}
