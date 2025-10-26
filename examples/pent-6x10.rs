use std::time::Instant;

use dlx::choose::*;
use dlx::p::Preproc;
use dlx::x::{Problem, make_problem};
use dlx::{OptOrder, Solver};

include!("./common/polyomino.rs");

fn main() {
    let ps = pentominoes();
    let bx = rectangle(6, 10);

    let mut os = Vec::new();
    for p in 0..9 {
        for base in ps[p].bases() {
            os.extend(base.options(Count(p), &bx));
        }
    }
    os.extend(ps[9].options_within(9, 0, 0, 5, 3, &bx));
    for p in 10..12 {
        for base in ps[p].bases() {
            os.extend(base.options(Count(p), &bx));
        }
    }

    let mut problem = make_problem(72, 0, &os, OptOrder::Seq);

    println!("Before reduction:");
    check_problem(&mut problem);

    let start = Instant::now();
    let rd = Preproc::new(&mut problem).reduce(200).unwrap();
    let mut reduced = make_problem(rd.0, rd.1, &rd.2, OptOrder::Seq);
    println!("Reduction took {:?}", start.elapsed());

    println!("After reduction");
    check_problem(&mut reduced);
}

fn check_problem(problem: &mut Problem) {
    let mut solver = Solver::new(problem);
    let mut chooser = mrv_chooser(prefer_any(), no_tiebreak());
    let mut sols = 0;
    let start = Instant::now();
    while solver.next_solution(&mut chooser) {
        sols += 1;
    }
    println!(
        "    {} solutions, {} updates in {:?}",
        sols,
        solver.get_updates(),
        start.elapsed()
    );
}
