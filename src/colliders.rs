use crate::level::Level;

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
    pub fn collide_check(&self, other:&Collider) -> bool {
        if matches!(other, Collider::Rectangular(_)) {
            let rect = if let Collider::Rectangular(r) = other {r} else {unreachable!()};
            return rect.collide_check(self);
        } else {
            let circ = if let Collider::Circular(c) = other {c} else {unreachable!()};
            return circ.collide_check(self);
        }
    }
}

#[derive(Clone)]
#[derive(Copy)]
#[derive(Default)]
pub struct Rect {
    ul: (f32, f32),
    ur: (f32, f32),
    dl: (f32, f32),
    dr: (f32, f32),
}

impl Rect {
    pub fn new(up_left: (f32, f32), down_right: (f32, f32)) -> Self {
        Self {
            ul: up_left,
            ur: (down_right.0, up_left.1),
            dl: (up_left.0, down_right.1),
            dr: down_right,
        }
    }

    fn collide_check(self, other:&Collider) -> bool {
        if matches!(other, Collider::Circular(_)) {
            let circ = if let Collider::Circular(c) = other {c} else {unreachable!()};
            return Rect::line_to_circ(circ, self.ul, (self.dr.0, self.ul.1))
                || Rect::line_to_circ(circ, (self.dr.0, self.ul.1), self.dr)
                || Rect::line_to_circ(circ, self.dr, (self.ul.0, self.dr.1))
                || Rect::line_to_circ(circ, (self.ul.0, self.dr.1), self.ul);
        } else {
            let rect = if let Collider::Rectangular(r) = other {r} else {unreachable!()};
            if rect.ul.0 < self.dr.0 && self.ul.0 < rect.dr.0 && rect.ul.1 < self.dr.1 {
                return self.ul.1 < rect.dr.1;
            } else {
                return false;
            }
        }
    }

    fn line_to_circ(circ:&Circle, from:(f32, f32), to:(f32, f32)) -> bool {
        let sub:(f32, f32) = (from.0 - to.0, from.1 - to.1);
        let sub2:(f32, f32) = (from.0 - circ.origin.0, from.1 - circ.origin.1);
        let mut val:f32 = (sub.0 * sub2.0 + sub.1 * sub2.1) / (sub2.0.powi(2) + sub2.1.powi(2));
        val = val.clamp(0., 1.);
        let closest:(f32, f32) = (from.0 + sub2.0 * val, from.1 + sub2.1 * val);
        let distance:f32 = (circ.origin.0 - closest.0).powi(2) +
            (circ.origin.1 - closest.1).powi(2);
        return distance < circ.radius.powi(2);
    }
}

#[derive(Clone)]
#[derive(Copy)]
#[derive(Default)]
pub struct Circle {
    radius: f32,
    origin: (f32, f32),
}

impl Circle {
    pub fn new(rad: f32, orig: (f32, f32)) -> Self {
        Self {
            radius: rad,
            origin: orig,
        }
    }

    fn collide_check(self, other:&Collider) -> bool {
        if matches!(other, Collider::Rectangular(_)) {
            let rect = if let Collider::Rectangular(r) = other {r} else {unreachable!()};
            return rect.collide_check(&Collider::Circular(self));
        } else {
            let circ = if let Collider::Circular(c) = other {c} else {unreachable!()};
            let distance: f32 =
                (self.origin.0 - circ.origin.0).powi(2) + (self.origin.1 - circ.origin.1).powi(2);
            return distance < (self.radius + circ.radius).powi(2);
        }
    }
}

/*pub struct Collider {
    shape:Shapes
}

impl Collider {
    pub fn new_rect(rect:Rect) -> Self {
        Self {
            shape: Shapes::Rectangular(rect)
        }
    }

    pub fn new_circ(circle:Circle) -> Self {
        Self {
            shape: Shapes::Circular(circle)
        }
    }

    pub fn collide_check(self, collider:Collider) -> bool {
        if matches!(collider.shape, Shapes::Circular(_)) && matches!(self.shape, Shapes::Circular(_)) {
            return Self::coll_circ_to_circ(self, collider);
        } else if matches!(collider.shape, Shapes::Circular(_)) && matches!(self.shape, Shapes::Rectangular(_)) {
            return Self::coll_rect_to_circ(self, collider);
        } else if matches!(collider.shape, Shapes::Rectangular(_)) && matches!(self.shape, Shapes::Circular(_)) {
            return Self::coll_rect_to_circ(collider, self);
        } else {
            return Self::coll_rect_to_rect(self, collider);
        }
    }

    fn coll_circ_to_circ(circ_a:Collider, circ_b:Collider) -> bool {
        let circ1:Circle = if let Shapes::Circular(c) = circ_a.shape {c} else {unreachable!()};
        let circ2:Circle = if let Shapes::Circular(c) = circ_b.shape {c} else {unreachable!()};
        let distance:f32 = (circ1.origin.0 - circ2.origin.0).powi(2) + (circ1.origin.1 - circ2.origin.1).powi(2);
        return distance < (circ1.radius + circ2.radius).powi(2);
    }

    fn coll_rect_to_circ(rect:Collider, circ:Collider) -> bool {
        return true;
    }

    fn coll_rect_to_rect(rect_a:Collider, rect_b:Collider) -> bool {
        return true;
    }
}*/

