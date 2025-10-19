use anyhow::{Result, bail};

use crate::x;
use crate::{Count, Dance, Data, Items, Link, OptOrder, Solve, Spec};

pub fn tweak<D: DanceM<I: ItemsM>>(x: Link, p: Link, dance: &mut D) {
    if *dance.bound(p) != 0 {
        dance.hide(x);
    }
    let d = *dance.dlink(x);
    *dance.dlink(p) = d;
    *dance.ulink(d) = p;
    *dance.len(p) -= 1;
}

pub fn untweak<D: DanceM<I: ItemsM>>(l: Count, unblock: bool, dance: &mut D) {
    let ftl = dance.ft()[l as usize];
    let p = if ftl <= dance.items().count() {
        ftl
    } else {
        *dance.top(ftl) as Link
    };
    let mut x = ftl;
    let mut y = p;
    let z = *dance.dlink(p);
    *dance.dlink(p) = x;
    let mut k = 0;
    while x != z {
        *dance.ulink(x) = y;
        k += 1;
        if unblock {
            dance.unhide(x);
        }
        y = x;
        x = *dance.dlink(x);
    }
    *dance.ulink(z) = y;
    *dance.len(p) += k;
    if !unblock {
        dance.uncover(p);
    }
}

pub fn branch_degree<D: DanceM<I: ItemsM>>(i: Link, dance: &mut D) -> Data {
    (*dance.len(i) + 1)
        .saturating_sub((*dance.bound(i)).saturating_sub(dance.slack(i)))
}

pub fn enter_level<S: SolveM>(solve: &mut S, _: Link, _: Count, _: Link) {
    solve.ft().push(0);
}

pub fn prepare_to_branch<S: SolveM>(
    solve: &mut S, i: Link, l: Count, xl: Link,
) {
    *solve.bound(i) -= 1;
    if *solve.bound(i) == 0 {
        solve.cover(i);
        if solve.slack(i) != 0 {
            solve.ft()[l as usize] = xl;
        }
    } else {
        solve.ft()[l as usize] = xl;
    }
}

pub fn try_item<S: SolveM>(solve: &mut S, i: Link, _: Count, xl: Link) -> bool {
    if solve.slack(i) == 0 && *solve.bound(i) == 0 {
        if xl == i {
            return false;
            // go to M8
        }
        // go to M6
    } else if *solve.len(i) <= (*solve.bound(i) - solve.slack(i)) {
        return false;
        // go to M8
    } else if xl != i {
        solve.tweak(xl, i);
    } else if *solve.bound(i) != 0 {
        let p = *solve.llink(i);
        let q = *solve.rlink(i);
        *solve.rlink(p) = q;
        *solve.llink(q) = p;
    }
    // M6
    if xl != i {
        let mut p = xl + 1;
        while p != xl {
            let j = *solve.top(p);
            if j <= 0 {
                p = *solve.ulink(p);
            } else if j as Count <= solve.items().primary() {
                p += 1;
                *solve.bound(j as Link) -= 1;
                if *solve.bound(j as Link) == 0 {
                    solve.cover(j as Link);
                }
            } else {
                solve.commit(p, j as Link);
                p += 1;
            }
        }
    }
    true
}

pub fn try_again<S: SolveM>(
    solve: &mut S, i: Link, l: Count, xl: &mut Link,
) -> bool {
    let mut i = i;
    let again = if *xl > solve.items().count() {
        let mut p = *xl - 1;
        while p != *xl {
            let j = *solve.top(p);
            if j <= 0 {
                p = *solve.dlink(p);
            } else if (j as Link) <= solve.items().primary() {
                p -= 1;
                *solve.bound(j as Link) += 1;
                if *solve.bound(j as Link) == 1 {
                    solve.uncover(j as Link);
                }
            } else {
                solve.uncommit(p, j as Link);
                p -= 1;
            }
        }
        *xl = *solve.dlink(*xl);
        solve.try_item(i, l, *xl)
    } else {
        i = *xl;
        let p = *solve.llink(i);
        let q = *solve.rlink(i);
        *solve.rlink(p) = i;
        *solve.llink(q) = i;
        false
    };
    if !again {
        solve.restore_item(i, l, *xl);
    }
    again
}

