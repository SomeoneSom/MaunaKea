use crate::level::Level;

use bitvec::prelude as bv;

#[derive(PartialEq)]
pub enum Direction {
    Left,
    Up,
    Right,
    Down,
}

pub enum Axes {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy)]
pub enum Collider {
    Rectangular(Rect),
    Circular(Circle),
}

impl Default for Collider {
    fn default() -> Collider {
        Collider::Rectangular(Rect::default())
    }
}

impl Collider {
    pub fn collide_check(&self, other: &Collider) -> bool {
        if matches!(other, Collider::Rectangular(_)) {
            let rect = if let Collider::Rectangular(r) = other {
                r
            } else {
                unreachable!()
            };
            return rect.collide_check(self);
        } else {
            let circ = if let Collider::Circular(c) = other {
                c
            } else {
                unreachable!()
            };
            return circ.collide_check(self);
        }
    }

    pub fn move_collider(self, x: f32, y: f32) -> Collider {
        if matches!(self, Collider::Rectangular(_)) {
            let mut rect: Rect = *self.rect().unwrap();
            rect.ul.0 += x / 60.0;
            rect.ul.1 += y / 60.0;
            rect.dr.0 += x / 60.0;
            rect.dr.1 += y / 60.0;
            return Collider::Rectangular(rect);
        } else {
            let mut circ: Circle = *self.circle().unwrap();
            circ.origin.0 += x / 60.;
            circ.origin.1 += y / 60.;
            return Collider::Circular(circ);
        }
    }

    //TODO: change these to let-else once it becomes stable
    pub fn rect(&self) -> Option<&Rect> {
        match self {
            Collider::Rectangular(value) => Some(value),
            _ => None,
        }
    }

