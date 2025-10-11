extern crate dlx;

use std::collections::HashMap;

use dlx::choose::*;
use dlx::x::{Problem, make_problem};
use dlx::{Count, Data, Link, OptOrder, Rng, Solver};

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
    print_solution(&clues.solution_grid(&solution, &os, &names));
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

fn print_solution(grid: &ClueData) {
    for j in 0..9 {
        println!(
            "{}",
            grid[j]
                .iter()
                .map(|&d| char::from_u32((d as u32) + ('0' as u32)).unwrap())
                .collect::<String>()
        );
    }
}

type ClueData = [[usize; 9]; 9];

struct Clues {
    p: ClueData,
    r: ClueData,
    c: ClueData,
    b: ClueData,
}

impl Clues {
    fn from_sdm(sdm: &str) -> Clues {
        let mut p = [[0usize; 9]; 9];
        let mut r = [[0usize; 9]; 9];
        let mut c = [[0usize; 9]; 9];
        let mut b = [[0usize; 9]; 9];
        let sdm = &sdm[0..81];

        for (i, d) in sdm.chars().enumerate() {
            if d < '1' || d > '9' {
                continue;
            }
            let d = ((d as u32) - ('1' as u32)) as usize;
            let j = i / 9;
            let k = i % 9;
            p[j][k] = d + 1;
            r[j][d] = k + 1;
            c[k][d] = j + 1;
            b[Clues::bx_no(j, k)][d] = j + 1;
        }

        Clues { p, r, c, b }
    }

    fn make_problem(
        &self, order: OptOrder,
    ) -> (Problem, Vec<Vec<Link>>, Vec<Count>) {
        let mut p_names = Vec::new();
        let mut r_names = Vec::new();
        let mut c_names = Vec::new();
        let mut b_names = Vec::new();
        for j in 0..9 {
            for k in 0..9 {
                if self.p[j][k] == 0 {
                    p_names.push(Link(j * 9 + k));
                }
                if self.r[j][k] == 0 {
                    r_names.push(Link(81 + (j * 9 + k)));
                }
                if self.c[j][k] == 0 {
                    c_names.push(Link(162 + (j * 9 + k)));
                }
                if self.b[j][k] == 0 {
                    b_names.push(Link(243 + (j * 9 + k)));
                }
            }
        }

        let names = [p_names, r_names, c_names, b_names].concat();
        let mut items = HashMap::new();
        for (n, &i) in names.iter().enumerate() {
            items.insert(i, Link(n));
        }

        let mut os = Vec::new();
        for j in 0..9 {
            for k in 0..9 {
                let x = Clues::bx_no(j, k);
                for d in 0..9 {
                    if self.p[j][k] == 0
                        && self.r[j][d] == 0
                        && self.c[k][d] == 0
                        && self.b[x][d] == 0
                    {
                        os.push(vec![
                            items[&Link(j * 9 + k)],
                            items[&Link(81 + (j * 9 + d))],
                            items[&Link(162 + (k * 9 + d))],
                            items[&Link(243 + (x * 9 + d))],
                        ]);
                    }
                }
            }
        }

        (make_problem(Count(names.len()), 0, &os, order), os, names)
    }

    fn solution_grid(
        &self, solution: &[Data], os: &[Vec<Link>], names: &[Count],
    ) -> ClueData {
        let mut g = self.p.clone();
        for &i in solution {
            let opt = &os[i as usize];
            let j = names[opt[0] as usize] / 9;
            let k = names[opt[0] as usize] % 9;
            let d = (names[opt[1] as usize] - 81) % 9;
            g[j as usize][k as usize] = (d + 1) as usize;
        }
        g
    }

    fn bx_no(j: usize, k: usize) -> usize {
        (j / 3) * 3 + (k / 3)
    }
}
