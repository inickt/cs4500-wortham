use gio::prelude::*;
use gtk::prelude::*;
use gtk::DrawingArea;
use std::cmp::max;
use crate::common::tile::Tile;

const LEFT_CLICK: u32 = 1;

#[derive(Clone)]
struct Hexagon(pub [(f64, f64); 6]);

impl Hexagon {
    fn new(size: f64) -> Hexagon {
        let mut default_hexagon: [(f64, f64); 6] = [
            (0.0,  1.0), (1.0,  2.0), (2.0,  2.0),
            (3.0,  1.0), (2.0,  0.0), (1.0,  0.0),
        ];

        for pair in default_hexagon.iter_mut() {
            pair.0 *= size;
            pair.1 *= size;
        };

        Hexagon(default_hexagon)
    }

    fn dimensions(&self) -> (i32, i32) {
        self.0.iter().fold((0, 0), |acc, element| {
            (max(acc.0, element.0 as i32), max(acc.1, element.1 as i32))
        })
    }

    // The scale with which the hexagon is created
    // Provided as a command line argument to the program
    fn size(&self) -> i32 {
        self.dimensions().1 / 2
    }
}

/// Generates a GTK drawing of a specific hexagon
fn make_hexagon_drawing(hexagon: Hexagon) -> DrawingArea {
    let drawing_area = DrawingArea::new();

    // draw lines (edges) between hexagon vertices, and color inside edges
    drawing_area.connect_draw(move |_, context| {
        context.set_source_rgb(1.0, 0.0, 0.0);

        for (x, y) in hexagon.0.iter().copied() {
            context.line_to(x, y);
        }

        context.fill();
        Inhibit(false)
    });
    
    drawing_area
}

/// Makes and shows a window in a given application displaying the given hexagon
fn make_window(application: &gtk::Application, hexagon: Hexagon) {
    let window = gtk::ApplicationWindow::new(application);
    let drawing_area = make_hexagon_drawing(hexagon.clone());
    let (width, height) = hexagon.dimensions();
    window.set_default_size(width, height);
    window.add(&drawing_area);
    window.show_all();
}

pub fn show_tile(tile: &Tile) {
    let application = gtk::Application::new(None, Default::default())
        .expect("Initialization failed...");

    let hexagon = Hexagon::new(50.0);

    application.connect_activate(move |app| {
        make_window(app, hexagon.clone());
    });

    application.run(&[]);
}