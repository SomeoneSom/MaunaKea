use std::ops::Range;

use bitvec::prelude as bv;

use crate::colliders::{Axes, Collider, Rect};
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
    let final_x = short_x.signum() * (f64::max(f64::abs(short_x as f64) / 32767f64 - DEADZONE, 0f64) / (1f64 - DEADZONE)) as f32;
    let final_y = short_y.signum() * (f64::max(f64::abs(short_y as f64) / 32767f64 - DEADZONE, 0f64) / (1f64 - DEADZONE)) as f32;
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
            retained: 0.,
            retained_timer: 0,
            alive: true,
            hurtbox: Collider::Rectangular(Rect::new(
                Point::new(position.x - 4., position.y - 11.),
                Point::new(position.x + 3., position.y - 3.),
            )),
            hitbox: Collider::Rectangular(Rect::new(
                Point::new(position.x - 4., position.y - 11.),
                Point::new(position.x + 3., position.y - 1.),
            )),
        }
    }

    pub fn move_self(&mut self, angle: i32, bounds: &Rect, static_solids: &[bv::BitVec]) {
        // TODO: add in stuff for when speed is outside octagon and should be pulled back to it
        // TODO: make speed capping actually work how its meant to
        // TODO: water surface bs
        self.retained_timer -= 1;
        let mut ang: i32 = angle - 90000;
        if ang < 0 {
            ang += 360000;
        }
        ang = 360000 - ang;
        let rang: f32 = (ang as f32 / 1000.).to_radians();
        let adjusted = precise_fix(rang, 1f32);
        let target: Point = Point::new(60f32 * adjusted.x, 80f32 * adjusted.y);
        if f32::abs(target.x - self.speed.x) < 10. {
            self.speed.x = target.x;
        } else {
            self.speed.x += f32::clamp(target.x - self.speed.x, -10., 10.);
        }
        if self.speed.x.signum() == self.retained.signum() && self.retained_timer > 0 {
            let temp_hitbox: Collider = self.hitbox;
            self.hitbox.move_collider(self.speed.x.signum(), 0.);
            if self.solids_collision(bounds, static_solids, false, self.speed.x.signum() < 0.) {
                self.speed.x = self.retained;
            }
            self.hitbox = temp_hitbox;
        }
        self.speed.x = self.speed.x.clamp(-60., 60.);
        if f32::abs(target.y - self.speed.y) < 10. {
            self.speed.y = target.y;
        } else {
            self.speed.y += f32::clamp(target.y - self.speed.y, -10., 10.);
        }
        self.speed.y = self.speed.y.clamp(-80., 80.);
        let mut speed_x: f32 = self.speed.x;
        let mut speed_y: f32 = self.speed.y;
        let sign_x: f32 = speed_x.signum();
        let sign_y: f32 = speed_y.signum();
        loop {
            if speed_x * sign_x > 0. {
                let speed: f32 = if sign_x > 0. {
                    speed_x.min(480.)
                } else {
                    speed_x.max(-480.)
                };
                self.hurtbox.move_collider(speed, 0.);
                self.hitbox.move_collider(speed, 0.);
                if self.solids_collision(bounds, static_solids, false, sign_x < 0.) {
                    speed_x = 0.;
                }
                speed_x -= 480. * sign_x;
            } else {
                let speed: f32 = if sign_y > 0. {
                    speed_y.min(480.)
                } else {
                    speed_y.max(-480.)
                };
                self.hurtbox.move_collider(0., speed);
                self.hitbox.move_collider(0., speed);
                if self.solids_collision(bounds, static_solids, true, sign_y < 0.) {
                    break;
                }
                speed_y -= 480. * sign_y;
            }
            if speed_x * sign_x <= 0. && speed_y * sign_y <= 0. {
                break;
            }
        }
    }

    pub fn move_self_restricted() {
        todo!()
    }

    pub fn collision_check() -> bool {
        todo!()
    }

    // this function is getting removed soon im pretty sure
    pub fn sim_frame(
        &mut self, angle: i32, bounds: &Rect, static_death: &[bv::BitVec],
        static_solids: &[bv::BitVec], checkpoint: &Rect,
    ) -> f32 {
        let temp_speed = self.speed;
        let temp_hurtbox = self.hurtbox;
        let temp_hitbox = self.hitbox;
        let temp_retained = self.retained;
        let temp_retained_timer = self.retained_timer;
        self.move_self(angle, bounds, static_solids);
        self.retained = temp_retained;
        self.retained_timer = temp_retained_timer;
        let hurtbox_rect: &Rect = self.hurtbox.rect().unwrap();
        let left: i32 = (hurtbox_rect.ul.x - bounds.ul.x).round() as i32;
        let right: i32 = (hurtbox_rect.dr.x - bounds.ul.x).round() as i32 + 1;
        let up: i32 = (hurtbox_rect.ul.y - bounds.ul.y).round() as i32;
        let down: i32 = (hurtbox_rect.dr.y - bounds.ul.y).round() as i32 + 1;
        // println!("{} {} {} {}", left, right, up, down);
        for x in left..right {
            for y in up..down {
                if static_death[y as usize][x as usize] {
                    self.speed = temp_speed;
                    self.hurtbox = temp_hurtbox;
                    self.hitbox = temp_hitbox;
                    return 9999999.;
                }
            }
        }
        // TODO: make it so this actually calcs distance properly
        let rect = self.hurtbox.rect().unwrap();
        let player_cent: Point = (rect.ul + rect.dr) / 2.;
        let check_cent: Point = Point::new(
            (checkpoint.ul.x + checkpoint.dr.x) / 2.,
            (checkpoint.ul.y + checkpoint.dr.y) / 2.,
        );
        self.speed = temp_speed;
        self.hurtbox = temp_hurtbox;
        self.hitbox = temp_hitbox;
        f32::sqrt((player_cent.x - check_cent.x).powi(2) + (player_cent.y - check_cent.y).powi(2))
    }

    pub fn solids_collision(
        &mut self, bounds: &Rect, static_solids: &[bv::BitVec], switch_xy: bool, switch_lr: bool,
    ) -> bool {
        let hitbox_rect: &Rect = self.hitbox.rect().unwrap();
        let left: usize = (hitbox_rect.ul.x - bounds.ul.x).round() as usize;
        let right: usize = (hitbox_rect.dr.x - bounds.ul.x).round() as usize + 1;
        let up: usize = (hitbox_rect.ul.y - bounds.ul.y).round() as usize;
        let down: usize = (hitbox_rect.dr.y - bounds.ul.y).round() as usize + 1;
        let first: Range<usize>;
        let second: Range<usize>;
        if switch_xy {
            first = up..down;
            second = left..right;
        } else {
            first = left..right;
            second = up..down;
        }
        let first_v: Vec<usize> = if switch_lr {
            first.rev().collect::<Vec<_>>()
        } else {
            first.collect::<Vec<_>>()
        };
        let second_v: Vec<usize> = if switch_lr {
            second.rev().collect::<Vec<_>>()
        } else {
            second.collect::<Vec<_>>()
        };
        let mut last_seen: i32 = -1;
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
        let multiplier: f32 = if switch_lr { 60. } else { -60. };
        if switch_xy {
            self.hitbox.move_collider(0., last_seen as f32 * multiplier);
            self.hurtbox
                .move_collider(0., last_seen as f32 * multiplier);
            self.speed.y = 0.;
        } else {
            self.hitbox.move_collider(last_seen as f32 * multiplier, 0.);
            self.hurtbox
                .move_collider(last_seen as f32 * multiplier, 0.);
            self.speed.x = 0.;
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
