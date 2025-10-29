use anyhow::{Result, bail};

use crate::c;
use crate::c::DanceC;
use crate::x;
use crate::{Dance, Int, Items, Opts, Uint};

type OSpec<R> = <<R as Dance>::O as Opts>::Spec;

pub trait Reduce: Dance {
    fn get_color(&mut self, n: Uint) -> Int;
    fn get_opt_data(
        &self, i: Uint, c: Int,
    ) -> <<Self as Dance>::O as Opts>::Spec;
}

impl Reduce for x::Problem {
    fn get_color(&mut self, _n: Uint) -> Int {
        0
    }

    fn get_opt_data(&self, i: Uint, _c: Int) -> Uint {
        i
    }
}

impl Reduce for c::Problem {
    fn get_color(&mut self, n: Uint) -> Int {
        *self.color(n)
    }

    fn get_opt_data(&self, i: Uint, c: Int) -> (Uint, Int) {
        (i, c)
    }
}

pub struct Preproc<'a, R> {
    problem: &'a mut R,
    aux: Vec<Int>,
    rounds: usize,
    // The first option node after the header nodes
    opt_start: Uint,
    stack: Int,
    change: bool,
}

impl<'a, R: Reduce> Preproc<'a, R> {
    pub fn new(problem: &mut R) -> Preproc<'_, R> {
        //let mut problem = problem;
        let opt_start = problem.items().count() + 2;
        Preproc {
            problem,
            aux: vec![0; (opt_start - 1) as usize],
            rounds: 0,
            opt_start,
            stack: 0,
            change: false,
        }
    }

    pub fn reduce(
        &mut self, max_rounds: usize,
    ) -> Result<(Uint, Uint, Vec<Vec<OSpec<R>>>, Vec<Uint>)> {
        for p_itm in 1..=self.problem.items().primary() {
            if *self.problem.len(p_itm) == 0 {
                bail!("Primary item {} is not in any option", p_itm);
            }
        }

        while self.rounds < max_rounds {
            self.rounds += 1;
            self.change = false;
            for itm in 1..=self.problem.items().count() {
                if *self.problem.len(itm) != 0 {
                    self.reduce_options(itm)?;
                }
            }
            if !self.change {
                break;
            }
        }
        let is = self.get_items();
        let (np, ns) = (is.0.len() as Uint, is.1.len() as Uint);
        let os = self.get_options();
        Ok((np, ns, os.1, os.0))
    }

    fn get_items(&mut self) -> (Vec<Uint>, Vec<Uint>) {
        let mut ps = Vec::new();
        let mut ss = Vec::new();
        for c in 1..=self.problem.items().count() {
            if *self.problem.len(c) == 0 {
                continue;
            }
            if self.is_primary(c as Int) {
                ps.push(c - 1);
            } else {
                ss.push(c - 1);
            }
        }
        (ps, ss)
    }

    fn get_options(&mut self) -> (Vec<Uint>, Vec<Vec<OSpec<R>>>) {
        // TODO: Verify that secondary are sequential like primary
        let (ps, ss) = self.get_items();
        let sd = if ss.is_empty() {
            0
        } else {
            ss[0] - ps.last().unwrap() - 1
        };
        let mut idx = Vec::new();
        let mut os = Vec::new();
        for c in 1..=self.problem.items().count() {
            if *self.problem.len(c) != 0 {
                let mut r = *self.problem.dlink(c);
                while r >= self.opt_start {
                    let mut q = r - 1;
                    while *self.problem.dlink(q) == q - 1 {
                        q -= 1;
                    }
                    if *self.problem.top(q) <= 0 {
                        let opt = self.get_option(r, sd);
                        idx.push(opt.0);
                        os.push(opt.1);
                    }
                    r = *self.problem.dlink(r);
                }
            }
        }
        (idx, os)
    }

    fn get_option(&mut self, p: Uint, sd: Uint) -> (Uint, Vec<OSpec<R>>) {
        let mut p = p - 1;
        while *self.problem.top(p) > 0 || *self.problem.dlink(p) < p {
            p -= 1;
        }
        let mut q = p + 1;
        let mut o: Vec<OSpec<R>> = Vec::new();
        loop {
            let itm = *self.problem.top(q);
            if itm < 0 {
                return ((-itm - 1) as Uint, o);
            }
            if itm > 0 {
                let itm = if itm > self.problem.items().primary() as Int {
                    itm - (sd as Int)
                } else {
                    itm
                };
                let clr = self.problem.get_color(q);
                let data = self.problem.get_opt_data((itm - 1) as Uint, clr);
                o.push(data);
            }
            q += 1;
        }
    }

    fn reduce_options(&mut self, itm: Uint) -> Result<()> {
        self.stack = 0;
        self.hide(itm);
        if self.stack != 0 {
            // TODO: option to skip removing redundant item
            self.remove_item(itm)?;
        } else {
            let mut r = *self.problem.dlink(itm);
            while r >= self.opt_start {
                let mut q = r - 1;
                while *self.problem.dlink(q) == q - 1 {
                    q -= 1;
                }
                if *self.problem.top(q) <= 0 && self.problem.get_color(r) == 0 {
                    // Stack option r for deletion if it leaves
                    // some primary item oncoverable
                    q = r + 1;
                    loop {
                        let cc = *self.problem.top(q);
                        if cc <= 0 {
                            q = *self.problem.ulink(q);
                            if q > r {
                                continue;
                            }
                            break;
                        }
                        self.aux[cc as usize] = r as Int;
                        q += 1;
                    }
                    if !self.hide_entries(r) {
                        self.backup(r - 1, r);
                    } else {
                        // Mark the unnecessary option
                        self.change = true;
                        *self.problem.top(r) = self.stack;
                        self.stack = r as Int;
                    }
                }
                r = *self.problem.dlink(r);
            }
            self.unhide(itm);
            r = self.stack as Uint;
            while r != 0 {
                let rr = *self.problem.top(r) as Uint;
                *self.problem.top(r) = itm as Int;
                self.really_delete_option(r);
                r = rr;
            }
        }
        Ok(())
    }

    fn hide_entries(&mut self, r: Uint) -> bool {
        let mut q = r + 1;
        loop {
            let cc = *self.problem.top(q);
            if cc <= 0 {
                q = *self.problem.ulink(q);
                if q > r {
                    continue;
                }
                return false;
            }
            let x = self.problem.get_color(q);
            let mut p = *self.problem.dlink(cc as Uint);
            while p >= self.opt_start {
                if x > 0 && x == self.problem.get_color(p) {
                    p = *self.problem.dlink(p);
                    continue;
                }
                // Hide the entries of option p, or goto backup
                let mut qq = p + 1;
                while qq != p {
                    let cc = *self.problem.top(qq);
                    if cc <= 0 {
                        qq = *self.problem.ulink(qq);
                        continue;
                    }
                    let t = *self.problem.len(cc as Uint) - 1;
                    if t == 0
                        && self.is_primary(cc)
                        && self.aux[cc as usize] != r as Int
                    {
                        self.unhide_entries(qq - 1, p);
                        let p = *self.problem.ulink(p);
                        self.pass_2(p, x);
                        self.backup(q - 1, r);
                        return true;
                    }
                    *self.problem.len(cc as Uint) = t;
                    let uu = *self.problem.ulink(qq);
                    let dd = *self.problem.dlink(qq);
                    *self.problem.dlink(uu) = dd;
                    *self.problem.ulink(dd) = uu;
                    qq += 1;
                }
                p = *self.problem.dlink(p);
            }
            q += 1;
        }
    }

    fn backup(&mut self, q: Uint, r: Uint) {
        let mut q = q;
        while q != r {
            let cc = *self.problem.top(q);
            if cc <= 0 {
                q = *self.problem.dlink(q);
                continue;
            }
            let x = self.problem.get_color(q);
            let p = *self.problem.ulink(cc as Uint);
            self.pass_2(p, x);
            q -= 1;
        }
    }

    fn pass_2(&mut self, p: Uint, x: Int) {
        let mut p = p;
        while p >= self.opt_start {
            if x > 0 && x == self.problem.get_color(p) {
                p = *self.problem.ulink(p);
                continue;
            }
            self.unhide_entries(p - 1, p);
            p = *self.problem.ulink(p);
        }
    }

    fn unhide_entries(&mut self, qq: Uint, p: Uint) {
        let mut qq = qq;
        while qq != p {
            let cc = *self.problem.top(qq);
            if cc <= 0 {
                qq = *self.problem.dlink(qq);
                continue;
            }
            *self.problem.len(cc as Uint) += 1;
            let uu = *self.problem.ulink(qq);
            let dd = *self.problem.dlink(qq);
            *self.problem.dlink(uu) = qq;
            *self.problem.ulink(dd) = qq;
            // midst
            qq -= 1;
        }
    }

    fn remove_item(&mut self, c: Uint) -> Result<()> {
        // Remove item c, and maybe some options
        self.unhide(c);
        let mut r = *self.problem.dlink(c);
        while r >= self.opt_start {
            let rrr = *self.problem.dlink(r);
            // Delete or shorten option r
            let mut q = r + 1;
            while q != r {
                let cc = *self.problem.top(q);
                if cc <= 0 {
                    q = *self.problem.ulink(q);
                    continue;
                }
                if cc == self.stack {
                    break;
                }
                q += 1;
            }
            if q != r {
                // Shorten and retain option r
                *self.problem.ulink(r) = r + 1;
                *self.problem.dlink(r) = r - 1;
                *self.problem.top(r) = 0;
            } else {
                // Delete option r
                q = r + 1;
                while q != r {
                    let cc = *self.problem.top(q);
                    if cc <= 0 {
                        q = *self.problem.ulink(q);
                        continue;
                    }
                    let t = *self.problem.len(cc as Uint) - 1;
                    if t == 0 && self.is_primary(cc) {
                        bail!("Primary item {} is not in any option", cc);
                    }
                    *self.problem.len(cc as Uint) = t;
                    let uu = *self.problem.ulink(q);
                    let dd = *self.problem.dlink(q);
                    *self.problem.dlink(uu) = dd;
                    *self.problem.ulink(dd) = uu;
                    q += 1;
                }
            }
            r = rrr;
        }
        *self.problem.ulink(c) = c;
        *self.problem.dlink(c) = c;
        *self.problem.len(c) = 0;
        self.change = true;
        Ok(())
    }

    fn really_delete_option(&mut self, r: Uint) {
        let mut p = r + 1;
        loop {
            let cc = *self.problem.top(p);
            if cc <= 0 {
                p = *self.problem.ulink(p);
                continue;
            }
            let uu = *self.problem.ulink(p);
            let dd = *self.problem.dlink(p);
            *self.problem.dlink(uu) = dd;
            *self.problem.ulink(dd) = uu;
            *self.problem.len(cc as Uint) -= 1;
            if *self.problem.len(cc as Uint) == 0 {
                // Take note that cc has no options
                if self.is_primary(cc) {
                    // Terminate with unfeasible item cc
                    return;
                }
            }
            if p == r {
                break;
            }
            p += 1;
        }
    }

    fn is_primary(&mut self, i: Int) -> bool {
        (i as Uint) <= self.problem.items().primary()
    }

    fn hide(&mut self, c: Uint) {
        let mut rr = *self.problem.dlink(c);
        while rr >= self.opt_start {
            if self.problem.get_color(rr) == 0 {
                let mut nn = rr + 1;
                while nn != rr {
                    let uu = *self.problem.ulink(nn);
                    let dd = *self.problem.dlink(nn);
                    let cc = *self.problem.top(nn);
                    if cc <= 0 {
                        nn = uu;
                        continue;
                    }
                    *self.problem.dlink(uu) = dd;
                    *self.problem.ulink(dd) = uu;
                    let t = *self.problem.len(cc as Uint) - 1;
                    *self.problem.len(cc as Uint) = t;
                    // TODO: option to skip removing redundant items
                    if t == 0 && self.is_primary(cc) {
                        self.stack = cc as Int;
                    }
                    nn += 1;
                }
            }
            rr = *self.problem.dlink(rr);
        }
    }

    fn unhide(&mut self, c: Uint) {
        let mut rr = *self.problem.dlink(c);
        while rr >= self.opt_start {
            if self.problem.get_color(rr) == 0 {
                let mut nn = rr + 1;
                while nn != rr {
                    let uu = *self.problem.ulink(nn);
                    let dd = *self.problem.dlink(nn);
                    let cc = *self.problem.top(nn);
                    if cc <= 0 {
                        nn = uu;
                        continue;
                    }
                    let t = *self.problem.len(cc as Uint);
                    *self.problem.dlink(uu) = nn;
                    *self.problem.ulink(dd) = nn;
                    *self.problem.len(cc as Uint) = t + 1;
                    nn += 1;
                }
            }
            rr = *self.problem.dlink(rr);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::OptOrder;
    use crate::x::INodes;

    #[test]
    fn test_preproc_color() {
        use crate::c::ONodes;
        use crate::c::Problem;
        let items = INodes::new(3, 2);
        let os: Vec<Vec<(Uint, Int)>> = vec![
            vec![(0, 0), (1, 0), (3, 48), (4, 48)],
            vec![(0, 0), (2, 0), (3, 49), (4, 49)],
            vec![(3, 48), (4, 49)],
            vec![(1, 0), (3, 49)],
            vec![(2, 0), (4, 49)],
        ];
        let opts = ONodes::new(5, 3, &os, OptOrder::Seq);
        let mut problem = Problem::new(items, opts);
        let mut preproc = Preproc::new(&mut problem);
        let (np, ns, os, orig) = preproc.reduce(200).unwrap();
        assert_eq!((np, ns), (2, 2));
        assert_eq!(orig, vec![1, 3]);
        assert_eq!(
            os,
            vec![vec![(0, 0), (2, 49), (3, 49)], vec![(1, 0), (2, 49)]]
        );
    }

    #[test]
    fn test_preproc() {
        use crate::x::ONodes;
        use crate::x::Problem;
        let items = INodes::new(5, 2);
        let os: Vec<Vec<Uint>> = vec![
            vec![2, 4, 5],
            vec![0, 3, 6],
            vec![1, 2, 5],
            vec![0, 3],
            vec![1, 6],
            vec![3, 4, 5],
        ];
        let opts = ONodes::new(7, 5, &os, OptOrder::Seq);
        let mut problem = Problem::new(items, opts);
        let mut preproc = Preproc::new(&mut problem);
        let (np, ns, os, orig) = preproc.reduce(200).unwrap();
        assert_eq!((np, ns), (3, 0));
        assert_eq!(orig, vec![3, 4, 0]);
        assert_eq!(os, vec![vec![0], vec![1], vec![2]]);
    }
}
