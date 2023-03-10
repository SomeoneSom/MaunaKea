use genevo::{operator::prelude::*, prelude::*};

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

impl FitnessFunction<Inputs, i32> for FitnessCalc {
    fn fitness_of(&self, phenotype: &Inputs) -> i32 {
        todo!()
    }

    fn average(&self, a: &[i32]) -> i32 {
        a.iter().sum::<i32>() / a.len() as i32
    }

    fn highest_possible_fitness(&self) -> i32 {
        (self.checkpoint_count * 1000) as i32
    }

    fn lowest_possible_fitness(&self) -> i32 {
        i32::MIN
    }
}
