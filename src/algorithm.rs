mod geneticalg;
mod waterspeed;

// use std::time::{Duration, Instant};
use std::num::ParseFloatError;

use geneticalg::{Inputs, Simulator};
use genevo::population::ValueEncodedGenomeBuilder;

use crate::colliders::Collider;
use crate::colliders::Rect;
use crate::level::Level;
use crate::player::Player;
use crate::point::Point;

use bitvec::prelude as bv;
use colored::Colorize;
use genevo::{operator::prelude::*, prelude::*};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataParseError {
    #[error("Invalid number of rectangle corners on line {0}: expected 4, got {1}")]
    InvalidNumRectCorners(usize, usize),

    #[error("Invalid float literal")]
    InvalidFloat(#[from] ParseFloatError),
}

fn parse_checkpoint(data: &str) -> Result<Vec<Rect>, DataParseError> {
    let data_split = data.split('\n').collect::<Vec<_>>();
    let mut rects = Vec::new();
    for (line, rect) in data_split.iter().enumerate() {
        let temp_split = rect.split(", ").collect::<Vec<_>>();
        if temp_split.len() != 4 {
            return Err(DataParseError::InvalidNumRectCorners(
                line,
                temp_split.len(),
            ));
        }
        let temp_nums = temp_split
            .iter()
            .map(|x| x.parse::<f32>())
            .collect::<Result<Vec<_>, _>>()?;
        rects.push(Rect::new(
            Point::new(temp_nums[0], temp_nums[1]),
            Point::new(temp_nums[2], temp_nums[3]),
        ));
    }
    Ok(rects)
}

#[derive(Error, Debug)]
pub enum GeneticAlgError {}

fn initial_path(
    level: Level, player: Player, checkpoints: Vec<Rect>,
) -> Result<Inputs, GeneticAlgError> {
    let initial_population: Population<Inputs> = build_population()
        .with_genome_builder(ValueEncodedGenomeBuilder::new(5, 0f32, 359.99996f32))
        .of_size(500) // TODO: allow for an option to change this please
        .uniform_at_random();
    let simulator = Simulator::new(player, level, checkpoints);
    // TODO: put this in a loop
    let mut ga_sim = simulate(
        genetic_algorithm()
            .with_evaluation(&simulator)
            .with_selection(MaximizeSelector::new(0.85, 12)) //  TODO: add options for this too
            .with_crossover(SinglePointCrossBreeder::new())
            .with_mutation(RandomValueMutator::new(0.2, 0f32, 359.99996f32)) // TODO: ditto
            .with_reinsertion(ElitistReinserter::new(&simulator, true, 0.85)) // TODO: again
            .with_initial_population(initial_population)
            .build(),
    )
    .until(GenerationLimit::new(200)) // TODO: yet again
    .build();
    Ok(loop {
        let result = loop {
            let result = ga_sim.step();
            match result {
                // TODO: actually handle this stuff
                Ok(SimResult::Intermediate(step)) => (),
                Ok(SimResult::Final(step, processing_time, duration, stop_reason)) => {
                    break step.result
                }
                Err(error) => panic!("{}", error),
            }
        };
        let mut population = (*result.evaluated_population.individuals()).clone();
        if population[0].len() >= 6 {
            // TODO: make breaking criteria correct
            break result.best_solution.solution.genome;
        }
        let to_add = build_population()
            .with_genome_builder(ValueEncodedGenomeBuilder::new(1, 0f32, 359.99996f32))
            .of_size(population.len())
            .uniform_at_random();
        for (p, t) in population.iter_mut().zip(to_add.individuals().iter()) {
            p.extend(t);
        }
        ga_sim = simulate(
            // TODO: all the options need to be checked here too
            genetic_algorithm()
                .with_evaluation(&simulator)
                .with_selection(MaximizeSelector::new(0.85, 12))
                .with_crossover(SinglePointCrossBreeder::new())
                .with_mutation(RandomValueMutator::new(0.2, 0f32, 359.99996f32))
                .with_reinsertion(ElitistReinserter::new(&simulator, true, 0.85))
                .with_initial_population(Population::with_individuals(population))
                .build(),
        )
        .until(GenerationLimit::new(200)) // TOO
        .build();
    })
}

#[derive(Error, Debug)]
pub enum AlgorithmError {
    #[error(transparent)]
    DataParseError(#[from] DataParseError),

    #[error(transparent)]
    GeneticAlgError(#[from] GeneticAlgError),
}

pub fn run_alg(level: Level, player: Player, checkpoints: &str) -> Result<(), AlgorithmError> {
    let base_inputs = initial_path(level, player, parse_checkpoint(checkpoints)?)?;
    todo!();
    Ok(())
}

#[cfg(test)]
mod tests {}
