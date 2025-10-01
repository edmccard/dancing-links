#![allow(clippy::unnecessary_cast)]

pub mod c;
pub mod m;
pub mod x;

pub type Link = usize;
pub type Count = Link;
pub type Data = isize;

const _: () = {
    assert!(Link::MAX as u128 <= u64::MAX as u128);
    assert!(Count::MAX as u128 <= u64::MAX as u128);
    assert!(Data::MAX as u128 <= u64::MAX as u128);
    assert!(Data::MAX as u128 <= Count::MAX as u128);
    assert!(Data::MAX as u128 <= Link::MAX as u128);
};

pub trait Dance {
    type I: Items;
    type O: Opts;

    fn items(&mut self) -> &mut Self::I;
    fn opts(&mut self) -> &mut Self::O;

    fn cover(&mut self, i: Link);
    fn commit(&mut self, p: Link, j: Link);
    fn uncover(&mut self, i: Link);
    fn uncommit(&mut self, p: Link, j: Link);
    fn hide(&mut self, p: Link);
    fn unhide(&mut self, p: Link);
    fn branch_degree(&mut self, i: Link) -> Data;
}

#[allow(clippy::len_without_is_empty)]
pub trait Items {
    fn llink(&mut self, i: Link) -> &mut Link;
    fn rlink(&mut self, i: Link) -> &mut Link;

    fn primary(&self) -> Count;
    fn secondary(&self) -> Count;

    fn len(&self) -> Count {
        self.primary() + self.secondary()
    }

    fn init_links(&mut self) {
        let n1 = self.primary();
        let n = self.len();
        for i in (1 as Link)..=n {
            *self.llink(i) = i - 1;
            *self.rlink(i - 1) = i;
        }
        *self.llink(n + 1) = n;
        *self.rlink(n) = n + 1;
        *self.llink(n1 + 1) = n + 1;
        *self.rlink(n + 1) = n1 + 1;
        *self.llink(0) = n1;
        *self.rlink(n1) = 0;
    }
}

pub trait Opts {
    type Spec;

    fn len(&mut self, i: Link) -> &mut Data;
    fn top(&mut self, i: Link) -> &mut Data;
    fn ulink(&mut self, i: Link) -> &mut Link;
    fn dlink(&mut self, i: Link) -> &mut Link;

    fn set_data(&mut self, pk: Link, s: Self::Spec) -> Link;

    fn init_links(
        &mut self, n: Count,
        os: impl IntoIterator<Item = impl IntoIterator<Item = Self::Spec>>,
    ) {
        for i in (1 as Link)..=n {
            *self.ulink(i) = i;
            *self.dlink(i) = i;
        }
        let mut m: Data = 0;
        let mut p: Link = n + 1;
        for opt in os.into_iter() {
            let mut k = 0;
            for node in opt.into_iter() {
                k += 1;
                // Internal item indexes are 1-based.
                let i = self.set_data(p + k, node) + 1;
                // TODO: i <= Data::MAX

                *self.len(i) += 1;
                let q = *self.ulink(i);
                let qd = *self.dlink(q);
                *self.ulink(p + k) = q;
                *self.dlink(p + k) = qd;
                *self.dlink(q) = p + k;
                *self.ulink(qd) = p + k;
                *self.top(p + k) = i as Data;
            }
            m += 1;
            *self.dlink(p) = p + k;
            p = p + k + 1;
            *self.top(p) = -m;
            *self.ulink(p) = p - k;
        }
    }
}

pub trait Solve: Dance {
    fn enter_level(&mut self, i: Link, l: Count, xl: Link);
    fn prepare_to_branch(&mut self, i: Link, l: Count, xl: Link);
    fn try_item(&mut self, i: Link, l: Count, xl: Link) -> bool;
    fn try_again(&mut self, i: Link, l: Count, xl: &mut Link) -> bool;
    fn restore_item(&mut self, i: Link, l: Count, xl: Link);
}

pub struct Solver<P: Solve> {
    problem: P,
    x: Vec<Link>,
    o: Vec<isize>,
    l: Count,
    i: Link,
    restart: bool,
}

impl<P: Solve> Solver<P> {
    pub fn new(problem: P) -> Solver<P> {
        Solver {
            problem,
            x: Vec::new(),
            o: Vec::new(),
            l: 0,
            i: 0,
            restart: false,
        }
    }

    pub fn next_solution(&mut self) -> bool {
        let mut l = self.l;
        let mut i = self.i;

        loop {
            if self.restart {
                self.restart = false;
            } else if *self.problem.items().rlink(0) == 0 {
                self.l = l;
                self.i = i;
                self.restart = true;
                return true;
            } else {
                if self.x.len() == l as usize {
                    self.x.push(0);
                }
                self.problem.enter_level(i, l, self.x[l as usize]);
                i = self.choose();
                // TODO: return option from choose
                if self.problem.branch_degree(i) != 0 {
                    self.x[l as usize] = *self.problem.opts().dlink(i);
                    self.problem.prepare_to_branch(i, l, self.x[l as usize]);
                    if self.problem.try_item(i, l, self.x[l as usize]) {
                        l += 1;
                        continue;
                    } else {
                        self.problem.restore_item(i, l, self.x[l as usize]);
                    }
                }
            }
            loop {
                if l == 0 {
                    self.l = l;
                    return false;
                }
                l -= 1;
                i = *self.problem.opts().top(self.x[l as usize]) as Link;
                if self.problem.try_again(i, l, &mut self.x[l as usize]) {
                    l += 1;
                    break;
                }
            }
        }
    }

    fn choose(&mut self) -> Link {
        let mut min = Data::MAX;
        let mut p = *self.problem.items().rlink(0);
        let mut i = p;
        while p != 0 {
            let curr = self.problem.branch_degree(p);
            if curr < min {
                min = curr;
                i = p;
            }
            p = *self.problem.items().rlink(p);
        }
        i
    }

    pub fn find_options(&mut self) {
        let n =
            self.problem.items().primary() + self.problem.items().secondary();
        self.o.clear();
        for xj in &self.x[..self.l as usize] {
            let mut r = *xj;
            if r <= n {
                // TODO: report these elemente
                continue;
            }
            while *self.problem.opts().top(r) >= 0 {
                r += 1;
            }
            // Internal option indexes are 1-based
            self.o.push(-*self.problem.opts().top(r) - 1);
        }
    }
}
