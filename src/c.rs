use crate::x;
use crate::{Count, Dance, Data, Link, Opts, Solve};

pub trait DanceC: Dance {
    fn purify(&mut self, p: Link);
    fn unpurify(&mut self, p: Link);
}

pub trait OptsC: Opts {
    fn color(&mut self, i: Link) -> &mut Data;
}

pub fn commit<D: DanceC<O: OptsC>>(p: Link, j: Link, dance: &mut D) {
    if *dance.opts().color(p) == 0 {
        dance.cover(j);
    }
    if *dance.opts().color(p) > 0 {
        dance.purify(p);
    }
}

pub fn uncommit<D: DanceC<O: OptsC>>(p: Link, j: Link, dance: &mut D) {
    if *dance.opts().color(p) == 0 {
        dance.uncover(j)
    }
    if *dance.opts().color(p) > 0 {
        dance.unpurify(p);
    }
}

pub fn hide<D: DanceC<O: OptsC>>(p: Link, dance: &mut D) {
    let mut q = p + 1;
    while q != p {
        let x = *dance.opts().top(q);
        let u = *dance.opts().ulink(q);
        let d = *dance.opts().dlink(q);
        if x <= 0 {
            q = u;
        } else {
            if *dance.opts().color(q) >= 0 {
                *dance.opts().dlink(u) = d;
                *dance.opts().ulink(d) = u;
                // TODO self.updates += 1;
                *dance.opts().len(x as Link) -= 1;
            }
            q += 1;
        }
    }
}

pub fn unhide<D: DanceC<O: OptsC>>(p: Link, dance: &mut D) {
    let mut q = p - 1;
    while q != p {
        let x = *dance.opts().top(q);
        let u = *dance.opts().ulink(q);
        let d = *dance.opts().dlink(q);
        if x <= 0 {
            q = d;
        } else {
            if *dance.opts().color(q) >= 0 {
                *dance.opts().dlink(u) = q;
                *dance.opts().ulink(d) = q;
                *dance.opts().len(x as Link) += 1;
            }
            q -= 1;
        }
    }
}

pub fn purify<D: DanceC<O: OptsC>>(p: Link, dance: &mut D) {
    let c = *dance.opts().color(p);
    let i = *dance.opts().top(p) as Link;
    // TODO is this needed?
    // *dance.opts().color(i) = c;
    let mut q = *dance.opts().dlink(i);
    while q != i {
        if *dance.opts().color(q) == c {
            *dance.opts().color(q) = -1;
        } else {
            dance.hide(q)
        }
        q = *dance.opts().dlink(q);
    }
}

