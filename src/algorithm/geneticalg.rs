use std::sync::{Arc, Mutex};

use crate::colliders::Rect;
use crate::level::Level;
use crate::player::{FrameResult, Player};

use genevo::genetic::{Children, Parents};
use genevo::operator::prelude::RandomGenomeMutation;
use genevo::prelude::*;
use genevo::recombination::discrete::MultiPointCrossover;
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

#[derive(Clone, Debug)]
pub(super) struct InputsPop(pub Inputs, pub Arc<Mutex<Option<OrdFloat64>>>);

impl PartialEq for InputsPop {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Genotype for InputsPop {
    type Dna = f64;
}

impl MultiPointCrossover for InputsPop {
    type Dna = f64;

    fn crossover<R>(parents: Parents<Self>, num_cut_points: usize, rng: &mut R) -> Children<Self>
    where
        R: Rng + Sized,
    {
        // TODO: cloning here is slow, just stop
        Inputs::crossover(
            parents.iter().map(|p| p.0.clone()).collect(),
            num_cut_points,
            rng,
        )
        .iter()
        .map(|c| InputsPop(c.clone(), Arc::new(Mutex::new(None))))
        .collect()
    }
}

impl RandomGenomeMutation for InputsPop {
    type Dna = f64;

    fn mutate_genome<R>(
        genome: Self, mutation_rate: f64, min_value: &<Self as Genotype>::Dna,
        max_value: &<Self as Genotype>::Dna, rng: &mut R,
    ) -> Self
    where
        R: Rng + Sized,
    {
        InputsPop(
            Inputs::mutate_genome(genome.0, mutation_rate, min_value, max_value, rng),
            Arc::new(Mutex::new(None)),
        )
    }
}

pub(super) struct InputsBuilder;

impl GenomeBuilder<InputsPop> for InputsBuilder {
    fn build_genome<R>(&self, size: usize, rng: &mut R) -> InputsPop
    where
        R: Rng + Sized,
    {
        InputsPop(
            (0..size).map(|_| rng.gen_range(0f64..360f64)).collect(),
            Arc::new(Mutex::new(None)),
        )
    }
}

#[derive(Clone, Debug)]
pub(super) struct Simulator<'a> {
    player: Player,
    level: &'a Level,
    checkpoints: Vec<Rect>,
    base_checkpoint: usize,
    base_frame: usize,
}

impl<'a> Simulator<'a> {
    pub fn new(player: Player, level: &'a Level, checkpoints: Vec<Rect>) -> Self {
        Self {
            player,
            level,
            checkpoints,
            base_checkpoint: 0,
            base_frame: 0,
        }
    }

    fn sim_player(&self, inp: &Inputs) -> (Player, Player, usize, usize) {
        //let now = SystemTime::now();
        let mut player = self.player.clone();
        let mut prev_player = player.clone();
        let mut checkpoint_index = self.base_checkpoint;
        let mut frame_count = self.base_frame;
        for &i in inp {
            frame_count += 1;
            prev_player = player.clone();
            player.speed_calc(i, self.level); // TODO: restrict
            player.move_self(self.level);
            if checkpoint_index == self.checkpoints.len() {
                break;
            }
            match player.collide(self.level, &self.checkpoints[checkpoint_index]) {
                FrameResult::Death => break,
                FrameResult::CheckpointHit => checkpoint_index += 1,
                FrameResult::Nothing => (),
            }
        }
        //println!("{}", now.elapsed().unwrap().as_secs_f64());
        (player, prev_player, checkpoint_index, frame_count)
    }

    // TODO: this function name is bad
    pub fn move_own_player(&mut self, inp: &Inputs) {
        (self.player, _, self.base_checkpoint, self.base_frame) = self.sim_player(inp);
    }

    pub fn check_if_hit_final(&self, inp: &Inputs) -> bool {
        let result = self.sim_player(inp);
        result.2 == self.checkpoints.len()
    }
}

impl FitnessFunction<InputsPop, OrdFloat64> for Simulator<'_> {
    fn fitness_of(&self, inp: &InputsPop) -> OrdFloat64 {
        let mut fitness = inp.1.lock().unwrap();
        if fitness.is_none() {
            let (player, prev_player, checkpoint_index, frame_count) = self.sim_player(&inp.0);
            if checkpoint_index == self.checkpoints.len() {
                let checkpoint = self.checkpoints[checkpoint_index - 1];
                let (mut accurate_distance, touched) =
                    checkpoint.accurate_distance(player.pos(), prev_player.pos());
                if !touched {
                    accurate_distance = 3.16666f64;
                }
                *fitness = Some(OrdFloat64(OrderedFloat(
                    checkpoint_index as f64 * 10000f64
                        - frame_count as f64 * 8f64
                        - accurate_distance,
                )));
            } else {
                // NOTE: this doesnt have closestDist or atFrame, i might need to add those later
                let checkpoint = self.checkpoints[checkpoint_index];
                let checkpoint_center = checkpoint.center();
                let player_center = match player.hitbox.rect() {
                    Some(rect) => rect,
                    None => unreachable!(),
                }
                .center();
                *fitness = Some(OrdFloat64(OrderedFloat(
                    checkpoint_index as f64 * 10000f64
                        - checkpoint_center.distance(player_center) as f64
                        - frame_count as f64,
                )));
            }
        }
        fitness.unwrap()
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
