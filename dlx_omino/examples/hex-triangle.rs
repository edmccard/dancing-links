extern crate dlx;
extern crate sdl2;

use dlx::choose::*;
use dlx::x::make_problem;
use dlx::{OptOrder, Solver, Uint};

use dlx_omino::*;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::time::Duration;

struct Info;
impl SpecInfo for Info {
    type OData = Uint;
    const PIECE_COUNT: usize = 35;
    const CELL_COUNT: usize = 210;
}

fn main() {
    let ps = hexominoes();
    let bx = triangle();
    let info = Info;

    let mut os = Vec::new();
    for (i, p) in ps.iter().enumerate() {
        for t in p.transform(255) {
            for o in t.all_options(Uint(i), &bx, &info) {
                os.push(o);
            }
        }
    }

    let mut problem = make_problem(245, 0, &os, OptOrder::Seq);
    let mut solver = Solver::new(&mut problem);
    let mut chooser = mrv_chooser(prefer_any(), no_tiebreak());

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("3x20 Pentominoes", 340, 340)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(255, 255, 255));
    canvas.clear();

    if solver.next_solution(&mut chooser) {
        let sol = solver.fmt_solution();
        let grid = ShapeGrid::from_solution(&sol, &info, &os, &bx);
        grid.draw(&mut canvas, 15, 20, 20, &PALETTE_35);
    }

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        canvas.present();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running;
                }
                Event::KeyDown { keycode: Some(Keycode::Space), .. } => {
                    let _ = canvas.window_mut().set_opacity(0.5);
                    let nxt = solver.next_solution(&mut chooser);
                    if !nxt {
                        solver.next_solution(&mut chooser);
                    }
                    let sol = solver.fmt_solution();
                    let grid = ShapeGrid::from_solution(&sol, &info, &os, &bx);
                    let _ = canvas.window_mut().set_opacity(1.0);
                    grid.draw(&mut canvas, 15, 20, 20, &PALETTE_35);
                }
                _ => {}
            }
        }
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}

fn triangle() -> Shape {
    let mut cells = Vec::new();
    for y in 0..20 {
        for x in 0..y + 1 {
            cells.push((x, y));
        }
    }
    println!("CELL LEN {}", cells.len());
    Omino::new(&cells)
}

fn cells_to_cells(shape: &Shape) -> Vec<Vec<(usize, usize)>> {
    let mut cells = vec![
        vec![(0, 0); (shape.bounds().2 + 1) as usize];
        (shape.bounds().3 + 1) as usize
    ];
    for i in 0..shape.size() {
        let (x, y) = shape.cell_at(i);
        cells[y as usize][x as usize] = (1, 1);
    }
    cells
}
