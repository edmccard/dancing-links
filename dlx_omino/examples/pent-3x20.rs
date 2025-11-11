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
    const PIECE_COUNT: usize = 12;
    const CELL_COUNT: usize = 60;
}

fn main() {
    let ps = pentominoes();
    let bx = rectangle(3, 20);
    let info = Info;

    let mut os = Vec::new();
    for (i, p) in ps.iter().enumerate() {
        if i == 7 {
            for o in p.transform(2)[0].all_options(7, &bx, &info) {
                os.push(o);
            }
        } else {
            for t in p.transform(255) {
                for o in t.all_options(Uint(i), &bx, &info) {
                    os.push(o);
                }
            }
        }
    }

    let mut problem = make_problem(72, 0, &os, OptOrder::Seq);
    let mut solver = Solver::new(&mut problem);
    let mut chooser = mrv_chooser(prefer_any(), no_tiebreak());

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("3x20 Pentominoes", 600, 300)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(255, 255, 255));
    canvas.clear();

    let x_off = 100;
    let mut y_off = 65;
    while solver.next_solution(&mut chooser) {
        let sol = solver.fmt_solution();
        let grid = ShapeGrid::from_solution(&sol, &info, &os, &bx);
        grid.draw(&mut canvas, 20, x_off, y_off, &PALETTE_12);
        y_off += 100;
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
                _ => {}
            }
        }
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
