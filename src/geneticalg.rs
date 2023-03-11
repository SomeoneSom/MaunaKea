use genevo::{operator::prelude::*, prelude::*};
use ordered_float::OrderedFloat;

#[derive(PartialEq, PartialOrd, Eq, Ord, Clone, Copy, Debug)]
struct OrdFloat64(OrderedFloat<f64>);

impl Fitness for OrdFloat64 {
    fn zero() -> Self {
        Self(OrderedFloat(0f64))
    }

    fn abs_diff(&self, other: &Self) -> Self {
        Self(OrderedFloat((self.0  - other.0).abs()))
    }
}

type Inputs = Vec<f32>;

#[derive(Clone, Debug)]
struct InputsPheno {
    inputs: Inputs,
}

impl Phenotype<Inputs> for InputsPheno {
    fn genes(&self) -> Inputs {
        self.inputs.clone()
    }

    fn derive(&self, genes: Inputs) -> Self {
        Self { inputs: genes }
    }
}

#[derive(Clone, Debug)]
struct FitnessCalc {
    checkpoint_count: usize,
}

impl FitnessFunction<Inputs, OrdFloat64> for FitnessCalc {
    fn fitness_of(&self, phenotype: &Inputs) -> OrdFloat64 {
        todo!()
    }

    fn average(&self, a: &[OrdFloat64]) -> OrdFloat64 {
        OrdFloat64(OrderedFloat(a.iter().map(|f| f.0.into_inner()).sum::<f64>() / (a.len() as f64)))
    }

    fn highest_possible_fitness(&self) -> OrdFloat64 {
        OrdFloat64(OrderedFloat((self.checkpoint_count * 1000) as f64))
    }

    fn lowest_possible_fitness(&self) -> OrdFloat64 {
        OrdFloat64(OrderedFloat(f64::MIN))
    }
}