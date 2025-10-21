use anyhow::{Result, anyhow, bail};

use crate::x;
use crate::{Count, Dance, Data, Link, OptOrder, Opts, Solve, Spec};

pub fn commit<D: DanceC<O: OptsC>>(p: Link, j: Link, dance: &mut D) {
    if *dance.color(p) == 0 {
        dance.cover(j);
    }
    if *dance.color(p) > 0 {
        dance.purify(p);
    }
}

pub fn uncommit<D: DanceC<O: OptsC>>(p: Link, j: Link, dance: &mut D) {
    if *dance.color(p) == 0 {
        dance.uncover(j)
    }
    if *dance.color(p) > 0 {
        dance.unpurify(p);
    }
}

pub fn hide<D: DanceC<O: OptsC>>(p: Link, dance: &mut D) {
    let mut q = p + 1;
    while q != p {
        let x = *dance.top(q);
        let u = *dance.ulink(q);
        let d = *dance.dlink(q);
        if x <= 0 {
            q = u;
        } else {
            if *dance.color(q) >= 0 {
                *dance.dlink(u) = d;
                *dance.ulink(d) = u;
                *dance.len(x as Link) -= 1;
                *dance.updates() += 1;
            }
            q += 1;
        }
    }
}

pub fn unhide<D: DanceC<O: OptsC>>(p: Link, dance: &mut D) {
    let mut q = p - 1;
    while q != p {
        let x = *dance.top(q);
        let u = *dance.ulink(q);
        let d = *dance.dlink(q);
        if x <= 0 {
            q = d;
        } else {
            if *dance.color(q) >= 0 {
                *dance.dlink(u) = q;
                *dance.ulink(d) = q;
                *dance.len(x as Link) += 1;
            }
            q -= 1;
        }
    }
}

pub fn purify<D: DanceC<O: OptsC>>(p: Link, dance: &mut D) {
    let c = *dance.color(p);
    let i = *dance.top(p) as Link;
    // TODO is this needed?
    // *dance.color(i) = c;
    let mut q = *dance.dlink(i);
    while q != i {
        if *dance.color(q) == c {
            *dance.color(q) = -1;
        } else {
            dance.hide(q)
        }
        q = *dance.dlink(q);
    }
}

