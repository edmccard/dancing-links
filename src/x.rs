use anyhow::{Result, anyhow, bail};

use crate::{Count, Dance, Data, Items, Link, OptOrder, Opts, Solve, Spec};

pub fn cover<D: Dance>(i: Link, dance: &mut D) {
    // TODO: increment updates
    let mut p = *dance.dlink(i);
    while p != i {
        dance.hide(p);
        p = *dance.dlink(p);
    }
    let l = *dance.llink(i);
    let r = *dance.rlink(i);
    *dance.rlink(l) = r;
    *dance.llink(r) = l;
    *dance.updates() += 1;
}

pub fn commit<D: Dance>(_: Link, j: Link, dance: &mut D) {
    cover(j, dance);
}

pub fn uncover<D: Dance>(i: Link, dance: &mut D) {
    let l = *dance.llink(i);
    let r = *dance.rlink(i);
    *dance.rlink(l) = i;
    *dance.llink(r) = i;
    let mut p = *dance.ulink(i);
    while p != i {
        dance.unhide(p);
        p = *dance.ulink(p);
    }
}

pub fn uncommit<D: Dance>(_: Link, j: Link, dance: &mut D) {
    uncover(j, dance);
}

pub fn hide<D: Dance>(p: Link, dance: &mut D) {
    let mut q = p + 1;
    while q != p {
        let x = *dance.top(q);
        let u = *dance.ulink(q);
        let d = *dance.dlink(q);
        if x <= 0 {
            q = u;
        } else {
            *dance.dlink(u) = d;
            *dance.ulink(d) = u;
            *dance.len(x as Link) -= 1;
            q += 1;
            *dance.updates() += 1;
        }
    }
}

pub fn unhide<D: Dance>(p: Link, dance: &mut D) {
    let mut q = p - 1;
    while q != p {
        let x = *dance.top(q);
        let u = *dance.ulink(q);
        let d = *dance.dlink(q);
        if x <= 0 {
            q = d;
        } else {
            *dance.dlink(u) = q;
            *dance.ulink(d) = q;
            *dance.len(x as Link) += 1;
            q -= 1;
        }
    }
}

pub fn branch_degree<D: Dance>(i: Link, dance: &mut D) -> Data {
    *dance.len(i) as Data
}

pub fn prepare_to_branch<S: Solve>(solve: &mut S, i: Link, _: Link, _: Link) {
    solve.cover(i);
}

pub fn try_item<S: Solve>(solve: &mut S, i: Link, xl: Link) -> bool {
    if xl == i {
        return false;
    }
    let mut p = xl + 1;
    while p != xl {
        let j = *solve.top(p);
        if j <= 0 {
            p = *solve.ulink(p);
        } else {
            solve.commit(p, j as Link);
            p += 1;
        }
    }
    true
}

pub fn try_again<S: Solve>(
    solve: &mut S, i: Link, l: Count, xl: &mut Link,
) -> bool {
    let mut p = *xl - 1;
    while p != *xl {
        let j = *solve.top(p);
        if j <= 0 {
            p = *solve.dlink(p);
        } else {
            solve.uncommit(p, j as Link);
            p -= 1;
        }
    }
    *xl = *solve.dlink(*xl);
    let again = solve.try_item(i, l, *xl);
    if !again {
        solve.restore_item(i, l, *xl);
    }
    again
}

