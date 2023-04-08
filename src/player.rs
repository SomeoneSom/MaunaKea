use std::ops::Range;

use bitvec::prelude as bv;

use crate::colliders::{Axes, Collider, Rect};
use crate::level::Level;
use crate::point::Point;

fn precise_fix(angle: f64, magnitude: f32) -> Point {
    const DEADZONE: f64 = 0.239532471;
    const AMPLOWERBOUND: f32 = (0.25 * (1f64 - DEADZONE) * (1f64 - DEADZONE)) as f32;
    const LOWERBOUND: i16 = 7849;
    let angle = Point::new(angle.sin() as f32, angle.cos() as f32);
    let raw_angle = if angle.x.abs() > angle.y.abs() {
        angle
    } else {
        Point::new(angle.y, angle.x)
    };
    let upper_bound = i16::max((magnitude * 32767f32) as i16, LOWERBOUND);
    let approx = (raw_angle.y.abs() as f64) / (raw_angle.x.abs() as f64);
    let multip = (raw_angle.x.abs() as f64) / (raw_angle.y.abs() as f64);
    let upperl = upper_bound as f64 / 32767f64;
    let mut least_error = approx;
    let mut short_x = upper_bound;
    let mut short_y = 0i16;
    let mut y = LOWERBOUND;
    loop {
        let ys = y as f64 / 32767f64 - DEADZONE;
        let xx = f64::min(DEADZONE + multip * ys, upperl);
        let mut x = f64::floor(xx * 32767f64) as i16;
        let mut xs = x as f64 / 32767f64 - DEADZONE;
        let mut error = f64::abs(ys / xs - approx);
        if xs * xs + ys * ys >= AMPLOWERBOUND as f64 && error < least_error {
            least_error = error;
            short_x = x;
            short_y = y;
        }
        if x < upper_bound {
            x += 1;
            xs = x as f64 / 32767f64 - DEADZONE;
            error = f64::abs(ys / xs - approx);
            if xs * xs + ys * ys >= AMPLOWERBOUND as f64 && error < least_error {
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
    let final_x = raw_angle.x.signum()
        * (f64::max(short_x as f64 / 32767f64 - DEADZONE, 0f64) / (1f64 - DEADZONE)) as f32;
    let final_y = raw_angle.y.signum()
        * (f64::max(short_y as f64 / 32767f64 - DEADZONE, 0f64) / (1f64 - DEADZONE)) as f32;
    if angle.x.abs() > angle.y.abs() {
        Point::new(final_x, final_y)
    } else {
        Point::new(final_y, final_x)
    }
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

    pub fn move_self(&mut self, angle: f64, level: &Level) {
        let adjusted = precise_fix(angle.to_radians(), 1f32);
        self.speed_calc(adjusted, level);
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
                if self.solids_collision(&level.bounds, &level.static_solids, false, sign_x < 0f32)
                {
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
        let target = Point::new(60f32 * vector.x, 80f32 * vector.y);
        if f32::abs(target.x - self.speed.x) < 10f32 {
            self.speed.x = target.x;
        } else {
            self.speed.x += f32::clamp(target.x - self.speed.x, -10f32, 10f32);
        }
        if self.speed.x.signum() == self.retained.signum() && self.retained_timer > 0 {
            let temp_hitbox = self.hitbox;
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
        &mut self, angle: f64, bounds: &Rect, static_death: &[bv::BitVec],
        static_solids: &[bv::BitVec], checkpoint: &Rect, level: &Level,
    ) -> f32 {
        self.move_self(angle, level);
        let hurtbox_rect = match self.hurtbox.rect() {
            Some(rect) => rect,
            None => unreachable!(),
        };
        let left = (hurtbox_rect.ul.x - bounds.ul.x).round() as i32;
        let right = (hurtbox_rect.dr.x - bounds.ul.x).round() as i32 + 1;
        let up = (hurtbox_rect.ul.y - bounds.ul.y).round() as i32;
        let down = (hurtbox_rect.dr.y - bounds.ul.y).round() as i32 + 1;
        // println!("{} {} {} {}", left, right, up, down);
        for x in left..right {
            for y in up..down {
                if static_death[y as usize][x as usize] {
                    return 9999999f32;
                }
            }
        }
        // TODO: make it so this actually calcs distance properly
        let player_cent = (hurtbox_rect.ul + hurtbox_rect.dr) / 2f32;
        let check_cent = Point::new(
            (checkpoint.ul.x + checkpoint.dr.x) / 2f32,
            (checkpoint.ul.y + checkpoint.dr.y) / 2f32,
        );
        f32::sqrt((player_cent.x - check_cent.x).powi(2) + (player_cent.y - check_cent.y).powi(2))
    }

    pub fn solids_collision(
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
mod tests {
    use crate::point::Point;

    #[test]
    fn precise_fix_test() {
        assert_eq!(
            super::precise_fix(67.53154007f64.to_radians(), 1f32),
            Point::new(0.4807418, 0.1988198)
        );
        assert_eq!(
            super::precise_fix(270.27667232f64.to_radians(), 1f32),
            Point::new(-0.6336017, 0.003059587)
        );
        assert_eq!(
            super::precise_fix(200.622924623f64.to_radians(), 1f32),
            Point::new(-0.25680944, -0.6824013)
        );
        assert_eq!(
            super::precise_fix(156.9761642057f64.to_radians(), 1f32),
            Point::new(0.4182976, -0.98430866)
        );
        assert_eq!(
            super::precise_fix(347.3860480961f64.to_radians(), 1f32),
            Point::new(-0.21728018, 0.970945)
        );
        assert_eq!(
            super::precise_fix(83.9673004749f64.to_radians(), 1f32),
            Point::new(0.9904488, 0.1046719)
        );
        assert_eq!(
            super::precise_fix(230.307557263f64.to_radians(), 1f32),
            Point::new(-0.94068605, -0.78076303)
        );
        assert_eq!(
            super::precise_fix(191.2593688182f64.to_radians(), 1f32),
            Point::new(-0.16723652, -0.84003687)
        );
    }
}
