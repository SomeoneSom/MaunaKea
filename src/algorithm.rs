mod geneticalg;
mod waterspeed;
//use std::time::{Duration, Instant};
use std::io::stdout;
use std::io::Write;
use std::num::ParseFloatError;

use crate::colliders::Collider;
use crate::colliders::Rect;
use crate::level::Level;
use crate::player::Player;
use crate::point::Point;

use bitvec::prelude as bv;
use colored::Colorize;
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

pub fn run_alg(level: Level, checkpoints: String) {
    todo!();
}
