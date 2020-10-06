use gio::prelude::*;
use gtk::prelude::*;
use gtk::{ Image, Fixed };
use std::cmp::max;
use crate::common::tile::Tile;

const LEFT_CLICK: u32 = 1;
const FISH_FILENAME_TEMPLATE: &str = "assets/fish";
const HEXAGON_FILENAME: &str = "assets/hexagon.png";

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

fn make_fish_image(fish_count: u8) -> Image {
    let filename = format!("{}{}.png", FISH_FILENAME_TEMPLATE, fish_count);
    Image::new_from_file(filename)
}

/// Generates a GTK drawing of a specific hexagon
fn make_hexagon_layout(hexagon: Hexagon) -> gtk::Fixed {
    let layout = gtk::Fixed::new();
    let hexagon = Image::new_from_file(HEXAGON_FILENAME);

    let fish = make_fish_image(2);
    layout.add(&hexagon);
    layout.add(&fish);

    // TODO: Remove constants
    // Both hexagon.get_allocated() and hexagon.size_request are (0, 0) for some reason.
    let hexagon_size = (300, 200);
    let fish_size = (71, 63);

    // Center the fish on the hexagon
    layout.move_(&fish,
        hexagon_size.0 / 2 - fish_size.0 / 2,
        hexagon_size.1 / 2 - fish_size.1 / 2);

    layout
}

/// Makes and shows a window in a given application displaying the given hexagon
fn make_window(application: &gtk::Application, hexagon: Hexagon) {
    let window = gtk::ApplicationWindow::new(application);
    let layout = make_hexagon_layout(hexagon.clone());
    let (width, height) = hexagon.dimensions();
    window.set_default_size(width, height);
    window.add(&layout);
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
