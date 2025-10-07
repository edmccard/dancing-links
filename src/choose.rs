use std::marker::PhantomData;

use crate::m::ItemsM;
use crate::{Dance, Data, Items, Link, Opts, Rng};

pub trait Choose<D: Dance> {
    fn choose(&mut self, dance: &mut D) -> Link;
}

pub trait Preference {
    fn prefer(&self, i: Link) -> bool;
}

pub trait Tiebreak {
    type D: Dance;

    fn reset(&mut self);
    fn replace(&mut self, i0: Link, i1: Link, dance: &mut Self::D) -> bool;
}

pub fn mrv_chooser<D: Dance, P: Preference, T: Tiebreak<D = D>>(
    pref: P, tbreak: T,
) -> impl Choose<D> {
    MRVChooser { pref, tbreak }
}

pub fn prefer_any() -> impl Preference {
    PreferAny
}

pub fn prefer_first_n(n: Link) -> impl Preference {
    PreferFirstN(n)
}

pub fn no_tiebreak<D: Dance>() -> impl Tiebreak<D = D> {
    NoTiebreak(PhantomData)
}

pub fn rnd_tiebreak<D: Dance>(seed: u32) -> impl Tiebreak<D = D> {
    RndTiebreak {
        rng: Rng::new(seed),
        c: 1,
        _phantom: PhantomData::<D>,
    }
}

pub fn knuth_tiebreak<D: Dance<I: ItemsM>>() -> impl Tiebreak<D = D> {
    KnuthTiebreak(PhantomData::<D>)
}

struct MRVChooser<P, T> {
    pref: P,
    tbreak: T,
}

impl<P: Preference, T: Tiebreak> MRVChooser<P, T> {
    fn choose(&mut self, dance: &mut T::D) -> Link {
        let mut min = Data::MAX;
        let mut p = *dance.items().rlink(0);
        let mut i = p;
        while p != 0 {
            let mut curr = dance.branch_degree(p);
            if !self.pref.prefer(p) {
                curr += *dance.opts().len(p);
            }
            if curr < min {
                self.tbreak.reset();
                min = curr;
                i = p;
            } else if curr == min {
                if self.tbreak.replace(i, p, dance) {
                    min = curr;
                    i = p
                }
            }
            p = *dance.items().rlink(p);
        }
        i
    }
}

impl<D: Dance, P: Preference, T: Tiebreak<D = D>> Choose<D>
    for MRVChooser<P, T>
{
    fn choose(&mut self, links: &mut T::D) -> Link {
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

struct NoTiebreak<D>(PhantomData<D>);

impl<D: Dance> Tiebreak for NoTiebreak<D> {
    type D = D;

    fn reset(&mut self) {}
    fn replace(&mut self, _: Link, _: Link, _: &mut D) -> bool {
        false
    }
}

struct RndTiebreak<D> {
    rng: Rng,
    c: u32,
    _phantom: PhantomData<D>,
}

impl<D: Dance> Tiebreak for RndTiebreak<D> {
    type D = D;
    fn reset(&mut self) {
        self.c = 1;
    }

    fn replace(&mut self, _: Link, _: Link, _: &mut Self::D) -> bool {
        self.c += 1;
        self.rng.uniform(self.c) == 0
    }
}

struct KnuthTiebreak<D>(PhantomData<D>);

impl<D: Dance<I: ItemsM>> Tiebreak for KnuthTiebreak<D> {
    type D = D;

    fn reset(&mut self) {}

    fn replace(&mut self, i0: Link, i1: Link, dance: &mut Self::D) -> bool {
        dance.items().slack(i1) < dance.items().slack(i0)
            || (dance.items().slack(i1) == dance.items().slack(i0)
                && *dance.opts().len(i1) > *dance.opts().len(i0))
    }
}
