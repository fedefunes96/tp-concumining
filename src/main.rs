mod concumining;

fn main() {
    let simulation = concumining::Concumining::new(5, 5);
    simulation.start();
}
