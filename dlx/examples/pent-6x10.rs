extern crate dlx_omino;

use std::time::Instant;

use dlx::choose::*;
use dlx::p::Preproc;
use dlx::x::{Problem, make_problem};
use dlx::{OptOrder, Solver, Uint};

use dlx_omino::*;

struct Info;
impl SpecInfo for Info {
    type OData = Uint;
    const PIECE_COUNT: usize = 12;
    const CELL_COUNT: usize = 60;
}

fn main() {
    let ps = pentominoes();
    let bx = rectangle(6, 10);
    let info = Info;

    let mut os = Vec::new();
    for (i, p) in ps.iter().enumerate() {
        if i == 9 {
            for o in p.options_filter(9, &bx, &info, |pp| {
                Bounds(0, 0, 5, 3).contains(&pp.bounds())
            }) {
                os.push(o);
            }
        } else {
            for t in p.transform(255) {
                for o in t.all_options(Uint(i), &bx, &info) {
                    os.push(o);
                }
            }
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
