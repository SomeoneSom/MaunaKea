pub struct Options {
    pub decimals: i32,
    pub opt2: i32,
    pub opt3: i32,
    pub opt4: i32
}

pub static mut OPTS:Options = Options {
    decimals: 3,
    opt2: 0,
    opt3: 0,
    opt4: 0
};

/*pub static mut ANGLES:[(f32, f32); 360000] = [(0., 0.); 360000];

const fn comp_angles() -> [(f32, f32); 360000] {
    let mut angles:[(f32, f32); 360000] = [(0., 0.); 360000];
    let mut i = 0;
    while i < 360000 {
        let x:f32 =  (60. * 80.) / ((640. + 360. * ((i as f32 / 1000.).to_radians().tan()).powi(2)).sqrt())
            * if i < 90000 || i > 270000 {1.} else {-1.};
        let y:f32 = (60. * 80.) / ((360. + 640. / ((i as f32 / 1000.).to_radians().tan()).powi(2)).sqrt())
            * if i < 180000 {1.} else {-1.};
        angles[i] = (x, y);
        i += 1;
    }
    return angles;
}*/