pub fn unpurify<D: DanceC<O: OptsC>>(p: Link, dance: &mut D) {
    let c = *dance.color(p);
    let i = *dance.top(p) as Link;
    let mut q = *dance.ulink(i);
    while q != i {
        if *dance.color(q) < 0 {
            *dance.color(q) = c;
        } else {
            dance.unhide(q);
        }
        q = *dance.ulink(q);
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
}

impl ONodes {
    pub fn new(
        n: Count, np: Count, os: &[Vec<(Count, Data)>], order: OptOrder,
    ) -> ONodes {
        // TODO: ensure primary have color 0
        let mut nodes =
            ONodes { nodes: vec![Default::default(); (n + 2) as usize] };
        nodes.init_links(n, np, order, os);
        nodes
    }

    pub fn from_spec(spec: &Spec, names: &[String]) -> Result<ONodes> {
        use std::collections::{HashMap, HashSet};
        let mut idx = HashMap::new();
        for (i, name) in names.iter().enumerate() {
            idx.insert(name.as_str(), i);
        }
        let mut os = Vec::new();
        for opt in &spec.opts {
            let mut is = Vec::new();
            let mut used = HashSet::new();
            for itm in opt {
                let data = itm.split(':').collect::<Vec<_>>();
                let (name, color) = if data.len() > 2 {
                    bail!("Too many ':' separators");
                } else if data.len() == 2 {
                    if data[1].len() != 1 {
                        bail!("Color must be a single character");
                    }
                    (data[0], data[1].chars().nth(0).unwrap())
                } else {
                    (data[0], ' ')
                };
                let i =
                    *idx.get(name).ok_or_else(|| anyhow!("Invalid item"))?;
                if !used.insert(itm) {
                    bail!("Duplicate items in option");
                }
                if i < spec.primary.len() && color != ' ' {
                    bail!("Color on primary item");
                }
                let color = if color == ' ' { 0 } else { color as Data };
                is.push((i as Count, color));
            }
            os.push(is);
        }
        let n = (spec.primary.len() + spec.secondary.len()) as Count;
        let opts =
            ONodes::new(n, spec.primary.len() as Count, &os, OptOrder::Seq);
        Ok(opts)
    }

    #[inline]
    fn get_node(&mut self, i: Link) -> &mut ONode {
        if cfg!(feature = "unsafe-fast-index") {
            unsafe { self.nodes.get_unchecked_mut(i as usize) }
        } else {
            &mut self.nodes[i as usize]
        }
    }
}

pub fn make_problem(
    np: Count, ns: Count, os: &[Vec<(Count, Data)>], order: OptOrder,
) -> Problem {
    Problem::new(x::INodes::new(np, ns), ONodes::new(np + ns, np, os, order))
}

#[derive(Debug, Eq, PartialEq)]
pub struct Problem {
    items: x::INodes,
    opts: ONodes,
    updates: isize,
}

impl Problem {
    pub fn new(items: x::INodes, opts: ONodes) -> Problem {
        Problem { items, opts, updates: 0 }
    }

    pub fn from_spec(spec: &Spec) -> Result<Problem> {
        let (items, names) = x::INodes::from_spec(spec)?;
        let opts = ONodes::from_spec(spec, &names)?;
        Ok(Problem::new(items, opts))
    }
}

impl Opts for ONodes {
    type Spec = (Count, Data);

    #[inline]
    fn len(&mut self, i: Link) -> &mut Data {
        &mut self.get_node(i).hdr_info
    }

    #[inline]
    fn top(&mut self, i: Link) -> &mut Data {
        &mut self.get_node(i).hdr_info
    }

    #[inline]
    fn ulink(&mut self, i: Link) -> &mut Link {
        &mut self.get_node(i).up
    }

    #[inline]
    fn dlink(&mut self, i: Link) -> &mut Link {
        &mut self.get_node(i).down
    }

    fn set_data(&mut self, pk: Link, s: (Count, Data)) -> Link {
        self.nodes.push(Default::default());
        *self.color(pk) = s.1;
        s.0
    }

    fn get_spec_item(s: Self::Spec) -> Link {
        s.0
    }
}

pub trait OptsC: Opts {
    fn color(&mut self, i: Link) -> &mut Data;
}

impl OptsC for ONodes {
    #[inline]
    fn color(&mut self, i: Link) -> &mut Data {
        &mut self.get_node(i).color
    }
}

impl Dance for Problem {
    type I = x::INodes;
    type O = ONodes;

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
        commit(p, j, self);
    }

    #[inline]
    fn uncover(&mut self, i: Link) {
        x::uncover(i, self);
    }

    #[inline]
    fn uncommit(&mut self, p: Link, j: Link) {
        uncommit(p, j, self);
    }

    #[inline]
    fn hide(&mut self, p: Link) {
        hide(p, self);
    }

    #[inline]
    fn unhide(&mut self, p: Link) {
        unhide(p, self);
    }

    #[inline]
    fn branch_degree(&mut self, i: Link) -> Data {
        x::branch_degree(i, self)
    }
}

pub trait DanceC: Dance<O: OptsC> {
    fn purify(&mut self, p: Link);
    fn unpurify(&mut self, p: Link);

    #[inline]
    fn color(&mut self, i: Link) -> &mut Data {
        self.opts().color(i)
    }
}

impl DanceC for Problem {
    #[inline]
    fn purify(&mut self, p: Link) {
        purify(p, self);
    }

    #[inline]
    fn unpurify(&mut self, p: Link) {
        unpurify(p, self);
    }
}

