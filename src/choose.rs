use crate::{Dance, Data, Items, Link, Opts};

pub trait Choose {
    fn choose<D: Dance>(&mut self, dance: &mut D) -> Link;
}

pub trait Preference {
    fn prefer(&self, i: Link) -> bool;
}

pub fn mrv_chooser<P: Preference>(pref: P) -> impl Choose {
    MRVChooser { pref }
}

pub fn prefer_any() -> impl Preference {
    PreferAny
}

pub fn prefer_first_n(n: Link) -> impl Preference {
    PreferFirstN(n)
}

struct MRVChooser<P: Preference> {
    pref: P,
}

impl<P: Preference> MRVChooser<P> {
    fn choose<D: Dance>(&mut self, dance: &mut D) -> Link {
        let mut min = Data::MAX;
        let mut p = *dance.items().rlink(0);
        let mut i = p;
        while p != 0 {
            let mut curr = dance.branch_degree(p);
            if !self.pref.prefer(p) {
                curr += *dance.opts().len(p);
            }
            if curr < min {
                min = curr;
                i = p;
            }
            p = *dance.items().rlink(p);
        }
        i
    }
}

impl<P: Preference> Choose for MRVChooser<P> {
    fn choose<D: Dance>(&mut self, links: &mut D) -> Link {
        self.choose(links)
    }
}

struct PreferAny;

impl Preference for PreferAny {
    fn prefer(&self, _: Link) -> bool {
        true
    }
}

struct PreferFirstN(Link);

impl Preference for PreferFirstN {
    fn prefer(&self, i: Link) -> bool {
        i < self.0
    }
}
