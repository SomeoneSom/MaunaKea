mod geneticalg;
mod waterspeed;

// use std::time::{Duration, Instant};
use std::num::ParseFloatError;

use geneticalg::{Inputs, InputsPop, InputsBuilder, Simulator};

use crate::colliders::Collider;
use crate::colliders::Rect;
use crate::level::Level;
use crate::player::Player;
use crate::point::Point;

use arboard::Clipboard;
use bitvec::prelude as bv;
use colored::Colorize;
use genevo::{operator::prelude::*, population::ValueEncodedGenomeBuilder, prelude::*};
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

// NOTE: not handling the error here because of absurd error type
fn initial_path(level: &Level, player: Player, checkpoints: Vec<Rect>) -> Inputs {
    let initial_population = build_population()
        .with_genome_builder(InputsBuilder)
        .of_size(50) // TODO: allow for an option to change this please
        .uniform_at_random();
    let mut simulator = Simulator::new(player, level, checkpoints);
    // TODO: put this in a loop
    let mut ga_sim = simulate(
        genetic_algorithm()
            .with_evaluation(simulator.clone())
            .with_selection(MaximizeSelector::new(0.85, 12)) //  TODO: add options for this too
            .with_crossover(SinglePointCrossBreeder::new())
            .with_mutation(RandomValueMutator::new(0.02, 0f64, 359.99999999999994)) // TODO: ditto
            .with_reinsertion(ElitistReinserter::new(simulator.clone(), false, 0.85)) // TODO: again
            .with_initial_population(initial_population)
            .build(),
    )
    .until(GenerationLimit::new(20)) // TODO: yet again
    .build();
    let mut frame_count: u32 = 5;
    let mut inputs = vec![];
    loop {
        let result = loop {
            let result = ga_sim.step();
            match result {
                // TODO: actually handle this stuff
                Ok(SimResult::Intermediate(step)) => {
                    println!("{}, {}", frame_count, step.iteration)
                }
                Ok(SimResult::Final(step, processing_time, duration, stop_reason)) => {
                    break step.result
                }
                Err(error) => panic!("{}", error),
            }
        };
        frame_count += 1;
        let mut population = (*result.evaluated_population.individuals()).clone();
        let hit_final = simulator.check_if_hit_final(&result.best_solution.solution.genome);
        println!("{:?}", result.best_solution.solution.genome);
        if hit_final {
            // TODO: make breaking criteria more correct
            inputs.extend_from_slice(&result.best_solution.solution.genome);
            break inputs;
        }
        // TODO: wow! this seems like it sucks!
        if result.best_solution.solution.genome.len() > 50 {
            simulator.move_own_player(&result.best_solution.solution.genome[0..25].to_vec());
            inputs.extend_from_slice(&result.best_solution.solution.genome[0..25]);
        }
        let to_add = build_population()
            .with_genome_builder(ValueEncodedGenomeBuilder::new(1, 0f64, 359.99999999999994))
            .of_size(population.len())
            .uniform_at_random();
        for (p, t) in population.iter_mut().zip(to_add.individuals().iter()) {
            p.extend(t);
            if p.len() > 50 {
                *p = p[25..].to_vec();
            }
        }
        println!("{:?}", inputs);
        ga_sim = simulate(
            // TODO: all the options need to be checked here too
            genetic_algorithm()
                .with_evaluation(simulator.clone())
                .with_selection(MaximizeSelector::new(0.85, 12))
                .with_crossover(SinglePointCrossBreeder::new())
                .with_mutation(RandomValueMutator::new(0.2, 0f64, 359.99999999999994))
                .with_reinsertion(ElitistReinserter::new(simulator.clone(), true, 0.85))
                .with_initial_population(Population::with_individuals(population))
                .build(),
        )
        .until(GenerationLimit::new(20)) // TODO: yes, here too
        .build();
    }
}

#[derive(Error, Debug)]
pub enum AlgorithmError {
    #[error(transparent)]
    DataParseError(#[from] DataParseError),
}

pub fn run_alg(level: Level, player: Player, checkpoints: &str) -> Result<(), AlgorithmError> {
    let base_inputs = initial_path(&level, player, parse_checkpoint(checkpoints)?);
    // TODO: the rest of the damn thing
    let out = format_inputs(base_inputs);
    println!("{out}");
    let mut clipboard = Clipboard::new().unwrap();
    clipboard.set_text(out).unwrap();
    Ok(())
}

fn format_inputs(inp: Inputs) -> String {
    let mut count = 1;
    let mut current = inp[0];
    let mut out = "".to_owned();
    for i in inp[1..].iter() {
        if *i == current {
            count += 1;
        } else {
            out += &format!("{count},f,{current}\n");
            count = 1;
            current = *i;
        }
    }
    out += &format!("{count},f,{current}\n");
    out
}

#[cfg(test)]
mod tests {
    use crate::algorithm::format_inputs;

    #[test]
    fn format_inputs_test() {
        let expected = format!("1,f,{}\n5,f,{}\n2,f,{}\n", 4.2f64, 99.3f64, 55.9f64);
        let got = format_inputs(vec![4.2, 99.3, 99.3, 99.3, 99.3, 99.3, 55.9, 55.9]);
        assert_eq!(expected, got);
    }
}
