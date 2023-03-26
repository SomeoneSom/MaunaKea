use crate::player::Player;
use crate::level::Level;
use genevo::{operator::prelude::*, prelude::*};
use ordered_float::OrderedFloat;

#[derive(PartialEq, PartialOrd, Eq, Ord, Clone, Copy, Debug)]
struct OrdFloat64(OrderedFloat<f64>);

impl Fitness for OrdFloat64 {
    fn zero() -> Self {
        Self(OrderedFloat(0f64))
    }

    fn abs_diff(&self, other: &Self) -> Self {
        Self(OrderedFloat((self.0 - other.0).abs()))
    }
}

type Inputs = Vec<f32>;

#[derive(Clone, Debug)]
struct PlayerSim<'a> {
    inputs: Inputs,
    player: Player,
    level: &'a Level,
}

impl<'a> PlayerSim<'a> {
    fn simulate(&self) {
        todo!()
    }

    fn from_inputs(inp: &Inputs) -> Self {
        todo!()
    }
}

impl<'a> Phenotype<Inputs> for PlayerSim<'a> {
    fn genes(&self) -> Inputs {
        self.inputs.clone()
    }

    //maybe this function is meant to not reset stuff? idk, remove this comment when this has been cleared up
    fn derive(&self, genes: Inputs) -> Self {
        Self::from_inputs(&genes)
    }
}

#[derive(Clone, Debug)]
struct Simulator {
    checkpoint_count: usize,
}

impl FitnessFunction<Inputs, OrdFloat64> for Simulator {
    fn fitness_of(&self, inp: &Inputs) -> OrdFloat64 {
        let player = PlayerSim::from_inputs(inp);
        player.simulate();
        todo!()
    }

    fn average(&self, a: &[OrdFloat64]) -> OrdFloat64 {
        OrdFloat64(OrderedFloat(
            a.iter().map(|f| f.0.into_inner()).sum::<f64>() / (a.len() as f64),
        ))
    }

    fn highest_possible_fitness(&self) -> OrdFloat64 {
        OrdFloat64(OrderedFloat((self.checkpoint_count * 10000 + 10000) as f64))
    }

    fn lowest_possible_fitness(&self) -> OrdFloat64 {
        OrdFloat64(OrderedFloat(f64::MIN))
    }
}