impl Solve for Problem {
    fn enter_level(&mut self, _: Link, _: Count, _: Link) {}

    #[inline]
    fn prepare_to_branch(&mut self, i: Link, l: Count, xl: Link) {
        x::prepare_to_branch(self, i, l, xl);
    }

    #[inline]
    fn try_item(&mut self, i: Link, _: Count, xl: Link) -> bool {
        x::try_item(self, i, xl)
    }

    #[inline]
    fn try_again(&mut self, i: Link, l: Count, xl: &mut Link) -> bool {
        x::try_again(self, i, l, xl)
    }

    #[inline]
    fn restore_item(&mut self, i: Link, _: Count, _: Link) {
        x::restore_item(self, i);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opt_init() {
        let os: Vec<Vec<(Count, Data)>> = vec![
            vec![(0, 0), (1, 0), (3, 0), (4, 65)],
            vec![(0, 0), (2, 0), (3, 65), (4, 0)],
            vec![(0, 0), (3, 66)],
            vec![(1, 0), (3, 65)],
            vec![(2, 0), (4, 66)],
        ];
        let opts = ONodes::new(5, 3, &os, OptOrder::Seq);
        let onodes = onodes_data();
        assert_eq!(opts.nodes, onodes, "incorrect options");
    }

    #[test]
    fn test_from_spec() {
        let spec_str = "
p q r | x y
p q x y:A
p r x:A y
p x:B
q x:A
r y:B
";
        let spec = Spec::new(spec_str, false).unwrap();
        let problem = Problem::from_spec(&spec).unwrap();
        assert_eq!(problem.opts.nodes, onodes_data());
    }

    #[test]
    fn test_xcc() {
        use crate::Solver;
        use crate::choose::*;
        let items = x::INodes::new(3, 2);
        let os: Vec<Vec<(Count, Data)>> = vec![
            vec![(0, 0), (1, 0), (3, 0), (4, 65)],
            vec![(0, 0), (2, 0), (3, 65), (4, 0)],
            vec![(0, 0), (3, 66)],
            vec![(1, 0), (3, 65)],
            vec![(2, 0), (4, 66)],
        ];
        let opts = ONodes::new(5, 3, &os, OptOrder::Seq);
        let items_init = items.clone();
        let opts_init = opts.clone();
        let problem = Problem::new(items, opts);
        let mut solver = Solver::new(problem);
        let mut solutions: Vec<Vec<Data>> = Vec::new();
        let mut expected = vec![vec![1, 3]];
        let mut i = 0;
        let mut chooser = mrv_chooser(prefer_any(), rnd_tiebreak(12345678));
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

    fn onodes_data() -> Vec<ONode> {
        vec![
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
            ONode { hdr_info: 5, up: 5, down: 15, color: 65 },
            ONode { hdr_info: -1, up: 7, down: 15, color: 0 },
            ONode { hdr_info: 1, up: 7, down: 17, color: 0 },
            ONode { hdr_info: 3, up: 3, down: 23, color: 0 },
            ONode { hdr_info: 4, up: 9, down: 18, color: 65 },
            ONode { hdr_info: 5, up: 10, down: 24, color: 0 },
            ONode { hdr_info: -2, up: 12, down: 18, color: 0 },
            ONode { hdr_info: 1, up: 12, down: 1, color: 0 },
            ONode { hdr_info: 4, up: 14, down: 21, color: 66 },
            ONode { hdr_info: -3, up: 17, down: 21, color: 0 },
            ONode { hdr_info: 2, up: 8, down: 2, color: 0 },
            ONode { hdr_info: 4, up: 18, down: 4, color: 65 },
            ONode { hdr_info: -4, up: 20, down: 24, color: 0 },
            ONode { hdr_info: 3, up: 13, down: 3, color: 0 },
            ONode { hdr_info: 5, up: 15, down: 5, color: 66 },
            ONode { hdr_info: -5, up: 23, down: 0, color: 0 },
        ]
    }
}
