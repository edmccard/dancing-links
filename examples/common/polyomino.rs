use dlx::Count;

type Coord = Count;
type Cell = (Count, Count);

type Shape = Omino;

fn rectangle(rows: usize, cols: usize) -> Shape {
    let c = (0..rows)
        .flat_map(|i| (0..cols).map(move |j| (j as Coord, i as Coord)))
        .collect();
    Shape {
        c,
        xmin: 0,
        ymin: 0,
        xmax: (cols - 1) as Coord,
        ymax: (rows - 1) as Coord,
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
    xmin: Coord,
    xmax: Coord,
    ymin: Coord,
    ymax: Coord,
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

    fn normalize(&mut self) {
        for i in 0..self.c.len() {
            self.c[i] = (self.c[i].0 - self.xmin, self.c[i].1 - self.ymin);
        }
        self.xmax -= self.xmin;
        self.ymax -= self.ymin;
        self.xmin = 0;
        self.ymin = 0;
    }

    fn rotate(&self) -> Omino {
        Omino::new(
            &self
                .c
                .iter()
                .map(|(x, y)| (*y, self.xmax - x))
                .collect::<Vec<_>>(),
        )
    }

    fn reflect(&self) -> Omino {
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

    pub fn options(&self, name: Coord, shape: &Shape) -> Vec<Vec<Coord>> {
        self.options_within(
            name, shape.xmin, shape.ymin, shape.xmax, shape.ymax, shape,
        )
    }

    pub fn options_within(
        &self, name: Coord, xmin: Coord, ymin: Coord, xmax: Coord, ymax: Coord,
        shape: &Shape,
    ) -> Vec<Vec<Coord>> {
        // TODO? generalize to arbitrary subshapes
        let mut os = Vec::new();
        for yd in ymin..=(ymax - self.ymax) {
            for xd in xmin..=(xmax - self.xmax) {
                let mut cells = self
                    .c
                    .iter()
                    .map_while(|(x, y)| {
                        shape
                            .c
                            .iter()
                            .position(|&c| (c.0, c.1) == (x + xd, y + yd))
                            .map(|p| p as Coord)
                    })
                    .collect::<Vec<_>>();
                if cells.len() == self.c.len() {
                    cells.push(name + shape.c.len() as Coord);
                    os.push(cells);
                }
            }
        }
        os
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
