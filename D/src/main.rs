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

fn is_mouse_in_hexagon(hexagon: &Hexagon, x: f64, y: f64) -> bool {
    let (_, height) = get_size(hexagon);
    let size = height as f64 / 2.0;

    // booleans representing different "inside of hexagon" cases

    // y <= size + x
    let bot_left = y <= size + x;
     
    // y >= size - x
    let top_left = y >= size - x;

    // y <= 4 * size - x
    let bot_right = y <= 4.0 * size - x;

    // y >= -2 * size + x
    let top_right = y >= -2.0 * size + x;

    bot_left && top_left && bot_right && top_right
}

fn on_click(hexagon: Hexagon) -> impl Fn(&gtk::ApplicationWindow, &gdk::EventButton) -> Inhibit {
    move |window, event| {
        if event.get_event_type() == gdk::EventType::ButtonPress && event.get_button() == LEFT_CLICK {
            let (x, y) = event.get_coords().unwrap();
            if is_mouse_in_hexagon(&hexagon, x, y) {
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
