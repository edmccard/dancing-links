use criterion::{Criterion, criterion_group, criterion_main};

use dlx::x::{INodes, ONodes, Problem};
use dlx::{Count, Solve, Solver};

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
    let opts = ONodes::new(7, 6, 16, os);
    let problem = Problem::new(items, opts);
    let mut solver = Solver::new(problem);
    c.bench_function("dance", |b| {
        b.iter(|| {
            solve(&mut solver);
        })
    });
}

fn solve<S: Solve>(solver: &mut Solver<S>) -> usize {
    let mut i = 0;
    while solver.next_solution() {
        i += 1;
    }
    i
}

criterion_group!(benches, bench_dance);
criterion_main!(benches);
