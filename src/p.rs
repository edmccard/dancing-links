use crate::c::{DanceC, Problem};
use crate::{Count, Dance, Data, Items, Link};

pub struct Preproc<'a> {
    problem: &'a mut Problem,
    rounds: usize,
    // The first option node after the header nodes
    opt_start: Link,
    stack: Data,
    change: bool,
}

impl<'a> Preproc<'a> {
    pub fn new(problem: &mut Problem) -> Preproc<'_> {
        //let mut problem = problem;
        let opt_start = problem.items().count() + 2;
        Preproc {
            problem,
            rounds: 0,
            opt_start,
            stack: 0,
            change: false,
        }
    }

    pub fn reduce(&mut self, max_rounds: usize) {
        for p_itm in 1..=self.problem.items().primary() {
            if *self.problem.len(p_itm) == 0 {
                // TODO: indicate failure (primary item is in no options)
                return;
            }
        }

        'l1: while self.rounds < max_rounds {
            self.rounds += 1;
            self.change = false;
            for itm in 1..=self.problem.items().count() {
                if *self.problem.len(itm) != 0 {
                    self.reduce_options(itm);
                }
            }
            if !self.change {
                break;
            }
        }
    }

    pub fn get_items(&mut self) -> (Vec<Count>, Vec<Count>) {
        let mut ps = Vec::new();
        let mut ss = Vec::new();
        for c in 1..=self.problem.items().count() {
            if *self.problem.len(c) == 0 {
                continue;
            }
            if self.is_primary(c as Data) {
                ps.push(c - 1);
            } else {
                ss.push(c - 1);
            }
        }
        (ps, ss)
    }

    pub fn get_options(&mut self) -> (Vec<Count>, Vec<Vec<(Count, Data)>>) {
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
                        let opt = self.get_option(r);
                        idx.push(opt.0);
                        os.push(opt.1);
                    }
                    r = *self.problem.dlink(r);
                }
            }
        }
        (idx, os)
    }

    fn get_option(&mut self, p: Link) -> (Count, Vec<(Count, Data)>) {
        let mut p = p - 1;
        while *self.problem.top(p) > 0 || *self.problem.dlink(p) < p {
            p -= 1;
        }
        let mut q = p + 1;
        let mut o = Vec::new();
        loop {
            let itm = *self.problem.top(q);
            if itm < 0 {
                return ((-itm - 1) as Count, o);
            }
            if itm > 0 {
                o.push(((itm - 1) as Count, *self.problem.color(q)));
            }
            q += 1;
        }
    }

    fn reduce_options(&mut self, itm: Link) {
        self.stack = 0;
        self.hide(itm);
        if self.stack != 0 {
            self.remove_item(itm);
        } else {
            let mut r = *self.problem.dlink(itm);
            'l5: while r >= self.opt_start {
                let mut q = r - 1;
                'l6: while *self.problem.dlink(q) == q - 1 {
                    q -= 1;
                } // end l6
                if *self.problem.top(q) <= 0 && *self.problem.color(r) == 0 {
                    // Stack option r for deletion if it leaves
                    // some primary item oncoverable
                    q = r + 1;
                    'l7: loop {
                        let cc = *self.problem.top(q);
                        if cc <= 0 {
                            q = *self.problem.ulink(q);
                            if q > r {
                                continue;
                            }
                            break;
                        }
                        *self.problem.color(cc as Link) = r as Data;
                        q += 1;
                    } // end l7
                    if !self.hide_entries(r) {
                        self.backup(r - 1, r);
                    } else {
                        // Mark the unnecessary option
                        self.change = true;
                        *self.problem.top(r) = self.stack;
                        self.stack = r as Data;
                    }
                }
                r = *self.problem.dlink(r);
            }
            self.unhide(itm);
            r = self.stack as Count;
            'l12: while r != 0 {
                let rr = *self.problem.top(r) as Link;
                *self.problem.top(r) = itm as Data;
                self.really_delete_option(r);
                r = rr;
            } // end l12
        }
    }

    fn hide_entries(&mut self, r: Link) -> bool {
        let mut q = r + 1;
        'l8: loop {
            let cc = *self.problem.top(q);
            if cc <= 0 {
                q = *self.problem.ulink(q);
                if q > r {
                    continue;
                }
                return false;
            }
            let x = *self.problem.color(q);
            let mut p = *self.problem.dlink(cc as Link);
            'l9: while p >= self.opt_start {
                if x > 0 && x == *self.problem.color(p) {
                    p = *self.problem.dlink(p);
                    continue;
                }
                // Hide the entries of option p, or goto backup
                // BEGIN 37
                let mut qq = p + 1;
                'l15: while qq != p {
                    let cc = *self.problem.top(qq);
                    if cc <= 0 {
                        qq = *self.problem.ulink(qq);
                        continue;
                    }
                    let t = *self.problem.len(cc as Link) - 1;
                    if t == 0
                        && self.is_primary(cc)
                        && *self.problem.color(cc as Link) != r as Data
                    {
                        self.unhide_entries(qq - 1, p);
                        let p = *self.problem.ulink(p);
                        self.pass_2(p, x);
                        self.backup(q - 1, r);
                        return true;
                    }
                    *self.problem.len(cc as Link) = t;
                    let uu = *self.problem.ulink(qq);
                    let dd = *self.problem.dlink(qq);
                    *self.problem.dlink(uu) = dd;
                    *self.problem.ulink(dd) = uu;
                    qq += 1;
                } // end l15
                // END 37
                p = *self.problem.dlink(p);
            } // end l9
            q += 1;
        } // end l8
    }

    fn backup(&mut self, q: Link, r: Link) {
        let mut q = q;
        while q != r {
            let cc = *self.problem.top(q);
            if cc <= 0 {
                q = *self.problem.dlink(q);
                continue;
            }
            let x = *self.problem.color(q);
            let p = *self.problem.ulink(cc as Link);
            self.pass_2(p, x);
            q -= 1;
        }
    }

    fn pass_2(&mut self, p: Link, x: Data) {
        let mut p = p;
        while p >= self.opt_start {
            if x > 0 && x == *self.problem.color(p) {
                p = *self.problem.ulink(p);
                continue;
            }
            self.unhide_entries(p - 1, p);
            p = *self.problem.ulink(p);
        }
    }

    fn unhide_entries(&mut self, qq: Link, p: Link) {
        let mut qq = qq;
        while qq != p {
            let cc = *self.problem.top(qq);
            if cc <= 0 {
                qq = *self.problem.dlink(qq);
                continue;
            }
            *self.problem.len(cc as Link) += 1;
            let uu = *self.problem.ulink(qq);
            let dd = *self.problem.dlink(qq);
            *self.problem.dlink(uu) = qq;
            *self.problem.ulink(dd) = qq;
            // midst
            qq -= 1;
        } // end l14
    }

    fn remove_item(&mut self, c: Link) {
        // Remove item c, and maybe some options
        self.unhide(c);
        let mut r = *self.problem.dlink(c);
        'l3: while r >= self.opt_start {
            let rrr = *self.problem.dlink(r);
            // Delete or shorten option r
            // BEGIN 33
            let mut q = r + 1;
            'l31: while q != r {
                let cc = *self.problem.top(q);
                if cc <= 0 {
                    q = *self.problem.ulink(q);
                    continue;
                }
                if cc == self.stack {
                    break;
                }
                q += 1;
            } // end l3
            if q != r {
                // Shorten and retain option r
                // BEGIN 34
                *self.problem.ulink(r) = r + 1;
                *self.problem.dlink(r) = r - 1;
                *self.problem.top(r) = 0;
            // END 34
            } else {
                // Delete option r
                // BEGIN 35
                q = r + 1;
                'l4: while q != r {
                    let cc = *self.problem.top(q);
                    if cc <= 0 {
                        q = *self.problem.ulink(q);
                        continue;
                    }
                    let t = *self.problem.len(cc as Link) - 1;
                    if t == 0 && self.is_primary(cc) {
                        // TODO: Primary item in no options
                        return;
                    }
                    *self.problem.len(cc as Link) = t;
                    let uu = *self.problem.ulink(q);
                    let dd = *self.problem.dlink(q);
                    *self.problem.dlink(uu) = dd;
                    *self.problem.ulink(dd) = uu;
                    q += 1;
                } // end l4
                // END 35
            }
            r = rrr;
        }
        // END 33
        *self.problem.ulink(c) = c;
        *self.problem.dlink(c) = c;
        *self.problem.len(c) = 0;
        self.change = true;
    }

    fn really_delete_option(&mut self, r: Link) {
        let mut p = r + 1;
        'l13: loop {
            let cc = *self.problem.top(p);
            if cc <= 0 {
                p = *self.problem.ulink(p);
                continue;
            }
            let uu = *self.problem.ulink(p);
            let dd = *self.problem.dlink(p);
            *self.problem.dlink(uu) = dd;
            *self.problem.ulink(dd) = uu;
            *self.problem.len(cc as Link) -= 1;
            if *self.problem.len(cc as Link) == 0 {
                // Take note that cc has no options
                // BEGIN 41
                if self.is_primary(cc) {
                    // Terminate with unfeasible item cc
                    return;
                }
                // END 41
            }
            if p == r {
                break;
            }
            p += 1;
        } // end l13
    }

    fn is_primary(&mut self, i: Data) -> bool {
        (i as Count) <= self.problem.items().primary()
    }

    fn hide(&mut self, c: Link) {
        let mut rr = *self.problem.dlink(c);
        while rr >= self.opt_start {
            if *self.problem.color(rr) == 0 {
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
                    let t = *self.problem.len(cc as Link) - 1;
                    *self.problem.len(cc as Link) = t;
                    if t == 0 && self.is_primary(cc) {
                        self.stack = cc as Data;
                    }
                    nn += 1;
                }
            }
            rr = *self.problem.dlink(rr);
        }
    }

    fn unhide(&mut self, c: Link) {
        let mut rr = *self.problem.dlink(c);
        while rr >= self.opt_start {
            if *self.problem.color(rr) == 0 {
                let mut nn = rr + 1;
                while nn != rr {
                    let uu = *self.problem.ulink(nn);
                    let dd = *self.problem.dlink(nn);
                    let cc = *self.problem.top(nn);
                    if cc <= 0 {
                        nn = uu;
                        continue;
                    }
                    let t = *self.problem.len(cc as Link);
                    *self.problem.dlink(uu) = nn;
                    *self.problem.ulink(dd) = nn;
                    *self.problem.len(cc as Link) = t + 1;
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
        let os: Vec<Vec<(Count, Data)>> = vec![
            vec![(0, 0), (1, 0), (3, 48), (4, 48)],
            vec![(0, 0), (2, 0), (3, 49), (4, 49)],
            vec![(3, 48), (4, 49)],
            vec![(1, 0), (3, 49)],
            vec![(2, 0), (4, 49)],
        ];
        let opts = ONodes::new(5, 3, &os, OptOrder::Seq);
        let mut problem = Problem::new(items, opts);
        let mut preproc = Preproc::new(&mut problem);
        preproc.reduce(200);
        assert_eq!(preproc.get_items(), (vec![0, 1], vec![3, 4]));
        assert_eq!(
            preproc.get_options(),
            (
                vec![1, 3],
                vec![vec![(0, 0), (3, 49), (4, 49)], vec![(1, 0), (3, 49)]]
            )
        );
    }
}
