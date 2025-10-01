use crate::{Count, Dance, Data, Items, Link, Opts, Solve};

pub fn cover<D: Dance>(i: Link, dance: &mut D) {
    // TODO: increment updates
    let mut p = *dance.opts().dlink(i);
    while p != i {
        dance.hide(p);
        p = *dance.opts().dlink(p);
    }
    let l = *dance.items().llink(i);
    let r = *dance.items().rlink(i);
    *dance.items().rlink(l) = r;
    *dance.items().llink(r) = l;
}

pub fn commit<D: Dance>(_: Link, j: Link, dance: &mut D) {
    cover(j, dance);
}

pub fn uncover<D: Dance>(i: Link, dance: &mut D) {
    let l = *dance.items().llink(i);
    let r = *dance.items().rlink(i);
    *dance.items().rlink(l) = i;
    *dance.items().llink(r) = i;
    let mut p = *dance.opts().ulink(i);
    while p != i {
        dance.unhide(p);
        p = *dance.opts().ulink(p);
    }
}

pub fn uncommit<D: Dance>(_: Link, j: Link, dance: &mut D) {
    uncover(j, dance);
}

pub fn hide<D: Dance>(p: Link, dance: &mut D) {
    let mut q = p + 1;
    while q != p {
        let x = *dance.opts().top(q);
        let u = *dance.opts().ulink(q);
        let d = *dance.opts().dlink(q);
        if x <= 0 {
            q = u;
        } else {
            *dance.opts().dlink(u) = d;
            *dance.opts().ulink(d) = u;
            // TODO: increment updates
            *dance.opts().len(x as Link) -= 1;
            q += 1;
        }
    }
}

pub fn unhide<D: Dance>(p: Link, dance: &mut D) {
    let mut q = p - 1;
    while q != p {
        let x = *dance.opts().top(q);
        let u = *dance.opts().ulink(q);
        let d = *dance.opts().dlink(q);
        if x <= 0 {
            q = d;
        } else {
            *dance.opts().dlink(u) = q;
            *dance.opts().ulink(d) = q;
            *dance.opts().len(x as Link) += 1;
            q -= 1;
        }
    }
}

pub fn branch_degree<D: Dance>(i: Link, dance: &mut D) -> Data {
    *dance.opts().len(i) as Data
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
        let j = *solve.opts().top(p);
        if j <= 0 {
            p = *solve.opts().ulink(p);
        } else {
            solve.commit(p, j as Link);
            p += 1;
        }
    }
    true
}

pub fn try_again<S: Solve>(solve: &mut S, i: Link, xl: &mut Link) -> bool {
    let mut p = *xl - 1;
    while p != *xl {
        let j = *solve.opts().top(p);
        if j <= 0 {
            p = *solve.opts().dlink(p);
        } else {
            solve.uncommit(p, j as Link);
            p -= 1;
        }
    }
    *xl = *solve.opts().dlink(*xl);
    solve.try_item(i, *xl)
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
    secondary: Count,
}

impl INodes {
    pub fn new(np: Count, ns: Count) -> INodes {
        assert!((np as u64) < Data::MAX as u64);
        assert!((ns as u64) < Data::MAX as u64);
        let mut nodes = INodes {
            nodes: vec![Default::default(); (np + ns + 2) as usize],
            primary: np,
            secondary: ns,
        };
        nodes.init_links();
        nodes
    }

    fn get_node(&mut self, i: Link) -> &mut INode {
        if cfg!(feature = "unsafe-fast-index") {
            unsafe { self.nodes.get_unchecked_mut(i as usize) }
        } else {
            &mut self.nodes[i as usize]
        }
    }
}

impl Items for INodes {
    fn llink(&mut self, i: Link) -> &mut Link {
        &mut self.get_node(i).left
    }

    fn rlink(&mut self, i: Link) -> &mut Link {
        &mut self.get_node(i).right
    }

    fn primary(&self) -> Count {
        self.primary
    }

    fn secondary(&self) -> Count {
        self.secondary
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
    count: Count,
}

impl ONodes {
    pub fn new(
        n: Count, m: Count, l: Count,
        os: impl IntoIterator<Item = impl IntoIterator<Item = Count>>,
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
    type Spec = Count;

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

    fn set_data(&mut self, _: Link, s: Count) -> Link {
        s
    }
}

pub struct Problem {
    items: INodes,
    opts: ONodes,
}

impl Problem {
    pub fn new(items: INodes, opts: ONodes) -> Problem {
        Problem { items, opts }
    }
}

impl Dance for Problem {
    type I = INodes;
    type O = ONodes;

    fn items(&mut self) -> &mut Self::I {
        &mut self.items
    }

    fn opts(&mut self) -> &mut Self::O {
        &mut self.opts
    }

    fn cover(&mut self, i: Link) {
        cover(i, self);
    }

    fn commit(&mut self, p: Link, j: Link) {
        commit(p, j, self);
    }

    fn uncover(&mut self, i: Link) {
        uncover(i, self);
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
        branch_degree(i, self)
    }
}

impl Solve for Problem {
    fn enter_level(&mut self, _: Count) {}

    fn prepare_to_branch(&mut self, i: Link, l: Link, xl: Link) {
        prepare_to_branch(self, i, l, xl);
    }

    fn try_item(&mut self, i: Link, xl: Link) -> bool {
        try_item(self, i, xl)
    }

    fn try_again(&mut self, i: Link, xl: &mut Link) -> bool {
        try_again(self, i, xl)
    }

    fn restore_item(&mut self, i: Link) {
        restore_item(self, i);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_init() {
        let items = INodes::new(3, 2);
        let inodes = vec![
            INode { left: 3, right: 1 },
            INode { left: 0, right: 2 },
            INode { left: 1, right: 3 },
            INode { left: 2, right: 0 },
            INode { left: 6, right: 5 },
            INode { left: 4, right: 6 },
            INode { left: 5, right: 4 },
        ];
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
        let opts = ONodes::new(5, 5, 14, os);
        let onodes = vec![
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
        ];
        assert_eq!(opts.nodes, onodes, "incorrect options");
    }

    #[test]
    fn test_xc() {
        use crate::Solver;
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
        let items_init = items.clone();
        let opts_init = opts.clone();
        let problem = Problem::new(items, opts);
        let mut solver = Solver::new(problem);
        let mut solutions: Vec<Vec<isize>> = Vec::new();
        let mut expected = vec![vec![0, 3, 4]];
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
