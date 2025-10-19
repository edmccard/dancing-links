use anyhow::Result;

use crate::{Count, Dance, Data, Link, Solve, Spec};
use crate::{c, m, x};

#[derive(Debug, Eq, PartialEq)]
pub struct Problem {
    items: m::INodes,
    opts: c::ONodes,
    ft: Vec<Link>,
    updates: isize,
}

impl Problem {
    pub fn new(items: m::INodes, opts: c::ONodes) -> Problem {
        Problem { items, opts, ft: Vec::new(), updates: 0 }
    }

    pub fn from_spec(spec: &Spec) -> Result<Problem> {
        let (items, names) = m::INodes::from_spec(spec)?;
        let opts = c::ONodes::from_spec(spec, &names)?;
        Ok(Problem::new(items, opts))
    }
}

impl Dance for Problem {
    type I = m::INodes;
    type O = c::ONodes;

    #[inline]
    fn items(&mut self) -> &mut Self::I {
        &mut self.items
    }

    #[inline]
    fn opts(&mut self) -> &mut Self::O {
        &mut self.opts
    }

    #[inline]
    fn updates(&mut self) -> &mut isize {
        &mut self.updates
    }

    #[inline]
    fn cover(&mut self, i: Link) {
        x::cover(i, self);
    }

    #[inline]
    fn commit(&mut self, p: Link, j: Link) {
        c::commit(p, j, self);
    }

    #[inline]
    fn uncover(&mut self, i: Link) {
        x::uncover(i, self);
    }

    #[inline]
    fn uncommit(&mut self, p: Link, j: Link) {
        c::uncommit(p, j, self);
    }

    #[inline]
    fn hide(&mut self, p: Link) {
        c::hide(p, self);
    }

    #[inline]
    fn unhide(&mut self, p: Link) {
        c::unhide(p, self);
    }

    #[inline]
    fn branch_degree(&mut self, i: Link) -> Data {
        m::branch_degree(i, self)
    }
}

impl c::DanceC for Problem {
    #[inline]
    fn purify(&mut self, p: Link) {
        c::purify(p, self);
    }

    #[inline]
    fn unpurify(&mut self, p: Link) {
        c::unpurify(p, self);
    }
}

impl m::DanceM for Problem {
    #[inline]
    fn tweak(&mut self, x: Link, p: Link) {
        m::tweak(x, p, self);
    }

    #[inline]
    fn untweak(&mut self, l: Count, unblock: bool) {
        m::untweak(l, unblock, self);
    }

    #[inline]
    fn ft(&mut self) -> &mut Vec<Link> {
        &mut self.ft
    }
}

impl Solve for Problem {
    #[inline]
    fn enter_level(&mut self, i: Link, l: Count, xl: Link) {
        m::enter_level(self, i, l, xl);
    }

    #[inline]
    fn prepare_to_branch(&mut self, i: Link, l: Count, xl: Link) {
        m::prepare_to_branch(self, i, l, xl);
    }

    #[inline]
    fn try_item(&mut self, i: Link, l: Count, xl: Link) -> bool {
        m::try_item(self, i, l, xl)
    }

    #[inline]
    fn try_again(&mut self, i: Link, l: Count, xl: &mut Link) -> bool {
        m::try_again(self, i, l, xl)
    }

    #[inline]
    fn restore_item(&mut self, i: Link, l: Count, xl: Link) {
        m::restore_item(self, i, l, xl);
    }
}

impl m::SolveM for Problem {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcc() {
        use crate::choose::*;
        use crate::{OptOrder, Solver};
        let ps = vec![(1, 1), (1, 1), (2, 3)];
        let items = m::INodes::new(ps, 2);
        let os = vec![
            vec![(0, 0), (1, 0), (3, 0), (4, 0)],
            vec![(0, 0), (2, 0), (3, 1), (4, 1)],
            vec![(2, 0), (3, 0)],
            vec![(1, 0), (3, 1)],
            vec![(2, 0), (4, 1)],
        ];
        let opts = c::ONodes::new(5, 3, &os, OptOrder::Seq);
        let items_init = items.clone();
        let opts_init = opts.clone();
        let problem = Problem::new(items, opts);
        let mut solver = Solver::new(problem);
        let mut solutions: Vec<Vec<Data>> = Vec::new();
        let mut expected = vec![vec![1, 3, 4]];
        let mut i = 0;
        let mut chooser = mrv_chooser(prefer_any(), no_tiebreak());
        while solver.next_solution(&mut chooser) {
            assert!(i <= expected.len(), "too many solutions");
            solver.get_solution();
            solver.o.sort();
            solutions.push(solver.o.clone());
            i += 1;
        }
        solutions.sort();
        expected.sort();
        assert_eq!(solutions, expected, "wrong solutions");
        assert_eq!(solver.problem.items, items_init, "items not backtracked");
        assert_eq!(solver.problem.opts, opts_init, "options not backtracked");
        assert!(
            solver.l == 0 && solver.restart == false,
            "initial state not restored"
        );
    }
}
