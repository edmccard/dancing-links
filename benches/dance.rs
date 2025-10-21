use criterion::{Criterion, criterion_group, criterion_main};

use dlx::choose::{Choose, mrv_chooser, no_tiebreak, prefer_any};
use dlx::x::{INodes, ONodes, Problem};
use dlx::{Count, OptOrder, Solve, Solver};

fn bench_dance(c: &mut Criterion) {
    let items = INodes::new(7, 0);
    let os: Vec<Vec<Count>> = vec![
        vec![2, 4],
        vec![0, 3, 6],
        vec![1, 2, 5],
        vec![0, 3, 5],
        vec![1, 6],
        vec![3, 4, 6],
    ];
    let opts = ONodes::new(7, 7, &os, OptOrder::Seq);
    let problem = Problem::new(items, opts);
    let mut solver = Solver::new(problem);
    let mut chooser = mrv_chooser(prefer_any(), no_tiebreak());
    c.bench_function("dance", |b| {
        b.iter(|| {
            solve(&mut solver, &mut chooser);
        })
    });
}

fn solve<S: Solve, C: Choose<S>>(
    solver: &mut Solver<S>, chooser: &mut C,
) -> usize {
    let mut i = 0;
    while solver.next_solution(chooser) {
        i += 1;
    }
    i
}

criterion_group!(benches, bench_dance);
criterion_main!(benches);
