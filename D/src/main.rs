use gio::prelude::*;
use gtk::prelude::*;
use gtk::DrawingArea;

use cairo::Context;
use std::cmp::max;

const LEFT_CLICK: u32 = 1;

type Hexagon = Vec<(f64, f64)>;

fn build_ui(application: &gtk::Application, hexagon: Hexagon) {
    draw(application, hexagon.clone(), move |_, cr| {
        cr.set_source_rgb(1.0, 0.0, 0.0);

        for (x, y) in hexagon.iter().copied() {
            cr.line_to(x, y);
        }

        cr.fill();
        Inhibit(false)
    });
}

fn bounds_check(points: &Hexagon, x: f64, y: f64) -> bool {
    let xs = points.iter().map(|&(x, _)| x).collect::<Vec<_>>();
    let ys = points.iter().map(|&(_, y)| y).collect::<Vec<_>>();

    let mut c = false;
    let indices = [5, 0, 1, 2, 3, 4, 5];

    for index in 1..indices.len() {
        let (i, j) = (indices[index], indices[index - 1]);

        if ((ys[i] > y) != (ys[j] > y)) && (x < (xs[j] - xs[i]) * (y - ys[i]) / (ys[j] - ys[i]) + xs[i]) {
            c = !c;
        }
    }

    c
}

fn on_click(hexagon: Hexagon) -> impl Fn(&gtk::ApplicationWindow, &gdk::EventButton) -> Inhibit {
    move |window, event| {
        if event.get_event_type() == gdk::EventType::ButtonPress && event.get_button() == LEFT_CLICK {
            let (x, y) = event.get_coords().unwrap();
            if bounds_check(&hexagon, x, y) {
                window.destroy();
            }
        }
        Inhibit(false)
    }
}

fn main() {
    let application = gtk::Application::new(None, Default::default())
        .expect("Initialization failed...");

    let scale = std::env::args().nth(1).unwrap()
        .parse::<u64>().unwrap() as f64;

    let points: Hexagon = [
        (0.0,  1.0), (1.0,  2.0), (2.0,  2.0),
        (3.0,  1.0), (2.0,  0.0), (1.0,  0.0),
    ].iter().map(|&(x, y)| (x * scale, y * scale)).collect(); // TODO make function

    application.connect_activate(move |app| {
        build_ui(app, points.clone());
    });

    application.run(&[]);
}

fn draw<F>(application: &gtk::Application, hexagon: Hexagon, draw_fn: F)
where
    F: Fn(&DrawingArea, &Context) -> Inhibit + 'static,
{
    let drawing_area = DrawingArea::new();
    drawing_area.connect_draw(draw_fn);

    let window = gtk::ApplicationWindow::new(application);
    let (width, height) = get_size(&hexagon);
    window.set_default_size(width, height);
    window.add(&drawing_area);
    window.connect_button_press_event(on_click(hexagon));
    window.show_all();
}

// Gets width and height of hexagon
fn get_size(h: &Hexagon) -> (i32, i32) {
    h.iter().fold((0, 0), |acc, element| {
        (max(acc.0, element.0 as i32), max(acc.1, element.1 as i32))
    })
}