pub fn restore_item<S: SolveM>(solve: &mut S, i: Link, l: Count, _: Link) {
    if *solve.bound(i) == 0 && solve.slack(i) == 0 {
        solve.uncover(i);
    } else {
        let unblock = *solve.bound(i) != 0;
        solve.untweak(l, unblock);
    }
    *solve.bound(i) += 1;
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct INode {
    left: Link,
    right: Link,
    slack: Data,
    bound: Data,
}

#[derive(Clone, Default, Debug, Eq, PartialEq)]
pub struct INodes {
    nodes: Vec<INode>,
    primary: Count,
    len: Count,
}

impl INodes {
    pub fn new(
        ps: impl IntoIterator<Item = (Data, Data)>, ns: Count,
    ) -> INodes {
        let mut nodes = vec![Default::default()];
        for (u, v) in ps.into_iter() {
            nodes.push(INode { bound: v, slack: v - u, ..Default::default() });
        }
        let primary = (nodes.len() - 1) as Count;
        for _ in 0..=ns {
            nodes.push(Default::default());
        }
        let mut inodes = INodes { nodes, primary, len: primary + ns };
        inodes.init_links();
        inodes
    }

    pub fn from_spec(spec: &Spec) -> Result<(INodes, Vec<String>)> {
        use std::collections::HashSet;
        let mut names: Vec<String> = Vec::new();
        let mut ps = Vec::new();
        for item in &spec.primary {
            let (name, u, v) = if item.contains('|') {
                let data = item.split('|').collect::<Vec<_>>();
                if data.len() > 2 {
                    bail!("Too many '|' (multiplicity) separators");
                }
                let name = data[1];
                let data = data[0];
                if data.contains(':') {
                    let data = data.split(':').collect::<Vec<_>>();
                    if data.len() > 2 {
                        bail!("Too many ':' (multiplicity) separators");
                    }
                    (name, data[0], data[1])
                } else {
                    (name, data, data)
                }
            } else {
                (item.as_str(), "1", "1")
            };
            names.push(name.into());
            let u: Data = u.parse().or_else(|_| bail!("non-numeric bound"))?;
            let v: Data = v.parse().or_else(|_| bail!("non-numeric bound"))?;
            ps.push((u, v));
        }
        for item in &spec.secondary {
            names.push(item.into());
        }
        let mut used = HashSet::new();
        let unique = names.iter().all(|e| used.insert(e));
        if !unique {
            bail!("Duplicate item names");
        }
        for name in &names {
            if !name.chars().all(|c| c.is_alphanumeric() || c == '#') {
                bail!("Invalid item name");
            }
        }
        let ns = spec.secondary.len() as Count;
        Ok((INodes::new(ps, ns), names))
    }

    #[inline]
    fn get_node(&mut self, i: Link) -> &mut INode {
        if cfg!(feature = "unsafe-fast-index") {
            unsafe { self.nodes.get_unchecked_mut(i as usize) }
        } else {
            &mut self.nodes[i as usize]
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Problem {
    items: INodes,
    opts: x::ONodes,
    ft: Vec<Link>,
    updates: isize,
}

impl Problem {
    pub fn new(items: INodes, opts: x::ONodes) -> Problem {
        Problem { items, opts, ft: Vec::new(), updates: 0 }
    }

    pub fn from_spec(spec: &Spec, order: OptOrder) -> Result<Problem> {
        let (items, names) = INodes::from_spec(spec)?;
        let opts = x::ONodes::from_spec(spec, &names, order)?;
        Ok(Problem::new(items, opts))
    }
}

impl Items for INodes {
    #[inline]
    fn llink(&mut self, i: Link) -> &mut Link {
        &mut self.get_node(i).left
    }

    #[inline]
    fn rlink(&mut self, i: Link) -> &mut Link {
        &mut self.get_node(i).right
    }

    #[inline]
    fn primary(&self) -> Count {
        self.primary
    }

    #[inline]
    fn count(&self) -> Count {
        self.len
    }
}

pub trait ItemsM: Items {
    fn bound(&mut self, i: Link) -> &mut Data;
    fn slack(&mut self, i: Link) -> Data;
}

impl ItemsM for INodes {
    #[inline]
    fn bound(&mut self, i: Link) -> &mut Data {
        &mut self.get_node(i).bound
    }

    #[inline]
    fn slack(&mut self, i: Link) -> Data {
        self.get_node(i).slack
    }
}

impl Dance for Problem {
    type I = INodes;
    type O = x::ONodes;

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
        x::commit(p, j, self);
    }

    #[inline]
    fn uncover(&mut self, i: Link) {
        x::uncover(i, self);
    }

    #[inline]
    fn uncommit(&mut self, p: Link, j: Link) {
        x::uncommit(p, j, self);
    }

    #[inline]
    fn hide(&mut self, p: Link) {
        x::hide(p, self);
    }

    #[inline]
    fn unhide(&mut self, p: Link) {
        x::unhide(p, self);
    }

    #[inline]
    fn branch_degree(&mut self, i: Link) -> Data {
        branch_degree(i, self)
    }
}

pub trait DanceM: Dance<I: ItemsM> {
    fn tweak(&mut self, x: Link, p: Link);
    fn untweak(&mut self, l: Count, unblock: bool);
    fn ft(&mut self) -> &mut Vec<Link>;

    #[inline]
    fn bound(&mut self, i: Link) -> &mut Data {
        self.items().bound(i)
    }

    #[inline]
    fn slack(&mut self, i: Link) -> Data {
        self.items().slack(i)
    }
}

impl DanceM for Problem {
    #[inline]
    fn tweak(&mut self, x: Link, p: Link) {
        tweak(x, p, self);
    }

    #[inline]
    fn untweak(&mut self, l: Count, unblock: bool) {
        untweak(l, unblock, self);
    }

    #[inline]
    fn ft(&mut self) -> &mut Vec<Link> {
        &mut self.ft
    }
}

impl Solve for Problem {
    #[inline]
    fn enter_level(&mut self, i: Link, l: Count, xl: Link) {
        enter_level(self, i, l, xl);
    }

    #[inline]
    fn prepare_to_branch(&mut self, i: Link, l: Count, xl: Link) {
        prepare_to_branch(self, i, l, xl);
    }

    #[inline]
    fn try_item(&mut self, i: Link, l: Count, xl: Link) -> bool {
        try_item(self, i, l, xl)
    }

    #[inline]
    fn try_again(&mut self, i: Link, l: Count, xl: &mut Link) -> bool {
        try_again(self, i, l, xl)
    }

    #[inline]
    fn restore_item(&mut self, i: Link, l: Count, xl: Link) {
        restore_item(self, i, l, xl);
    }
}

pub trait SolveM: Solve + DanceM<I: ItemsM> {}

impl SolveM for Problem {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_spec() {
        let spec_str = "
A B 2:3|C | X Y
A B X Y
A C X Y
C X
B X
C Y
";
        let spec = Spec::new(spec_str, false).unwrap();
        let problem = Problem::from_spec(&spec, OptOrder::Seq).unwrap();
        let ps = vec![(1, 1), (1, 1), (2, 3)];
        let items = INodes::new(ps, 2);
        assert_eq!(problem.items, items);
    }

    #[test]
    fn test_mc() {
        use crate::choose::*;
        use crate::{OptOrder, Rng, Solver};
        use core::iter::repeat_n;
        let ps = repeat_n((1, 1), 8)
            .chain(repeat_n((2, 2), 4))
            .chain(repeat_n((0, 2), 12));
        let items = INodes::new(ps, 0);

        let mut os: Vec<Vec<Count>> = Vec::new();
        for i in 0..2 {
            for j in 0..2 {
                os.push(vec![i, 8 + j, 12 + i + 1 - j, 15 + i + j]);
                os.push(vec![10 + i, 2 + j, 12 + i + 1 - j, 18 + i + j]);
                os.push(vec![4 + i, 8 + j, 21 + i + 1 - j, 18 + i + j]);
                os.push(vec![10 + i, 6 + j, 21 + i + 1 - j, 15 + i + j]);
            }
        }
        let opts =
            x::ONodes::new(24, 24, &os, OptOrder::Rnd(Rng::new(12345678)));

        let items_init = items.clone();
        let opts_init = opts.clone();
        let problem = Problem::new(items, opts);
        let mut solver = Solver::new(problem);
        let mut solutions: Vec<Vec<Data>> = Vec::new();
        let mut expected = vec![
            vec![0, 1, 5, 6, 8, 11, 14, 15],
            vec![0, 2, 5, 7, 9, 11, 12, 14],
            vec![0, 3, 6, 7, 8, 9, 13, 14],
            vec![1, 2, 4, 5, 10, 11, 12, 15],
            vec![1, 3, 4, 6, 8, 10, 13, 15],
            vec![2, 3, 4, 7, 9, 10, 12, 13],
        ];
        let mut i = 0;
        let mut chooser = mrv_chooser(prefer_any(), knuth_tiebreak());
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
