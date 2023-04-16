use crate::colliders::Rect;
use crate::level::Level;
use crate::player::{Player, FrameResult};

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

    fn sim_player(&self, inp: &Inputs) -> (Player, usize) {
        let mut player = self.player.clone();
        let mut checkpoint_index = 0usize;
        for &i in inp {
            player.speed_calc(i, &self.level); // TODO: restrict
            player.move_self(&self.level);
            match player.collide(&self.level, &self.checkpoints[checkpoint_index]) {
                FrameResult::Death => break,
                FrameResult::CheckpointHit => checkpoint_index += 1,
                FrameResult::Nothing => (),
            }
        }
        (player, checkpoint_index)
    }
}

impl<'a> FitnessFunction<Inputs, OrdFloat64> for &'a Simulator {
    fn fitness_of(&self, inp: &Inputs) -> OrdFloat64 {
        let (player, checkpoint_index) = self.sim_player(inp);
        let checkpoint = self.checkpoints[checkpoint_index];
        todo!()
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