pub fn unpurify<D: DanceC<O: OptsC>>(p: Link, dance: &mut D) {
    let c = *dance.opts().color(p);
    let i = *dance.opts().top(p) as Link;
    let mut q = *dance.opts().ulink(i);
    while q != i {
        if *dance.opts().color(q) < 0 {
            *dance.opts().color(q) = c;
        } else {
            dance.unhide(q);
        }
        q = *dance.opts().ulink(q);
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ONode {
    hdr_info: Data,
    up: Link,
    down: Link,
    color: Data,
}

#[derive(Clone, Default, Debug, Eq, PartialEq)]
pub struct ONodes {
    nodes: Vec<ONode>,
    count: Count,
}

impl ONodes {
    pub fn new(
        n: Count, m: Count, l: Count,
        os: impl IntoIterator<Item = impl IntoIterator<Item = (Count, Data)>>,
    ) -> ONodes {
        assert!((m as u64) < Data::MAX as u64);
        let mut nodes = ONodes {
            nodes: vec![Default::default(); (l + m + n + 2) as usize],
            count: m,
        };
        nodes.init_links(n, os);
        nodes
    }

    fn get_node(&mut self, i: Link) -> &mut ONode {
        if cfg!(feature = "unsafe-fast-index") {
            unsafe { self.nodes.get_unchecked_mut(i as usize) }
        } else {
            &mut self.nodes[i as usize]
        }
    }
}

impl Opts for ONodes {
    type Spec = (Count, Data);

    fn len(&mut self, i: Link) -> &mut Data {
        &mut self.get_node(i).hdr_info
    }

    fn top(&mut self, i: Link) -> &mut Data {
        &mut self.get_node(i).hdr_info
    }

    fn ulink(&mut self, i: Link) -> &mut Link {
        &mut self.get_node(i).up
    }

    fn dlink(&mut self, i: Link) -> &mut Link {
        &mut self.get_node(i).down
    }

    fn set_data(&mut self, pk: Link, s: (Count, Data)) -> Link {
        *self.color(pk) = s.1;
        s.0
    }
}

impl OptsC for ONodes {
    fn color(&mut self, i: Link) -> &mut Data {
        &mut self.get_node(i).color
    }
}

pub struct Problem {
    items: x::INodes,
    opts: ONodes,
}

impl Problem {
    pub fn new(items: x::INodes, opts: ONodes) -> Problem {
        Problem { items, opts }
    }
}

impl Dance for Problem {
    type I = x::INodes;
    type O = ONodes;

    fn items(&mut self) -> &mut Self::I {
        &mut self.items
    }

    fn opts(&mut self) -> &mut Self::O {
        &mut self.opts
    }

    fn cover(&mut self, i: Link) {
        x::cover(i, self);
    }

    fn commit(&mut self, p: Link, j: Link) {
        commit(p, j, self);
    }

    fn uncover(&mut self, i: Link) {
        x::uncover(i, self);
    }

    fn uncommit(&mut self, p: Link, j: Link) {
        uncommit(p, j, self);
    }

    fn hide(&mut self, p: Link) {
        hide(p, self);
    }

    fn unhide(&mut self, p: Link) {
        unhide(p, self);
    }

    fn branch_degree(&mut self, i: Link) -> Data {
        x::branch_degree(i, self)
    }
}

impl DanceC for Problem {
    fn purify(&mut self, p: Link) {
        purify(p, self);
    }

    fn unpurify(&mut self, p: Link) {
        unpurify(p, self);
    }
}

impl Solve for Problem {
    fn enter_level(&mut self, _: Count) {}

    fn prepare_to_branch(&mut self, i: Link, l: Link, xl: Link) {
        x::prepare_to_branch(self, i, l, xl);
    }

    fn try_item(&mut self, i: Link, xl: Link) -> bool {
        x::try_item(self, i, xl)
    }

    fn try_again(&mut self, i: Link, xl: &mut Link) -> bool {
        x::try_again(self, i, xl)
    }

    fn restore_item(&mut self, i: Link) {
        x::restore_item(self, i);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opt_init() {
        let opt_spec: Vec<Vec<(Count, Data)>> = vec![
            vec![(0, 0), (1, 0), (3, 0), (4, 1)],
            vec![(0, 0), (2, 0), (3, 1), (4, 0)],
            vec![(0, 0), (3, 2)],
            vec![(1, 0), (3, 1)],
            vec![(2, 0), (4, 2)],
        ];
        let opts = ONodes::new(5, 5, 14, opt_spec);
        let onodes = vec![
            ONode { hdr_info: 0, up: 0, down: 0, color: 0 },
            ONode { hdr_info: 3, up: 17, down: 7, color: 0 },
            ONode { hdr_info: 2, up: 20, down: 8, color: 0 },
            ONode { hdr_info: 2, up: 23, down: 13, color: 0 },
            ONode { hdr_info: 4, up: 21, down: 9, color: 0 },
            ONode { hdr_info: 3, up: 24, down: 10, color: 0 },
            ONode { hdr_info: 0, up: 0, down: 10, color: 0 },
            ONode { hdr_info: 1, up: 1, down: 12, color: 0 },
            ONode { hdr_info: 2, up: 2, down: 20, color: 0 },
            ONode { hdr_info: 4, up: 4, down: 14, color: 0 },
            ONode { hdr_info: 5, up: 5, down: 15, color: 1 },
            ONode { hdr_info: -1, up: 7, down: 15, color: 0 },
            ONode { hdr_info: 1, up: 7, down: 17, color: 0 },
            ONode { hdr_info: 3, up: 3, down: 23, color: 0 },
            ONode { hdr_info: 4, up: 9, down: 18, color: 1 },
            ONode { hdr_info: 5, up: 10, down: 24, color: 0 },
            ONode { hdr_info: -2, up: 12, down: 18, color: 0 },
            ONode { hdr_info: 1, up: 12, down: 1, color: 0 },
            ONode { hdr_info: 4, up: 14, down: 21, color: 2 },
            ONode { hdr_info: -3, up: 17, down: 21, color: 0 },
            ONode { hdr_info: 2, up: 8, down: 2, color: 0 },
            ONode { hdr_info: 4, up: 18, down: 4, color: 1 },
            ONode { hdr_info: -4, up: 20, down: 24, color: 0 },
            ONode { hdr_info: 3, up: 13, down: 3, color: 0 },
            ONode { hdr_info: 5, up: 15, down: 5, color: 2 },
            ONode { hdr_info: -5, up: 23, down: 0, color: 0 },
        ];
        assert_eq!(opts.nodes, onodes, "incorrect options");
    }

    #[test]
    fn test_xcc() {
        use crate::Solver;
        let items = x::INodes::new(3, 2);
        let os: Vec<Vec<(Count, Data)>> = vec![
            vec![(0, 0), (1, 0), (3, 0), (4, 1)],
            vec![(0, 0), (2, 0), (3, 1), (4, 0)],
            vec![(0, 0), (3, 2)],
            vec![(1, 0), (3, 1)],
            vec![(2, 0), (4, 2)],
        ];
        let opts = ONodes::new(5, 5, 14, os);
        let items_init = items.clone();
        let opts_init = opts.clone();
        let problem = Problem::new(items, opts);
        let mut solver = Solver::new(problem);
        let mut solutions: Vec<Vec<isize>> = Vec::new();
        let mut expected = vec![vec![1, 3]];
        let mut i = 0;
        while solver.next_solution() {
            assert!(i <= expected.len(), "too many solutions");
            solver.find_options();
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
