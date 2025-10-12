extern crate dlx;

use dlx::Rng;
use dlx::choose::*;

include!("./common/sudoku.rs");

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let puzzle = if args.len() > 1 {
        std::fs::read_to_string(&args[1]).expect("Couldn't open file")
    } else {
        "12.3..4..5..4..1......2..6.7...........7...31....547..4..5..3...8.........9.4....".into()
    };

    let clues = Clues::from_sdm(&puzzle);
    let (problem, os, names) = clues.make_problem(OptOrder::Seq);
    let solution = verify_problem(problem);
    print_grid(&clues.solution_grid(&solution, &os, &names));
    rate_problem(&clues);
}

fn verify_problem(problem: Problem) -> Vec<Data> {
    let mut chooser = mrv_chooser(prefer_any(), no_tiebreak());
    let mut solver = Solver::new(problem);
    let mut n = 0;
    let mut solutions = Vec::new();
    while solver.next_solution(&mut chooser) {
        solutions.push(solver.get_solution().to_vec());
        n += 1;
        if n > 1 {
            panic!("Too many solutions");
        }
    }
    if n == 0 {
        panic!("No solution");
    }
    solutions[0].clone()
}

fn rate_problem(clues: &Clues) {
    let mut seeds = Rng::new(12345678);
    let mut chooser = mrv_chooser(prefer_any(), rnd_tiebreak(seeds.next()));
    let mut updates = Vec::new();
    let mut profile = Vec::new();
    for _ in 0..10 {
        let seed = seeds.next();
        let (problem, ..) = clues.make_problem(OptOrder::Rnd(Rng::new(seed)));
        let mut solver = Solver::new(problem);
        solver.next_solution(&mut chooser);
        profile.push(solver.get_profile().iter().sum::<usize>());
        updates.push(solver.get_updates());
    }
    println!(
        "UPDATES (min/avg/max) {} {} {}",
        updates.iter().min().unwrap(),
        updates.iter().sum::<isize>() / (updates.len() as isize),
        updates.iter().max().unwrap(),
    );
    println!(
        "NODES (min/avg/max) {} {} {}",
        profile.iter().min().unwrap(),
        profile.iter().sum::<usize>() / updates.len(),
        profile.iter().max().unwrap(),
    );
}
