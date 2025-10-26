// In most implementations of the dancing links algorithm, the `solve` method
// is recursive; it will not return until all solutions are found. The `dlx`
// crate provides an iterative method (`next_solution`), so you can get the
// solutions one at a time.

extern crate dlx;

use dlx::Solve;
use dlx::choose::*;

include!("./common/sudoku.rs");

fn main() {
    // This sudoku puzzle is "under-soecified" and has more than one solution.
    let puzzle = ".3..1.......4..1...5.....9.2.....6.4....35...1........4..6............5..9.......";
    let clues = Clues::from_sdm(puzzle);
    print_grid(&clues.p);
    println!("");
    let (mut problem, os, names) = clues.make_problem(OptOrder::Seq);
    let chooser = mrv_chooser(prefer_any(), no_tiebreak());
    let solutions = SolveIter { solver: Solver::new(&mut problem), chooser };

    for solution in solutions {
        print_grid(&clues.solution_grid(&solution, &os, &names));
        println!("");
    }
}

struct SolveIter<'a, P, C> {
    solver: Solver<'a, P>,
    chooser: C,
}

impl<'a, P: Solve, C: Choose<P>> Iterator for SolveIter<'a, P, C> {
    type Item = Vec<Data>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.solver.next_solution(&mut self.chooser) {
            Some(self.solver.fmt_solution().to_vec())
        } else {
            None
        }
    }
}
