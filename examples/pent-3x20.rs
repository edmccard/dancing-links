include!("./common/polyomino.rs");

use std::io::IsTerminal;

use termcolor::{ColorChoice, StandardStream};

use dlx::choose::*;
use dlx::x::make_problem;
use dlx::{Data, OptOrder, Solver};

fn main() {
    let ps = pentominoes();
    let bx = rectangle(3, 20);
    let os = Omino::all_options(&ps, &bx);

    let mut problem = make_problem(72, 0, &os, OptOrder::Seq);
    let mut solver = Solver::new(&mut problem);
    let mut chooser = mrv_chooser(prefer_any(), no_tiebreak());
    while solver.next_solution(&mut chooser) {
        let grid = fill_grid(&bx, solver.fmt_solution(), &os);
        print_grid(&grid, "OPQRSTUVWXYZ");
        println!("");
    }
}

fn fill_grid(bx: &Shape, sol: &[Data], os: &[Vec<Count>]) -> Vec<Vec<usize>> {
    let mut grid =
        vec![vec![0; (bx.xmax + 1) as usize]; (bx.ymax + 1) as usize];
    for &opt in sol {
        let o = &os[opt as usize];
        for &itm in o[..o.len() - 1].iter() {
            let (x, y) = bx.cell_at(itm);
            let p = (o[o.len() - 1] as usize) - bx.size();
            grid[y][x] = p + 1;
        }
    }
    grid
}

fn print_grid(grid: &[Vec<usize>], names: &str) {
    let names: Vec<char> = names.chars().collect();
    if !std::io::stdin().is_terminal() {
        for line in grid {
            println!(
                "{}",
                line.iter()
                    .map(|&c| if c == 0 { ' ' } else { names[c - 1] })
                    .collect::<String>()
            );
        }
        return;
    }

    use std::io::Write;
    use termcolor::{Color, ColorSpec, WriteColor};
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    for line in grid {
        for &p in line {
            if p == 0 {
                write!(&mut stdout, " ").unwrap();
                continue;
            }
            stdout
                .set_color(
                    ColorSpec::new()
                        .set_fg(Some(Color::Ansi256((p - 1) as u8))),
                )
                .unwrap();
            write!(&mut stdout, "█").unwrap();
        }
        writeln!(&mut stdout).unwrap();
    }
    stdout.reset().unwrap();
}