trait Entity {
    fn on_enter(&self, player: &mut Player, axes: Axes) -> ();
    fn on_exit(&self, player: &mut Player, axes: Axes) -> ();
}

//woo boilerplate
pub struct Solid {
    colliders: Vec<Collider>,
}

impl Solid {
    fn new() -> Self {
        Self {
            colliders: Vec::new(),
        }
    }
}

impl Entity for Solid {
    fn on_enter(&self, player: &mut Player, axes: Axes) -> () {}
    //should never exit
    fn on_exit(&self, _player: &mut Player, _axes: Axes) -> () {
        panic!("Tried to exit a Solid, something wrong with the simulator code.");
    }
}

pub struct SemiSolid {
    direction: Direction,
    colliders: Vec<Collider>,
}

impl SemiSolid {
    fn new(dir: Direction) -> Self {
        Self {
            direction: dir,
            colliders: Vec::new(),
        }
    }
}

impl Entity for SemiSolid {
    fn on_enter(&self, player: &mut Player, axes: Axes) -> () {}
    fn on_exit(&self, player: &mut Player, axes: Axes) -> () {}
}

pub struct Death {
    colliders: Vec<Collider>,
}

impl Death {
    pub fn new(colliders:Vec<Collider>) -> Self {
        Self {
            colliders: colliders,
        }
    }
}

impl Entity for Death {
    //kill player
    fn on_enter(&self, player: &mut Player, _axes: Axes) -> () {
        player.die();
    }
    //should never exit
    fn on_exit(&self, _player: &mut Player, _axes: Axes) -> () {
        panic!("Tried to exit a Death, something wrong with the simulator code.");
    }
}

pub struct Spike {
    direction: Direction,
    colliders: Vec<Collider>,
}

impl Spike {
    fn new(dir: Direction) -> Self {
        Self {
            direction: dir,
            colliders: Vec::new(),
        }
    }
}

impl Entity for Spike {
    //kill player IF not moving with them
    fn on_enter(&self, player: &mut Player, _axes: Axes) -> () {
        if !(self.direction == Direction::Left && player.speed.0 < 0.
            || self.direction == Direction::Up && player.speed.1 < 0.
            || self.direction == Direction::Right && player.speed.0 > 0.
            || self.direction == Direction::Down && player.speed.1 > 0.)
        {
            player.die();
        }
    }
    fn on_exit(&self, _player: &mut Player, _axes: Axes) -> () {}
}

pub struct Trigger {
    colliders: Vec<Collider>,
}

impl Trigger {
    fn new() -> Self {
        Self {
            colliders: Vec::new(),
        }
    }
}

impl Entity for Trigger {
    fn on_enter(&self, player: &mut Player, axes: Axes) -> () {}
    fn on_exit(&self, player: &mut Player, axes: Axes) -> () {}
}

#[derive(Default)]
pub struct Player {
    pub speed: (f32, f32),
    pub alive: bool,
    pub hurtbox: Collider,
    pub hitbox: Collider
}

impl Player {
    pub fn new(speed:(f32, f32), position:(f32, f32)) -> Self {
        Self {
            speed: speed,
            alive: true,
            hurtbox: Collider::Rectangular(Rect::new((position.0 - 4., position.1 - 11.),
                (position.0 + 4., position.1 - 2.))),
            hitbox: Collider::Rectangular(Rect::new(position, (position.0 + 8., position.1 + 11.)))
        }
    }

