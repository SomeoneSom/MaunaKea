use quadtree_rs::{
    area::{Area, AreaBuilder},
    point::Point as QTPoint,
};

use crate::point::Point;

const DELTATIME: f32 = 0.0166667;

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

#[derive(Clone, Copy, Debug)]
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
            Collider::Circular(circ) => circ.collide_check(self),
        }
    }

    pub fn move_collider(&mut self, x: f32, y: f32) {
        match self {
            Collider::Rectangular(rect) => {
                rect.ul.x += x * DELTATIME;
                rect.ul.y += y * DELTATIME;
                rect.dr.x += x * DELTATIME;
                rect.dr.y += y * DELTATIME;
            }
            Collider::Circular(circ) => {
                circ.origin.x += x * DELTATIME;
                circ.origin.y += y * DELTATIME;
            }
        };
    }

    pub fn reset_subpixels(&mut self, axis: Axes) {
        match self {
            Collider::Rectangular(rect) => match axis {
                Axes::Horizontal => {
                    rect.ul.x = rect.ul.x.round();
                    rect.dr.x = rect.dr.x.round();
                }
                Axes::Vertical => {
                    rect.ul.y = rect.ul.y.round();
                    rect.dr.y = rect.dr.y.round();
                }
            },
            _ => panic!("Tried to call reset_subpixels on a Circle. Should be unreachable."),
        };
    }

    // TODO: change these to let-else once it becomes stable
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

    // TODO: handle errors
    pub fn to_qt_area(self) -> Area<i32> {
        match self {
            Collider::Rectangular(rect) => {
                match AreaBuilder::default()
                    .anchor(QTPoint {
                        x: rect.ul.x.round() as i32,
                        y: rect.ul.y.round() as i32,
                    })
                    .dimensions((
                        rect.dr.x.round() as i32 - rect.ul.x.round() as i32 + 1,
                        rect.dr.y.round() as i32 - rect.ul.y.round() as i32 + 1,
                    ))
                    .build()
                {
                    Ok(area) => area,
                    Err(err) => panic!("{err}"),
                }
            }
            Collider::Circular(circ) => { // TODO: fix this match arm
                match AreaBuilder::default()
                    .anchor(QTPoint {
                        x: (circ.origin.x - circ.radius) as i32,
                        y: (circ.origin.y - circ.radius) as i32,
                    })
                    .dimensions(((circ.radius * 2f32) as i32, (circ.radius * 2f32) as i32))
                    .build()
                {
                    Ok(area) => area,
                    Err(err) => panic!("{err}"),
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Rect {
    pub ul: Point,
    pub ur: Point,
    pub dl: Point,
    pub dr: Point,
}

impl Rect {
    // TODO: check if up_left and down_right are actually up_left and down_right
    pub fn new(up_left: Point, down_right: Point) -> Self {
        Self {
            ul: up_left,
            ur: Point::new(down_right.x, up_left.y),
            dl: Point::new(up_left.x, down_right.y),
            dr: down_right,
        }
    }

    pub fn new_xywh(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            ul: Point::new(x, y),
            ur: Point::new(x + w - 1f32, y),
            dl: Point::new(x, y + h - 1f32),
            dr: Point::new(x + w - 1f32, y + h - 1f32),
        }
    }

    #[inline]
    pub fn center(&self) -> Point {
        Point::new((self.ul.x + self.dr.x) / 2.0, (self.ul.y + self.dr.y) / 2.0)
    }

    pub fn accurate_distance(&self, pos: Point, prev_pos: Point) -> (f64, bool) {
        let mut raw_l = self.ul.x - pos.x;
        let mut raw_r = pos.x - self.dr.x;
        let mut raw_u = self.ul.y - pos.y;
        let mut raw_d = pos.y - self.dr.y;
        let touched = raw_l < 0f32 || raw_r < 0f32 || raw_u < 0f32 || raw_d < 0f32;
        if touched {
            raw_l = self.ul.x - prev_pos.x;
            raw_r = prev_pos.x - self.dr.x;
            raw_u = self.ul.y - prev_pos.y;
            raw_d = prev_pos.y - self.dr.y;
        }
        let x_diff = if raw_l > 0f32 {
            raw_l as f64
        } else if raw_r > 0f32 {
            raw_r as f64
        } else {
            0f64
        };
        let y_diff = if raw_u > 0f32 {
            raw_u as f64
        } else if raw_d > 0f32 {
            raw_d as f64
        } else {
            0f64
        };
        (f64::sqrt(x_diff.powi(2) + y_diff.powi(2)), touched)
    }

    fn collide_check(&self, other: &Collider) -> bool {
        match other {
            Collider::Rectangular(rect) => {
                rect.ul.x < self.dr.x
                    && self.ul.x < rect.dr.x
                    && rect.ul.y < self.dr.y
                    && self.ul.y < rect.dr.y
            }
            Collider::Circular(circ) => {
                Rect::line_to_circ(circ, self.ul, Point::new(self.dr.x, self.ul.y))
                    || Rect::line_to_circ(circ, Point::new(self.dr.x, self.ul.y), self.dr)
                    || Rect::line_to_circ(circ, self.dr, Point::new(self.ul.x, self.dr.y))
                    || Rect::line_to_circ(circ, Point::new(self.ul.x, self.dr.y), self.ul)
            }
        }
    }

    fn line_to_circ(circ: &Circle, from: Point, to: Point) -> bool {
        let sub = Point::new(from.x - to.x, from.y - to.y);
        let sub2 = Point::new(from.x - circ.origin.x, from.y - circ.origin.y);
        let mut val = sub.x * sub2.x + sub.y * sub2.y / (sub2.x.powi(2) + sub2.y.powi(2));
        val = val.clamp(0f32, 1f32);
        let closest = Point::new(from.x + sub2.x * val, from.y + sub2.y * val);
        let distance = (circ.origin.x - closest.x).powi(2) + (circ.origin.y - closest.y).powi(2);
        distance < circ.radius.powi(2)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Circle {
    pub radius: f32,
    pub origin: Point,
}

impl Circle {
    pub fn new(rad: f32, orig: Point) -> Self {
        Self {
            radius: rad,
            origin: orig,
        }
    }

    fn collide_check(self, other: &Collider) -> bool {
        match other {
            Collider::Rectangular(rect) => rect.collide_check(&Collider::Circular(self)),
            Collider::Circular(circ) => {
                let distance = (self.origin.x - circ.origin.x).powi(2)
                    + (self.origin.y - circ.origin.y).powi(2);
                distance < (self.radius + circ.radius).powi(2)
            }
        }
    }
}

#[cfg(test)]
mod tests {}
