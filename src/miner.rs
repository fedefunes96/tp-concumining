extern crate rand;
extern crate rand_distr;

static MAX_GOLD_PIPS: f64 = 20.0;

use rand_distr::{Distribution, Beta};

pub fn explore() -> u32 {
    let mut rng = rand::thread_rng();

    let beta = Beta::new(2.0, 5.0).unwrap();

    return (beta.sample(&mut rng) * MAX_GOLD_PIPS) as u32;
}
