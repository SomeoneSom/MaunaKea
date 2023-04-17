use std::time::SystemTime;

use crate::colliders::Rect;
use crate::level::Level;
use crate::player::{FrameResult, Player};

use genevo::prelude::*;
use ordered_float::OrderedFloat;

#[derive(PartialEq, PartialOrd, Eq, Ord, Clone, Copy, Debug)]
pub struct OrdFloat64(OrderedFloat<f64>);

impl Fitness for OrdFloat64 {
    fn zero() -> Self {
        Self(OrderedFloat(0f64))
    }

    fn abs_diff(&self, other: &Self) -> Self {
        Self(OrderedFloat((self.0 - other.0).abs()))
    }
}

pub type Inputs = Vec<f64>;

/*#[derive(Clone, Debug)]
struct PlayerSim<'a> {
    inputs: Inputs,
    player: Player,
    level: &'a Level,
}

impl<'a> PlayerSim<'a> {
    fn simulate(&self) {
        todo!()
    }
}

impl<'a> Phenotype<Inputs> for PlayerSim<'a> {
    fn genes(&self) -> Inputs {
        self.inputs.clone()
    }

    // maybe this function is meant to not reset stuff? idk, remove this comment when this has been cleared up
    fn derive(&self, genes: Inputs) -> Self {
        Self {
            inputs: genes,
            player: self.player.clone(),
            level: self.level,
        }
    }
}*/

#[derive(Clone, Debug)]
pub(super) struct Simulator {
    player: Player,
    level: Level, // lets just say this owns the level for now
    checkpoints: Vec<Rect>,
}

impl Simulator {
    pub fn new(player: Player, level: Level, checkpoints: Vec<Rect>) -> Self {
        Self {
            player,
            level,
            checkpoints,
        }
    }

    // TODO: break when hit final checkpoint
    fn sim_player(&self, inp: &Inputs) -> (Player, Player, usize, usize) {
        //let now = SystemTime::now();
        let mut player = self.player.clone();
        let mut prev_player = player.clone();
        let mut checkpoint_index = 0usize;
        let mut frame_count = 0usize;
        for &i in inp {
            frame_count += 1;
            prev_player = player.clone();
            player.speed_calc(i, &self.level); // TODO: restrict
            player.move_self(&self.level);
            match player.collide(&self.level, &self.checkpoints[checkpoint_index]) {
                FrameResult::Death => break,
                FrameResult::CheckpointHit => checkpoint_index += 1,
                FrameResult::Nothing => (),
            }
        }
        //println!("{}", now.elapsed().unwrap().as_secs_f64());
        (player, prev_player, checkpoint_index, frame_count)
    }
}

impl<'a> FitnessFunction<Inputs, OrdFloat64> for &'a Simulator {
    fn fitness_of(&self, inp: &Inputs) -> OrdFloat64 {
        let (player, prev_player, checkpoint_index, frame_count) = self.sim_player(inp);
        let checkpoint = self.checkpoints[checkpoint_index];
        if checkpoint_index == self.checkpoints.len() - 1 {
            let (mut accurate_distance, touched) =
                checkpoint.accurate_distance(player.pos(), prev_player.pos());
            if !touched {
                accurate_distance = 3.16666f64;
            }
            OrdFloat64(OrderedFloat(
                checkpoint_index as f64 * 10000f64 - frame_count as f64 * 8f64 - accurate_distance,
            ))
        } else {
            // NOTE: this doesnt have closestDist or atFrame, i might need to add those later
            let checkpoint_center = checkpoint.center();
            let player_center = match player.hitbox.rect() {
                Some(rect) => rect,
                None => unreachable!(),
            }
            .center();
            OrdFloat64(OrderedFloat(
                checkpoint_index as f64 * 10000f64
                    - checkpoint_center.distance(player_center) as f64
                    - frame_count as f64,
            ))
        }
    }

    fn average(&self, a: &[OrdFloat64]) -> OrdFloat64 {
        OrdFloat64(OrderedFloat(
            a.iter().map(|f| f.0.into_inner()).sum::<f64>() / (a.len() as f64),
        ))
    }

    fn highest_possible_fitness(&self) -> OrdFloat64 {
        OrdFloat64(OrderedFloat(
            (self.checkpoints.len() * 10000 + 10000) as f64,
        ))
    }

    fn lowest_possible_fitness(&self) -> OrdFloat64 {
        OrdFloat64(OrderedFloat(f64::MIN))
    }
}

#[cfg(test)]
mod tests {}