    pub fn circle(&self) -> Option<&Circle> {
        match self {
            Collider::Circular(value) => Some(value),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct Rect {
    pub ul: (f32, f32),
    pub dr: (f32, f32),
}

impl Rect {
    pub fn new(up_left: (f32, f32), down_right: (f32, f32)) -> Self {
        Self {
            ul: up_left,
            dr: down_right,
        }
    }

    pub fn new_xywh(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            ul: (x, y),
            dr: (x + w, y + h),
        }
    }

    fn collide_check(self, other: &Collider) -> bool {
        if matches!(other, Collider::Circular(_)) {
            let circ = other.circle().unwrap();
            return Rect::line_to_circ(circ, self.ul, (self.dr.0, self.ul.1))
                || Rect::line_to_circ(circ, (self.dr.0, self.ul.1), self.dr)
                || Rect::line_to_circ(circ, self.dr, (self.ul.0, self.dr.1))
                || Rect::line_to_circ(circ, (self.ul.0, self.dr.1), self.ul);
        } else {
            let rect = other.rect().unwrap();
            if rect.ul.0 < self.dr.0 && self.ul.0 < rect.dr.0 && rect.ul.1 < self.dr.1 {
                return self.ul.1 < rect.dr.1;
            } else {
                return false;
            }
        }
    }

    fn line_to_circ(circ: &Circle, from: (f32, f32), to: (f32, f32)) -> bool {
        let sub: (f32, f32) = (from.0 - to.0, from.1 - to.1);
        let sub2: (f32, f32) = (from.0 - circ.origin.0, from.1 - circ.origin.1);
        let mut val: f32 = (sub.0 * sub2.0 + sub.1 * sub2.1) / (sub2.0.powi(2) + sub2.1.powi(2));
        val = val.clamp(0., 1.);
        let closest: (f32, f32) = (from.0 + sub2.0 * val, from.1 + sub2.1 * val);
        let distance: f32 =
            (circ.origin.0 - closest.0).powi(2) + (circ.origin.1 - closest.1).powi(2);
        return distance < circ.radius.powi(2);
    }
}

#[derive(Clone, Copy, Default)]
pub struct Circle {
    pub radius: f32,
    pub origin: (f32, f32),
}

impl Circle {
    pub fn new(rad: f32, orig: (f32, f32)) -> Self {
        Self {
            radius: rad,
            origin: orig,
        }
    }

    fn collide_check(self, other: &Collider) -> bool {
        if matches!(other, Collider::Rectangular(_)) {
            let rect = other.rect().unwrap();
            return rect.collide_check(&Collider::Circular(self));
        } else {
            let circ = other.circle().unwrap();
            let distance: f32 =
                (self.origin.0 - circ.origin.0).powi(2) + (self.origin.1 - circ.origin.1).powi(2);
            return distance < (self.radius + circ.radius).powi(2);
        }
    }
}

#[derive(Default)]
pub struct Player {
    pub speed: (f32, f32),
    pub alive: bool,
    pub hurtbox: Collider,
    pub hitbox: Collider,
}

impl Player {
    pub fn new(speed: (f32, f32), position: (f32, f32)) -> Self {
        Self {
            speed: speed,
            alive: true,
            hurtbox: Collider::Rectangular(Rect::new(
                (position.0 - 4., position.1 - 12.),
                (position.0 + 3., position.1 - 3.),
            )),
            hitbox: Collider::Rectangular(Rect::new(
                (position.0 - 4., position.1 - 12.),
                (position.0 + 3., position.1 - 1.),
            )),
        }
    }

    pub fn sim_frame(
        &mut self, angle: i32, bounds: &Rect, static_death: &Vec<bv::BitVec>, checkpoint: &Rect,
    ) -> f32 {
        let temp_speed: (f32, f32) = self.speed.clone();
        let temp_hurtbox = self.hurtbox.clone();
        let temp_hitbox = self.hitbox.clone();
        self.move_self(angle);
        let hurtbox_rect: &Rect = self.hurtbox.rect().unwrap();
        let left: i32 = (hurtbox_rect.ul.0 - bounds.ul.0).round() as i32;
        let right: i32 = (hurtbox_rect.dr.0 - bounds.dr.0).round() as i32;
        let up: i32 = (hurtbox_rect.ul.1 - bounds.ul.1).round() as i32;
        let down: i32 = (hurtbox_rect.dr.1 - bounds.dr.1).round() as i32;
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
        //TODO: make it so this actually calcs distance properly and not just center of player to center of checkpoint
        let rect = if let Collider::Rectangular(r) = self.hurtbox {
            r
        } else {
            unreachable!()
        };
        let player_cent: (f32, f32) = ((rect.ul.0 + rect.dr.0) / 2., (rect.ul.1 + rect.dr.1) / 2.);
        let check_cent: (f32, f32) = (
            (checkpoint.ul.0 + checkpoint.dr.0) / 2.,
            (checkpoint.ul.1 + checkpoint.dr.1) / 2.,
        );
        self.speed = temp_speed;
        self.hurtbox = temp_hurtbox;
        self.hitbox = temp_hitbox;
        return f32::sqrt(
            (player_cent.0 - check_cent.0).powi(2) + (player_cent.1 - check_cent.1).powi(2),
        );
    }

    pub fn move_self(&mut self, angle: i32) -> () {
        //TODO: add in stuff for when speed is outside octagon and should be pulled back to it
        //TODO: make speed capping actually work how its meant to
        //TODO: water surface bs
        //TODO: precompute angles
        let mut ang: i32 = angle - 90000;
        if ang < 0 {
            ang += 360000;
        }
        ang = 360000 - ang;
        let rang: f32 = (ang as f32 / 1000.).to_radians();
        let rad: f32 = 4800. / ((80. * rang.cos()).powi(2) + (60. * rang.sin()).powi(2)).sqrt();
        let target: (f32, f32) = (rad * rang.cos(), rad * rang.sin() * -1.);
        if f32::abs(target.0 - self.speed.0) < 10. {
            self.speed.0 = target.0;
        } else {
            self.speed.0 += f32::clamp(target.0 - self.speed.0, -10., 10.);
        }
        self.speed.0 = self.speed.0.clamp(-60., 60.);
        //and again for y
        if f32::abs(target.1 - self.speed.1) < 10. {
            self.speed.1 = target.1;
        } else {
            self.speed.1 += f32::clamp(target.1 - self.speed.1, -10., 10.);
        }
        self.speed.1 = self.speed.1.clamp(-80., 80.);
        self.hurtbox = self.hurtbox.move_collider(self.speed.0, self.speed.1);
        self.hitbox = self.hitbox.move_collider(self.speed.0, self.speed.1);
    }

    //TODO: maybe get rid of this, it might not be useful
    fn die(&mut self) -> () {
        self.alive = false;
    }
}