pub fn restore_item<S: Solve>(solve: &mut S, i: Link) {
    solve.uncover(i);
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct INode {
    left: Link,
    right: Link,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct INodes {
    nodes: Vec<INode>,
    primary: Count,
    len: Count,
}

impl INodes {
    pub fn new(np: Count, ns: Count) -> INodes {
        assert!((np as u64) < Data::MAX as u64);
        assert!((ns as u64) < Data::MAX as u64);
        let mut nodes = INodes {
            nodes: vec![Default::default(); (np + ns + 2) as usize],
            primary: np,
            len: np + ns,
        };
        nodes.init_links();
        nodes
    }

    pub fn from_spec(spec: &Spec) -> Result<(INodes, Vec<String>)> {
        use std::collections::HashSet;
        let np = spec.primary.len() as Count;
        let ns = spec.secondary.len() as Count;
        let mut names = spec.primary.clone();
        names.extend(spec.secondary.clone());
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
        Ok((INodes::new(np, ns), names))
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

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ONode {
    hdr_info: Data,
    up: Link,
    down: Link,
}

#[derive(Clone, Default, Debug, Eq, PartialEq)]
pub struct ONodes {
    nodes: Vec<ONode>,
}

impl ONodes {
    pub fn new(
        n: Count, np: Count, os: &[Vec<Count>], order: OptOrder,
    ) -> ONodes {
        let mut onodes =
            ONodes { nodes: vec![Default::default(); (n + 2) as usize] };
        onodes.init_links(n, np, order, os);
        onodes
    }

    pub fn from_spec(
        spec: &Spec, names: &[String], order: OptOrder,
    ) -> Result<ONodes> {
        use std::collections::{HashMap, HashSet};
        let mut idx = HashMap::new();
        for (i, name) in names.iter().enumerate() {
            idx.insert(name, i);
        }
        let mut os = Vec::new();
        for opt in &spec.opts {
            let mut is = Vec::new();
            let mut used = HashSet::new();
            for itm in opt {
                let i = idx.get(itm).ok_or_else(|| anyhow!("Invalid item"))?;
                if !used.insert(itm) {
                    bail!("Duplicate items in option");
                }
                is.push(*i as Count);
            }
            os.push(is);
        }
        let n = (spec.primary.len() + spec.secondary.len()) as Count;
        let opts = ONodes::new(n, spec.primary.len() as Count, &os, order);
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
    np: Count, ns: Count, os: &[Vec<Count>], order: OptOrder,
) -> Problem {
    Problem::new(INodes::new(np, ns), ONodes::new(np + ns, np, os, order))
}

#[derive(Debug, Eq, PartialEq)]
pub struct Problem {
    items: INodes,
    opts: ONodes,
    updates: isize,
}

impl Problem {
    pub fn new(items: INodes, opts: ONodes) -> Problem {
        Problem { items, opts, updates: 0 }
    }

    pub fn from_spec(spec: &Spec, order: OptOrder) -> Result<Problem> {
        let (items, names) = INodes::from_spec(spec)?;
        let opts = ONodes::from_spec(spec, &names, order)?;
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

impl Opts for ONodes {
    type Spec = Count;

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

    fn set_data(&mut self, _pk: Link, s: Count) -> Link {
        self.nodes.push(Default::default());
        s
    }

    fn get_spec_item(s: Self::Spec) -> Link {
        s
    }
}

impl Dance for Problem {
    type I = INodes;
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
        cover(i, self);
    }

    #[inline]
    fn commit(&mut self, p: Link, j: Link) {
        commit(p, j, self);
    }

    #[inline]
    fn uncover(&mut self, i: Link) {
        uncover(i, self);
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
        branch_degree(i, self)
    }
}

impl Solve for Problem {
    fn enter_level(&mut self, _: Link, _: Count, _: Link) {}

    #[inline]
    fn prepare_to_branch(&mut self, i: Link, l: Count, xl: Link) {
        prepare_to_branch(self, i, l, xl);
    }

    #[inline]
    fn try_item(&mut self, i: Link, _: Count, xl: Link) -> bool {
        try_item(self, i, xl)
    }

    #[inline]
    fn try_again(&mut self, i: Link, l: Count, xl: &mut Link) -> bool {
        try_again(self, i, l, xl)
    }

    #[inline]
    fn restore_item(&mut self, i: Link, _: Count, _: Link) {
        restore_item(self, i);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_init() {
        let items = INodes::new(3, 2);
        let inodes = inodes_data();
        assert_eq!(items.nodes, inodes, "incorrect items");
    }

    #[test]
    fn test_opt_init() {
        let os: Vec<Vec<Count>> = vec![
            vec![0, 1, 3, 4],
            vec![0, 2, 3, 4],
            vec![0, 3],
            vec![1, 3],
            vec![2, 4],
        ];
        let opts = ONodes::new(5, 3, &os, OptOrder::Seq);
        let onodes = onodes_data();
        assert_eq!(opts.nodes, onodes, "incorrect options");
    }

    #[test]
    fn test_from_spec() {
        let spec_str = "
| This is a comment
p q r | x y

p q x y
p r x y
| Another comment
p x
q x
r y";
        let spec = Spec::new(spec_str, false).unwrap();
        let problem = Problem::from_spec(&spec, OptOrder::Seq).unwrap();
        assert_eq!(problem.items.nodes, inodes_data());
        assert_eq!(problem.opts.nodes, onodes_data());
    }

    #[test]
    fn test_xc() {
        use crate::Solver;
        use crate::choose::*;
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
        let items_init = items.clone();
        let opts_init = opts.clone();
        let problem = Problem::new(items, opts);
        let mut solver = Solver::new(problem);
        let mut solutions: Vec<Vec<Data>> = Vec::new();
        let mut expected = vec![vec![0, 3, 4]];
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

    pub(crate) fn inodes_data() -> Vec<INode> {
        vec![
            INode { left: 3, right: 1 },
            INode { left: 0, right: 2 },
            INode { left: 1, right: 3 },
            INode { left: 2, right: 0 },
            INode { left: 6, right: 5 },
            INode { left: 4, right: 6 },
            INode { left: 5, right: 4 },
        ]
    }

    fn onodes_data() -> Vec<ONode> {
        vec![
            ONode { hdr_info: 0, up: 0, down: 0 },
            ONode { hdr_info: 3, up: 17, down: 7 },
            ONode { hdr_info: 2, up: 20, down: 8 },
            ONode { hdr_info: 2, up: 23, down: 13 },
            ONode { hdr_info: 4, up: 21, down: 9 },
            ONode { hdr_info: 3, up: 24, down: 10 },
            ONode { hdr_info: 0, up: 0, down: 10 },
            ONode { hdr_info: 1, up: 1, down: 12 },
            ONode { hdr_info: 2, up: 2, down: 20 },
            ONode { hdr_info: 4, up: 4, down: 14 },
            ONode { hdr_info: 5, up: 5, down: 15 },
            ONode { hdr_info: -1, up: 7, down: 15 },
            ONode { hdr_info: 1, up: 7, down: 17 },
            ONode { hdr_info: 3, up: 3, down: 23 },
            ONode { hdr_info: 4, up: 9, down: 18 },
            ONode { hdr_info: 5, up: 10, down: 24 },
            ONode { hdr_info: -2, up: 12, down: 18 },
            ONode { hdr_info: 1, up: 12, down: 1 },
            ONode { hdr_info: 4, up: 14, down: 21 },
            ONode { hdr_info: -3, up: 17, down: 21 },
            ONode { hdr_info: 2, up: 8, down: 2 },
            ONode { hdr_info: 4, up: 18, down: 4 },
            ONode { hdr_info: -4, up: 20, down: 24 },
            ONode { hdr_info: 3, up: 13, down: 3 },
            ONode { hdr_info: 5, up: 15, down: 5 },
            ONode { hdr_info: -5, up: 23, down: 0 },
        ]
    }
}
