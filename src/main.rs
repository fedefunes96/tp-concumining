#[macro_use]
extern crate clap;

use clap::{App, Arg};

mod ipc;
mod workers;
mod concumining;
mod miners_info;

fn main() {
    let app = App::new("Concumining")
        .arg(Arg::with_name("miners")
                .short("m")
                .long("miners")
                .value_name("miners")
                .help("Miners to simulate")
                .takes_value(true).required(true))
        .arg(Arg::with_name("rounds")
                .short("r")
                .long("rounds")
                .value_name("rounds")
                .help("Amount of rounds to simulate")
                .takes_value(true).required(true));
    let matches = app.get_matches();

    let miners = value_t!(matches, "miners", usize).unwrap();
    let rounds = value_t!(matches, "rounds", usize).unwrap();
    println!("Simulating {} miners, {} rounds...", miners, rounds);
    let simulation = concumining::concumining::Concumining::new(miners, rounds);
    simulation.start();
}
