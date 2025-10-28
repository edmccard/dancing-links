use std::collections::HashMap;

use dlx::x::{Problem, make_problem};
use dlx::{Int, OptOrder, Solver, Uint};

fn print_grid(grid: &ClueData) {
    for j in 0..9 {
        println!(
            "{}",
            grid[j]
                .iter()
                .map(|&d| {
                    if d >= 1 && d <= 9 {
                        char::from_u32((d as u32) + ('0' as u32)).unwrap()
                    } else {
                        '.'
                    }
                })
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
    ) -> (Problem, Vec<Vec<Uint>>, Vec<Uint>) {
        let mut p_names = Vec::new();
        let mut r_names = Vec::new();
        let mut c_names = Vec::new();
        let mut b_names = Vec::new();
        for j in 0..9 {
            for k in 0..9 {
                if self.p[j][k] == 0 {
                    p_names.push(Uint(j * 9 + k));
                }
                if self.r[j][k] == 0 {
                    r_names.push(Uint(81 + (j * 9 + k)));
                }
                if self.c[j][k] == 0 {
                    c_names.push(Uint(162 + (j * 9 + k)));
                }
                if self.b[j][k] == 0 {
                    b_names.push(Uint(243 + (j * 9 + k)));
                }
            }
        }

        let names = [p_names, r_names, c_names, b_names].concat();
        let mut items = HashMap::new();
        for (n, &i) in names.iter().enumerate() {
            items.insert(i, Uint(n));
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
                            items[&Uint(j * 9 + k)],
                            items[&Uint(81 + (j * 9 + d))],
                            items[&Uint(162 + (k * 9 + d))],
                            items[&Uint(243 + (x * 9 + d))],
                        ]);
                    }
                }
            }
        }

        (make_problem(Uint(names.len()), 0, &os, order), os, names)
    }

    fn solution_grid(
        &self, solution: &[Int], os: &[Vec<Uint>], names: &[Uint],
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
