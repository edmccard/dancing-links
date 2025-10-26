use dlx::Count;

type Cell = (Count, Count);

type Shape = Omino;

fn rectangle(rows: usize, cols: usize) -> Shape {
    let c = (0..rows)
        .flat_map(|i| (0..cols).map(move |j| (Count(j), Count(i))))
        .collect();
    Shape {
        c,
        xmin: 0,
        ymin: 0,
        xmax: Count(cols - 1),
        ymax: Count(rows - 1),
    }
}

fn pentominoes() -> Vec<Omino> {
    vec![
        Omino::new(&[(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]), // O
        Omino::new(&[(0, 0), (1, 0), (0, 1), (1, 1), (0, 2)]), // P
        Omino::new(&[(0, 0), (1, 0), (2, 0), (3, 0), (3, 1)]), // Q
        Omino::new(&[(1, 0), (2, 0), (0, 1), (1, 1), (1, 2)]), // R
        Omino::new(&[(0, 0), (0, 1), (1, 1), (1, 2), (1, 3)]), // S
        Omino::new(&[(0, 0), (1, 0), (2, 0), (1, 1), (1, 2)]), // T
        Omino::new(&[(0, 0), (2, 0), (0, 1), (1, 1), (2, 1)]), // U
        Omino::new(&[(0, 0), (0, 1), (0, 2), (1, 2), (2, 2)]), // V
        Omino::new(&[(0, 0), (0, 1), (1, 1), (1, 2), (2, 2)]), // W
        Omino::new(&[(1, 0), (0, 1), (1, 1), (2, 1), (1, 2)]), // X
        Omino::new(&[(2, 0), (0, 1), (1, 1), (2, 1), (3, 1)]), // Y
        Omino::new(&[(0, 0), (1, 0), (1, 1), (1, 2), (2, 2)]), // Z
    ]
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Omino {
    c: Vec<Cell>,
    xmin: Count,
    xmax: Count,
    ymin: Count,
    ymax: Count,
}

impl Omino {
    pub fn new(c: &[Cell]) -> Omino {
        let mut c = c.to_vec();
        c.sort();
        let (ymin, ymax) =
            c.iter()
                .fold((c[0].1, c[0].1), |(cur_min, cur_max), &item| {
                    (cur_min.min(item.1), cur_max.max(item.1))
                });
        let xmin = c[0].0;
        let xmax = c.last().unwrap().0;
        let mut o = Omino { c, xmin, xmax, ymin, ymax };
        o.normalize();
        o
    }

    pub fn all_options(ps: &[Omino], bx: &Shape) -> Vec<Vec<Count>> {
        let mut os = Vec::new();
        for (i, p) in ps.iter().enumerate() {
            for base in p.bases() {
                os.extend(base.options(Count(i), bx));
            }
        }
        os
    }

    fn normalize(&mut self) {
        for i in 0..self.c.len() {
            self.c[i] = (self.c[i].0 - self.xmin, self.c[i].1 - self.ymin);
        }
        self.xmax -= self.xmin;
        self.ymax -= self.ymin;
        self.xmin = 0;
        self.ymin = 0;
    }

    pub fn rotate(&self) -> Omino {
        Omino::new(
            &self
                .c
                .iter()
                .map(|(x, y)| (*y, self.xmax - x))
                .collect::<Vec<_>>(),
        )
    }

    pub fn reflect(&self) -> Omino {
        Omino::new(&self.c.iter().map(|(x, y)| (*y, *x)).collect::<Vec<_>>())
    }

    pub fn bases(&self) -> Vec<Omino> {
        let mut b = vec![self.clone()];
        for _ in 0..3 {
            let rotate = b.last().unwrap().rotate();
            if b.contains(&rotate) {
                break;
            }
            b.push(rotate);
        }
        let reflect = b[0].reflect();
        if !b.contains(&reflect) {
            b.push(reflect);
            for _ in 0..3 {
                let rotate = b.last().unwrap().rotate();
                if b.contains(&rotate) {
                    break;
                }
                b.push(rotate);
            }
        }
        // let mut j = 0;
        // while j < b.len() {
        //     let rotate = b[j].rotate();
        //     if !b.contains(&rotate) {
        //         b.push(rotate);
        //     }
        //     let reflect = b[j].reflect();
        //     if !b.contains(&reflect) {
        //         b.push(reflect);
        //     }
        //     j += 1;
        // }
        b
    }

    pub fn options(&self, name: Count, shape: &Shape) -> Vec<Vec<Count>> {
        self.options_within(
            name, shape.xmin, shape.ymin, shape.xmax, shape.ymax, shape,
        )
    }

    pub fn options_within(
        &self, name: Count, xmin: Count, ymin: Count, xmax: Count, ymax: Count,
        shape: &Shape,
    ) -> Vec<Vec<Count>> {
        // TODO? generalize to arbitrary subshapes
        let mut os = Vec::new();
        if self.ymax > ymax {
            return os;
        }
        for yd in ymin..=(ymax - self.ymax) {
            if self.xmax > xmax {
                continue;
            }
            for xd in xmin..=(xmax - self.xmax) {
                let mut cells = self
                    .c
                    .iter()
                    .map_while(|(x, y)| {
                        shape
                            .c
                            .iter()
                            .position(|&c| (c.0, c.1) == (x + xd, y + yd))
                            .map(|p| Count(p))
                    })
                    .collect::<Vec<_>>();
                if cells.len() == self.c.len() {
                    cells.push(name + Count(shape.c.len()));
                    os.push(cells);
                }
            }
        }
        os
    }

    pub fn cell_at(&self, idx: Count) -> (usize, usize) {
        let idx = idx as usize;
        (self.c[idx].0 as usize, self.c[idx].1 as usize)
    }

    pub fn size(&self) -> usize {
        self.c.len()
    }

    pub fn show(&self) {
        let mut grid =
            vec![vec![' '; (self.xmax + 1) as usize]; (self.ymax + 1) as usize];
        for c in &self.c {
            grid[c.1 as usize][c.0 as usize] = '█';
        }
        for line in &grid {
            println!("{}", line.iter().collect::<String>());
        }
    }
}
