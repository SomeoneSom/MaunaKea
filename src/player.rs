use bitvec::prelude as bv;

use crate::colliders::{Collider, Rect};

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
        let rect = self.hurtbox.rect().unwrap();
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
