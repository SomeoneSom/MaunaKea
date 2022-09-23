use crate::level::Level;

#[derive(PartialEq)]
pub enum Direction {
    Left,
    Up,
    Right,
    Down
}

pub enum Axes {
    Horizontal,
    Vertical
}

pub enum Shapes {
    Rectangular(Rect),
    Circular(Circle)
}

pub struct Rect {
    ul:f32,
    ur:f32,
    dl:f32,
    dr:f32
}

pub struct Circle {
    radius:f32,
    origin:f32
}

pub struct Collider {
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
}

trait Entity {
    fn on_enter(&self, player:&mut Player, axes:Axes) -> ();
    fn on_exit(&self, player:&mut Player, axes:Axes) -> ();
}

//woo boilerplate
pub struct Solid<'a> {
    colliders:Vec<Collider>,
    parent:&'a mut Level
}

impl<'a> Solid<'a> {
    fn new(level:&'a mut Level) -> Self {
        Self {
            colliders: Vec::new(),
            parent: level
        }
    }
}

impl<'a> Entity for Solid<'a> {
    fn on_enter(&self, player:&mut Player, axes:Axes) -> () {

    }
    //should never exit
    fn on_exit(&self, _player:&mut Player, _axes:Axes) -> () {
        panic!("Tried to exit a Solid, something wrong with the simulator code.");
    }
}

pub struct SemiSolid<'a> {
    direction:Direction,
    colliders:Vec<Collider>,
    parent:&'a mut Level
}

impl<'a> SemiSolid<'a> {
    fn new(dir:Direction, level:&'a mut Level) -> Self {
        Self {
            direction: dir,
            colliders: Vec::new(),
            parent: level
        }
    }
}

impl<'a> Entity for SemiSolid<'a> {
    fn on_enter(&self, player:&mut Player, axes:Axes) -> () {

    }
    fn on_exit(&self, player:&mut Player, axes:Axes) -> () {
        
    }
}

pub struct Death<'a> {
    colliders:Vec<Collider>,
    parent:&'a mut Level
}

impl<'a> Death<'a> {
    fn new(level:&'a mut Level) -> Self {
        Self {
            colliders: Vec::new(),
            parent: level
        }
    }
}

impl<'a> Entity for Death<'a> {
    //kill player
    fn on_enter(&self, player:&mut Player, _axes:Axes) -> () {
        player.die();
    }
    //should never exit
    fn on_exit(&self, _player:&mut Player, _axes:Axes) -> () {
        panic!("Tried to exit a Death, something wrong with the simulator code.");
    }
}

pub struct Spike<'a> {
    direction:Direction,
    colliders:Vec<Collider>,
    parent:&'a mut Level
}

impl<'a> Spike<'a> {
    fn new(dir:Direction, level:&'a mut Level) -> Self {
        Self {
            direction: dir,
            colliders: Vec::new(),
            parent: level
        }
    }
}

impl<'a> Entity for Spike<'a> {
    //kill player IF not moving with them
    fn on_enter(&self, player:&mut Player, _axes:Axes) -> () {
        if !(self.direction == Direction::Left && player.speed.0 < 0. ||
            self.direction == Direction::Up && player.speed.1 < 0. ||
            self.direction == Direction::Right && player.speed.0 > 0. ||
            self.direction == Direction::Down && player.speed.1 > 0.) {
            player.die();
        }
    }
    fn on_exit(&self, _player:&mut Player, _axes:Axes) -> () {}
}

pub struct Trigger<'a> {
    colliders:Vec<Collider>,
    parent:&'a mut Level
}

impl<'a> Trigger<'a> {
    fn new(level:&'a mut Level) -> Self {
        Self {
            colliders: Vec::new(),
            parent: level
        }
    }
}

impl<'a> Entity for Trigger<'a> {
    fn on_enter(&self, player:&mut Player, axes:Axes) -> () {

    }
    fn on_exit(&self, player:&mut Player, axes:Axes) -> () {

    }
}

pub struct Player {
    pub speed:(f32, f32),
    pub alive:bool
}

impl Player {
    fn die(&mut self) -> () {
        self.alive = false;
    }
}