#![allow(clippy::unnecessary_cast)]

use anyhow::{Result, anyhow, bail};

use crate::choose::Choose;

pub mod x;
pub mod c;
pub mod m;
pub mod mc;
pub mod choose;

pub type Link = usize;
pub type Count = Link;
pub type Data = isize;

const _: () = {
    assert!(Link::MAX as u128 <= u64::MAX as u128);
    assert!(Count::MAX as u128 <= u64::MAX as u128);
    assert!(Data::MAX as u128 <= u64::MAX as u128);
    assert!(Data::MAX as u128 <= Count::MAX as u128);
    assert!(Data::MAX as u128 <= Link::MAX as u128);
    assert!(Data::MIN < 0);
};

pub trait Dance {
    type I: Items;
    type O: Opts;

    fn items(&mut self) -> &mut Self::I;
    fn opts(&mut self) -> &mut Self::O;

    fn updates(&mut self) -> &mut isize;

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
    fn len(&self) -> Count;

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
    type Spec: Default + Copy;

    fn len(&mut self, i: Link) -> &mut Data;
    fn top(&mut self, i: Link) -> &mut Data;
    fn ulink(&mut self, i: Link) -> &mut Link;
    fn dlink(&mut self, i: Link) -> &mut Link;

    fn set_data(&mut self, pk: Link, s: Self::Spec) -> Link;

    fn init_links(
        &mut self, n: Count, order: OptOrder, os: &[Vec<Self::Spec>],
    ) {
        let mut order = order;
        for i in (1 as Link)..=n {
            *self.ulink(i) = i;
            *self.dlink(i) = i;
        }
        let mut m: Data = 0;
        let mut p: Link = n + 1;

        for opt in os {
            let mut k = 0;
            for node in opt {
                k += 1;
                // Internal item indexes are 1-based.
                let i = self.set_data(p + k, *node) + 1;
                // TODO: i <= Data::MAX

                *self.len(i) += 1;
                let q = match order {
                    OptOrder::Seq => *self.ulink(i),
                    OptOrder::Rnd(ref mut rng) => {
                        let mut i = i;
                        let p = rng.uniform(*self.len(i) as u32);
                        for _ in 0..p {
                            i = *self.dlink(i);
                        }
                        i
                    }
                };
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
            self.set_data(p, Default::default());
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

pub struct Solver<P> {
    problem: P,
    x: Vec<Link>,
    o: Vec<isize>,
    profile: Vec<usize>,
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
            profile: Vec::new(),
            l: 0,
            i: 0,
            restart: false,
        }
    }

    pub fn next_solution<C: Choose<P>>(&mut self, chooser: &mut C) -> bool {
        let mut l = self.l;
        let mut i = self.i;
        if *self.problem.updates() < 0 {
            *self.problem.updates() = 0;
        }

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
                    self.profile.push(0);
                    self.problem.enter_level(i, l, self.x[l as usize]);
                }
                self.profile[l] += 1;
                i = chooser.choose(&mut self.problem);
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
                    *self.problem.updates() = -*self.problem.updates();
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

    pub fn get_solution(&mut self) -> &[isize] {
        let n = self.problem.items().len();
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
        &self.o
    }

    pub fn get_updates(&mut self) -> isize {
        *self.problem.updates()
    }

    pub fn get_profile(&self) -> &[usize] {
        &self.profile
    }
}

pub struct Spec {
    pub primary: Vec<String>,
    pub secondary: Vec<String>,
    pub opts: Vec<Vec<String>>,
}

impl Spec {
    pub fn new(spec: &str, sharp_pref: bool) -> Result<Spec> {
        use std::cmp::Ordering;
        let mut lines = spec
            .lines()
            .map(str::trim)
            .filter(|s| !s.is_empty() && !s.starts_with('|'));
        let items =
            lines.next().ok_or_else(|| anyhow!("No items specified"))?;
        let opts: Vec<String> = lines.map(String::from).collect();
        if opts.is_empty() {
            bail!("No options specified");
        }
        let item_list = items
            .split_whitespace()
            .map(String::from)
            .collect::<Vec<_>>();
        let items = item_list.split(|e| e == "|").collect::<Vec<_>>();
        if items.len() > 2 {
            bail!("Too many '|' separators");
        }
        let secondary = if items.len() > 1 {
            if items[1].is_empty() {
                bail!("No seecondary items specified");
            }
            // TODO: no '#' in secondary?
            items[1].to_vec()
        } else {
            Vec::new()
        };
        let mut primary = items[0].to_vec();
        primary.sort_by(|a, b| {
            let a_sharp = a.contains("#");
            let b_sharp = b.contains("#");
            if a_sharp == b_sharp {
                Ordering::Equal
            } else if a_sharp == sharp_pref {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        });
        let opts = opts
            .iter()
            .map(|o| o.split_whitespace().map(String::from).collect())
            .collect();
        Ok(Spec { primary, secondary, opts })
    }
}

pub struct Rng {
    state: u32,
}

#[allow(clippy::should_implement_trait)]
impl Rng {
    pub fn new(state: u32) -> Rng {
        assert_ne!(state, 0);
        Rng { state }
    }

    pub fn next(&mut self) -> u32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        x
    }

    pub fn uniform(&mut self, max: u32) -> u32 {
        let t = 0x80000000 - (0x80000000 % max);
        let mut r;
        loop {
            r = self.next();
            if t > r {
                break;
            }
        }
        r % max
    }
}

pub enum OptOrder {
    Seq,
    Rnd(Rng),
}
