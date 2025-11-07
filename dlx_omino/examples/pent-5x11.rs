extern crate dlx;

use dlx::choose::*;
use dlx::x::make_problem;
use dlx::{OptOrder, Solver, Uint};

use dlx_omino::*;

struct Info;
impl SpecInfo for Info {
    type OData = Uint;
    const PIECE_COUNT: usize = 11;
    const CELL_COUNT: usize = 55;
}

fn main() {
    let ps = pentominoes();
    let bx = rectangle(5, 11);
    let info = Info;

    let mut os = Vec::new();
    // Skip the "O" pentomino
    for (i, p) in ps[1..].iter().enumerate() {
        // For the "X" pentomino ...
        if i == 8 {
            os.extend(p.options(8, &bx, &info, |mut oo, pp| {
                // ... restrict it to the upper left quadrant ...
                if !Bounds(0, 0, 6, 3).contains(&pp.bounds()) {
                    vec![]
                } else {
                    let (x, y) = pp.cell_at(2);
                    // ... add secondary item "s" if in either
                    // middle row or middle column ...
                    if (x == 5) || (y == 2) {
                        oo.push(66);
                        // .. and add secondary item "c" if in dead
                        // center.
                        if x == 5 && y == 2 {
                            oo.push(67);
                        }
                    }
                    oo
                }
            }));
        }
        // For the "Z" pentomino ...
        else if i == 10 {
            os.extend(
                p.transform(0b1111)
                    .iter()
                    .flat_map(|t| t.all_options(10, &bx, &info)),
            );
            // ... add secondary item "s" if flipped.
            os.extend(p.transform(0b11110000).iter().flat_map(|t| {
                t.options(10, &bx, &info, |mut oo, _| {
                    oo.push(66);
                    oo
                })
            }));
        }
        // For the "W" pentomino ...
        else if i == 7 {
            os.extend(
                p.transform(0b1101)
                    .iter()
                    .flat_map(|t| t.all_options(7, &bx, &info)),
            );
            // ... add secondary item "c" for 90° rotation.
            os.extend(p.transform(0b0010).iter().flat_map(|t| {
                t.options(7, &bx, &info, |mut oo, _| {
                    oo.push(67);
                    oo
                })
            }));
        }
        // And the rest.
        else {
            os.extend(
                p.transform(0b11111111)
                    .iter()
                    .flat_map(|t| t.all_options(Uint(i), &bx, &info)),
            );
        }
    }

    let mut problem = make_problem(66, 2, &os, OptOrder::Seq);
    let mut solver = Solver::new(&mut problem);
    let mut chooser = mrv_chooser(prefer_any(), no_tiebreak());
    let mut sols = 0;
    while solver.next_solution(&mut chooser) {
        sols += 1;
        if sols == 1 {
            println!("First solution:");
            let sol = solver.fmt_solution();
            let grid = SolutionGrid::new(sol, &info, &os, &bx);
            grid.colorize(' ', &PALETTE_12[1..]);
        }
    }
    println!("Total solutions: {}", sols);
}
