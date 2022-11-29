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
    #[inline]
    pub fn collide_check(&self, other: &Collider) -> bool {
        match other {
            Collider::Rectangular(rect) => rect.collide_check(self),
            Collider::Circular(circ) => circ.collide_check(self)
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

    pub fn reset_subpixels(&mut self, switch_xy: bool) -> () {
        match self {
            Collider::Rectangular(rect) => {
                if switch_xy {
                    rect.ul.1 = rect.ul.1.round();
                    rect.dr.1 = rect.dr.1.round();
                } else {
                    rect.ul.0 = rect.ul.0.round();
                    rect.dr.0 = rect.dr.0.round();
                }
            },
            _ => panic!("Tried to call reset_subpixels on a Circle. Should be unreachable.")
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
        match other {
            Collider::Rectangular(rect) => rect.collide_check(&Collider::Circular(self)),
            Collider::Circular(circ) => {
                let distance: f32 = (self.origin.0 - circ.origin.0).powi(2)
                    + (self.origin.1 - circ.origin.1).powi(2);
                distance < (self.radius + circ.radius).powi(2)
            }
        }
    }
}
