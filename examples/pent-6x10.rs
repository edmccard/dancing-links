use dlx::choose::*;
use dlx::p::Preproc;
use dlx::x::make_problem;
use dlx::{OptOrder, Solver};

include!("./common/polyomino.rs");

fn main() {
    let ps = pentominoes();
    let bx = rectangle(6, 10);

    let mut os = Vec::new();
    for p in 0..9 {
        for base in ps[p].bases() {
            os.extend(base.options(p as Coord, &bx));
        }
    }
    os.extend(ps[9].options_within(9, 0, 0, 5, 3, &bx));
    for p in 10..12 {
        for base in ps[p].bases() {
            os.extend(base.options(p as Coord, &bx));
        }
    }

    let mut chooser = mrv_chooser(prefer_any(), no_tiebreak());

    let problem = make_problem(72, 0, &os, OptOrder::Seq);
    let mut solver = Solver::new(problem);
    let mut sols = 0;
    while solver.next_solution(&mut chooser) {
        sols += 1;
    }
    println!(
        "Before redutcion: {} solutions, {} updates",
        sols,
        solver.get_updates()
    );

    let mut problem = make_problem(72, 0, &os, OptOrder::Seq);
    let mut preproc = Preproc::new(&mut problem);
    let rd = preproc.reduce(200).unwrap();
    let reduced = make_problem(rd.0, rd.1, &rd.2, OptOrder::Seq);
    let mut solver = Solver::new(reduced);
    let mut sols = 0;
    while solver.next_solution(&mut chooser) {
        sols += 1;
    }
    println!(
        "After redutcion: {} solutions, {} updates",
        sols,
        solver.get_updates()
    );
}
