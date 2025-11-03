#![allow(clippy::needless_range_loop)]

use std::ops::Range;

use dlx::{Int, OptData, Uint};

pub type Cell = (Uint, Uint);
pub type Shape = Omino;
type XForm = fn(Uint, Uint, Uint, Uint) -> (Uint, Uint);

pub struct Bounds(pub Uint, pub Uint, pub Uint, pub Uint);

impl Bounds {
    pub fn contains(&self, other: &Bounds) -> bool {
        self.0 <= other.0
            && self.1 <= other.1
            && self.2 >= other.2
            && self.3 >= other.3
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Omino {
    c: Vec<Cell>,
    xmin: Uint,
    xmax: Uint,
    ymin: Uint,
    ymax: Uint,
}

impl Omino {
    pub fn new(c: &[Cell]) -> Omino {
        let mut omino = Omino::raw(c);
        for i in 0..omino.c.len() {
            omino.c[i] = (omino.c[i].0 - omino.xmin, omino.c[i].1 - omino.ymin);
        }
        omino.xmax -= omino.xmin;
        omino.ymax -= omino.ymin;
        omino.xmin = 0;
        omino.ymin = 0;

        omino
    }

    fn raw(c: &[Cell]) -> Omino {
        let mut c = c.to_vec();
        c.sort_by_key(|cell| (cell.1, cell.0));
        let (xmin, xmax) =
            c.iter().fold((c[0].0, c[0].0), |(min, max), &item| {
                (min.min(item.0), max.max(item.0))
            });
        let ymin = c[0].1;
        let ymax = c.last().unwrap().1;

        Omino { c, xmin, ymin, xmax, ymax }
    }

    pub fn bounds(&self) -> Bounds {
        Bounds(self.xmin, self.ymin, self.xmax, self.ymax)
    }

    pub fn cell_at(&self, i: Uint) -> (Uint, Uint) {
        (self.c[i as usize].0, self.c[i as usize].1)
    }

    const XFORMS: [XForm; 8] = [
        |x, y, _, _| (x, y),             // identity
        |x, y, _, ym| (ym - y, x),       // 90
        |x, y, xm, ym| (xm - x, ym - y), // 180
        |x, y, xm, _| (y, xm - x),       // 270
        |x, y, _, _| (y, x),             // flip
        |x, y, _, ym| (x, ym - y),       // flip + 90
        |x, y, xm, ym| (ym - y, xm - x), // flip + 180
        |x, y, xm, _| (xm - x, y),       // flip + 270
    ];

    pub fn transform(&self, t: u8) -> Vec<Omino> {
        fn push_unique<T: PartialEq>(v: &mut Vec<T>, t: T) {
            if !v.contains(&t) {
                v.push(t);
            }
        }

        let mut ps = Vec::new();
        for b in 0..8 {
            if (t >> b) & 1 == 1 {
                push_unique(
                    &mut ps,
                    Omino::new(
                        &self
                            .c
                            .iter()
                            .map(|(x, y)| {
                                Self::XFORMS[b](*x, *y, self.xmax, self.ymax)
                            })
                            .collect::<Vec<_>>(),
                    ),
                );
            }
        }
        ps
    }

    pub fn all_options<S: SpecInfo>(
        &self, p: Uint, shape: &Shape, info: &S,
    ) -> Vec<Vec<S::OData>> {
        self.options(p, shape, info, |c, _| c)
    }

    pub fn options_filter<S: SpecInfo>(
        &self, p: Uint, shape: &Shape, info: &S, f: fn(&Omino) -> bool,
    ) -> Vec<Vec<S::OData>> {
        self.options(p, shape, info, move |c, p| if f(p) { c } else { vec![] })
    }

    #[allow(clippy::redundant_closure)]
    pub fn options<S: SpecInfo>(
        &self, p: Uint, shape: &Shape, info: &S,
        ext: impl Fn(Vec<S::OData>, &Omino) -> Vec<S::OData>,
    ) -> Vec<Vec<S::OData>> {
        let Bounds(xmin, ymin, xmax, ymax) = shape.bounds();
        let mut os = Vec::new();
        if self.ymax > ymax || self.xmax > xmax {
            return os;
        }
        for yd in ymin..=(ymax - self.ymax) {
            for xd in xmin..=(xmax - self.xmax) {
                let ctp = self
                    .c
                    .iter()
                    .map(|(x, y)| (x + xd, y + yd))
                    .collect::<Vec<_>>();
                let tp = Omino::raw(&ctp);
                let mut opt = vec![p];
                opt.extend(ctp.iter().map_while(|(x, y)| {
                    shape
                        .c
                        .iter()
                        .position(|&c| (c.0, c.1) == (*x, *y))
                        .map(|p| Uint(p))
                }));
                if opt.len() == self.c.len() + 1 {
                    let opt = opt
                        .iter()
                        .enumerate()
                        .map(|(i, &o)| {
                            if i == 0 {
                                info.piece_to_item(o)
                            } else {
                                info.cell_to_item(o)
                            }
                        })
                        .collect::<Vec<_>>();
                    let opt = ext(opt, &tp);
                    if !opt.is_empty() {
                        os.push(opt);
                    }
                }
            }
        }
        os
    }
}

pub trait SpecInfo {
    type OData: OptData;

    const PIECE_COUNT: usize;
    const CELL_COUNT: usize;

    fn cell_to_item(&self, i: Uint) -> Self::OData {
        Self::OData::new_item(i + Uint(Self::PIECE_COUNT))
    }
    fn item_to_cell(&self, i: Self::OData) -> Uint {
        i.get_item() - Uint(Self::PIECE_COUNT)
    }
    fn piece_to_item(&self, i: Uint) -> Self::OData {
        Self::OData::new_item(i)
    }
    // item_to_piece(i) must not equal item_to_cell(i)
    fn item_to_piece(&self, i: Self::OData) -> Uint {
        i.get_item()
    }
    fn cell_range(&self) -> Range<Uint> {
        (Self::PIECE_COUNT as Uint)
            ..(Self::PIECE_COUNT + Self::CELL_COUNT) as Uint
    }
    fn piece_range(&self) -> Range<Uint> {
        0..(Self::PIECE_COUNT as Uint)
    }
}

pub fn rectangle(rows: usize, cols: usize) -> Shape {
    let c = (0..rows)
        .flat_map(|i| (0..cols).map(move |j| (Uint(j), Uint(i))))
        .collect();
    Shape {
        c,
        xmin: 0,
        ymin: 0,
        xmax: Uint(cols - 1),
        ymax: Uint(rows - 1),
    }
}

pub struct SolutionGrid {
    cells: Vec<Vec<(usize, usize)>>,
    placements: Vec<Vec<(Uint, Uint)>>,
    piece_count: usize,
}

impl SolutionGrid {
    pub fn new<S: SpecInfo>(
        sol: &[Int], info: &S, os: &[Vec<S::OData>], shape: &Shape,
    ) -> SolutionGrid {
        let mut cells = vec![
            vec![(0, 0); (shape.xmax + 1) as usize];
            (shape.ymax + 1) as usize
        ];
        let mut placements = Vec::new();
        let mut idx = 0;

        for &opt_idx in sol {
            let opt = &os[opt_idx as usize];
            let mut ps = Vec::new();
            let mut cs = Vec::new();
            for i in 0..opt.len() {
                if info.piece_range().contains(&opt[i].get_item()) {
                    ps.push(info.item_to_piece(opt[i]));
                } else if info.cell_range().contains(&opt[i].get_item()) {
                    cs.push(info.item_to_cell(opt[i]));
                }
            }
            if ps.len() != 1 || cs.is_empty() {
                continue;
            }
            let cs = cs.iter().map(|x| shape.cell_at(*x)).collect::<Vec<_>>();
            for (x, y) in &cs {
                cells[*y as usize][*x as usize] =
                    ((idx + 1) as usize, (ps[0] + 1) as usize);
            }
            idx += 1;
            placements.push(cs);
        }
        SolutionGrid { cells, placements, piece_count: S::PIECE_COUNT }
    }

    pub fn print<T: AsRef<str>>(&self, names: &[T]) {
        assert_eq!(names.len(), self.piece_count);
        for line in &self.cells {
            println!(
                "{}",
                line.iter()
                    .map(|c| if c.1 == 0 {
                        " "
                    } else {
                        names[c.1 - 1].as_ref()
                    })
                    .collect::<String>()
            );
        }
    }

    pub fn colorize(&self, palette: &[u32]) {
        let mut names = Vec::new();
        for color in palette {
            let b = color & 0xff;
            let g = (color >> 8) & 0xff;
            let r = (color >> 16) & 0xff;
            let name = format!("\u{1b}[38;2;{};{};{}m█", r, g, b);
            names.push(name);
        }
        self.print(&names);
        print!("\u{1b}[0m");
    }
}

pub fn pentominoes() -> Vec<Omino> {
    vec![
        Omino::new(&[(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]), // O
        Omino::new(&[(0, 0), (1, 0), (0, 1), (1, 1), (0, 2)]), // P
        Omino::new(&[(0, 0), (1, 0), (2, 0), (3, 0), (3, 1)]), // Q
        Omino::new(&[(1, 0), (2, 0), (0, 1), (1, 1), (1, 2)]), // R
        Omino::new(&[(2, 0), (3, 0), (0, 1), (1, 1), (2, 1)]), // S
        Omino::new(&[(0, 0), (1, 0), (2, 0), (1, 1), (1, 2)]), // T
        Omino::new(&[(0, 0), (2, 0), (0, 1), (1, 1), (2, 1)]), // U
        Omino::new(&[(0, 0), (0, 1), (0, 2), (1, 2), (2, 2)]), // V
        Omino::new(&[(0, 0), (0, 1), (1, 1), (1, 2), (2, 2)]), // W
        Omino::new(&[(1, 0), (0, 1), (1, 1), (2, 1), (1, 2)]), // X
        Omino::new(&[(2, 0), (0, 1), (1, 1), (2, 1), (3, 1)]), // Y
        Omino::new(&[(0, 0), (1, 0), (1, 1), (1, 2), (2, 2)]), // Z
    ]
}

pub const PALETTE_12: [u32; 12] = [
    0xd8b2b2, 0xffbf7f, 0x7f3300, 0xe5e5f2, 0xffff00, 0xffcccc, 0x7f00cc,
    0x4c66cc, 0x007f00, 0xff007f, 0x3fff3f, 0xf2f2bf,
];

pub const PALETTE_35: [u32; 35] = [
    0x008080, 0xf802ff, 0xbc8f8e, 0x8f563e, 0xbfc0ff, 0xa52a2b, 0xa1d3a7,
    0x80ff80, 0xdc6f94, 0xcb9e70, 0xda9818, 0x01bf02, 0x01ffff, 0x483d8c,
    0xffff00, 0x7f8000, 0xf39908, 0xcc5d5b, 0xadd8e7, 0xffe5c5, 0xff02ff,
    0x4682b4, 0x12ffab, 0xd3691d, 0xbfd648, 0xefdc01, 0xff3344, 0x3a4cc8,
    0x0000ff, 0xff0100, 0xbcff0a, 0xffbf33, 0xffc0c1, 0xf29bd6, 0xfea079,
];
