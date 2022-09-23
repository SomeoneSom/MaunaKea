//use crate::globals::{ANGLES};
//use std::time::{Duration, Instant};

pub struct Initializer {}

impl Initializer {
    /*pub fn comp_angles() -> () {
        let start = Instant::now();
        let mut vec:Vec<(f32, f32)> = Vec::new();
        for i in 0..360000 {
            let x:f32 =  (60. * 80.) / ((640. + 360. * ((i as f32 / 1000.).to_radians().tan()).powi(2)).sqrt())
                * if i < 90000 || i > 270000 {1.} else {-1.};
            let y:f32 = (60. * 80.) / ((360. + 640. / ((i as f32 / 1000.).to_radians().tan()).powi(2)).sqrt())
                * if i < 180000 {1.} else {-1.};
            vec.push((x, y));
        }
        let boxes:Box<[(f32, f32)]> = vec.clone().into_boxed_slice();
        vec.clear();
        unsafe {
            for i in 0..360000 {
                ANGLES[i] = boxes[i];
            }
            PRE_COMPING = false;
        }
        let duration = start.elapsed();
        println!("Precomputing done in {:?}!", duration);
    }*/
}

pub struct Algorithm {

}

//current approach will be brute force, then use a GA to tweak. this is prob bad, but it will be improved later
impl Algorithm {
    pub fn new() -> Self {
        Self {

        }
    }
}