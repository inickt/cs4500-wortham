use gio::prelude::*;
use gtk::prelude::*;
use gtk::DrawingArea;
use std::cmp::max;

const LEFT_CLICK: u32 = 1;

#[derive(Clone)]
struct Hexagon(pub Vec<(f64, f64)>);

impl Hexagon {
    fn get_size(&self) -> (i32, i32) {
        self.0.iter().fold((0, 0), |acc, element| {
            (max(acc.0, element.0 as i32), max(acc.1, element.1 as i32))
        })
    }
}

fn is_mouse_in_hexagon(hexagon: &Hexagon, x: f64, y: f64) -> bool {
    let (_, height) = hexagon.get_size();
    let size = height as f64 / 2.0;

    // booleans representing different "inside of hexagon" cases
    let bot_left = y <= size + x;
    let top_left = y >= size - x;
    let bot_right = y <= 4.0 * size - x;
    let top_right = y >= -2.0 * size + x;
    let vertical_bounds = (y <= 2.0 * size) && (y >= 0.0);

    bot_left && top_left && bot_right && top_right && vertical_bounds
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

    let hexagon = Hexagon([
        (0.0,  1.0), (1.0,  2.0), (2.0,  2.0),
        (3.0,  1.0), (2.0,  0.0), (1.0,  0.0),
    ].iter().map(|&(x, y)| (x * scale, y * scale)).collect()); // TODO make function

    application.connect_activate(move |app| {
        build_ui(app, hexagon.clone());
    });

    application.run(&[]);
}

fn build_ui(application: &gtk::Application, hexagon: Hexagon) {
    let drawing_area = DrawingArea::new();

    let hexagon_copy = hexagon.clone();
    drawing_area.connect_draw(move |_, context| {
        context.set_source_rgb(1.0, 0.0, 0.0);

        for (x, y) in hexagon_copy.0.iter().copied() {
            context.line_to(x, y);
        }

        context.fill();
        Inhibit(false)
    });

    let window = gtk::ApplicationWindow::new(application);
    let (width, height) = hexagon.get_size();
    window.set_default_size(width, height);
    window.add(&drawing_area);
    window.connect_button_press_event(on_click(hexagon));
    window.show_all();
}