    pub fn sim_frame(&mut self, angle:i32, death:&Vec<Death>, checkpoint:&Rect) -> f32 {
        //TODO: precompute these angles
        let mut ang:i32 = angle - 90000;
        if ang < 0 {
            ang += 360000;
        }
        ang = 360000 - ang;
        let rang:f32 = (ang as f32 / 1000.).to_radians();
        let rad:f32 = 4800. / ((80. * rang.cos()).powi(2) + (60. * rang.sin()).powi(2)).sqrt(); 
        let target:(f32, f32) = (rad * rang.cos(), rad * rang.sin() * -1.);
        //println!("{} -> {}, {}", ang, target.0, target.1);
        let mut temp_speed:(f32, f32) = self.speed;
        //TODO: add in stuff for when speed is outside octagon and should be pulled back to it
        //TODO: maybe make this use move_self?
        //TODO: make speed capping actually work how its meant to
        //TODO: water surface bs
        //sim speed change for x
        if f32::abs(target.0 - temp_speed.0) < 10. {
            temp_speed.0 = target.0;
        } else {
            temp_speed.0 += f32::clamp(target.0 - temp_speed.0, -10., 10.);
        }
        temp_speed.0 = temp_speed.0.clamp(-60., 60.);
        //and again for y
        if f32::abs(target.1 - temp_speed.1) < 10. {
            temp_speed.1 = target.1;
        } else {
            temp_speed.1 += f32::clamp(target.1 - temp_speed.1, -10., 10.);
        }
        temp_speed.1 = temp_speed.1.clamp(-80., 80.);
        //and apply
        let rect = if let Collider::Rectangular(r) = self.hurtbox {r} else {unreachable!()};
        let rect3 = Rect::new((rect.ul.0 + temp_speed.0 / 60., rect.ul.1 + temp_speed.1 / 60.),
        (rect.dr.0 + temp_speed.0 / 60., rect.dr.1 + temp_speed.1 / 60.));
        let temp_hurtbox:Collider = Collider::Rectangular(rect3);
        //TODO: make it so that you dont have to do all these collision checks
        for d in death {
            for c in &d.colliders {
                if temp_hurtbox.collide_check(c) {
                    /*if matches!(c, Collider::Rectangular(_)) {
                        let rect4 = if let Collider::Rectangular(rec) = c {rec} else {unreachable!()};
                        println!("died to Rect, UL: {},{} , DR: {},{} , PUL: {},{} , PDR: {},{}",
                        rect4.ul.0, rect4.ul.1, rect4.dr.0, rect4.dr.1,
                        rect3.ul.0, rect3.ul.1, rect3.dr.0, rect3.dr.1);
                    } else {
                        let circ = if let Collider::Circular(cir) = c {cir} else {unreachable!()};
                        println!("died to Circle, ORIG: {},{} , RAD: {}, PUL: {},{} , PDR: {},{}",
                        circ.origin.0, circ.origin.1, circ.radius,
                        rect3.ul.0, rect3.ul.1, rect3.dr.0, rect3.dr.1);
                    }*/
                    return 9999999.
                }
            }
        }
        //TODO: make it so this actually calcs distance properly and not just center of player to center of checkpoint
        let rect2 = if let Collider::Rectangular(r) = temp_hurtbox {r} else {unreachable!()};
        let player_cent:(f32, f32) = ((rect2.ul.0 + rect2.dr.0) / 2., (rect.ul.1 + rect2.dr.1) / 2.);
        let check_cent:(f32, f32) = ((checkpoint.ul.0 + checkpoint.dr.0) / 2., (checkpoint.ul.1 + checkpoint.dr.1) / 2.);
        return f32::sqrt((player_cent.0 - check_cent.0).powi(2) + (player_cent.1 - check_cent.1).powi(2));
    }

    pub fn move_self(&mut self, angle:i32) -> () {
        let mut ang:i32 = angle - 90000;
        if ang < 0 {
            ang += 360000;
        }
        ang = 360000 - ang;
        let rang:f32 = (ang as f32 / 1000.).to_radians();
        let rad:f32 = 4800. / ((80. * rang.cos()).powi(2) + (60. * rang.sin()).powi(2)).sqrt(); 
        let target:(f32, f32) = (rad * rang.cos(), rad * rang.sin() * -1.);
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
        //and apply
        let rect = if let Collider::Rectangular(r) = self.hurtbox {r} else {unreachable!()};
        self.hurtbox = Collider::Rectangular(Rect::new((rect.ul.0 + self.speed.0 / 60., rect.ul.1 + self.speed.1 / 60.),
            (rect.dr.0 + self.speed.0 / 60., rect.dr.1 + self.speed.1 / 60.)));
        self.hitbox = Collider::Rectangular(Rect::new((rect.ul.0 + self.speed.0 / 60., rect.ul.1 + self.speed.1 / 60.),
            (rect.dr.0 + self.speed.0 / 60., rect.dr.1 + self.speed.1 / 60. + 2.)));
    }

    //TODO: maybe get rid of this, it might not be useful
    fn die(&mut self) -> () {
        self.alive = false;
    }
}
