use crate::common::tile::{ TileId, Tile };
use crate::common::gamestate::GameState;

use gio::prelude::*;
use gtk::prelude::*;
use gtk::{ Image, Fixed };

const FISH_FILENAME_TEMPLATE: &str = "assets/fish";
const HEXAGON_FILENAME: &str = "assets/hexagon.png";

fn make_fish_image(fish_count: u8) -> Image {
    let filename = format!("{}{}.png", FISH_FILENAME_TEMPLATE, fish_count);
    Image::new_from_file(filename)
}

/// Generates a GTK drawing of a specific hexagon
fn make_tile_layout(tile: &Tile) -> Fixed {
    let layout = Fixed::new();
    let hexagon = Image::new_from_file(HEXAGON_FILENAME);

    let fish = make_fish_image(tile.fish_count);
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
fn make_window(application: &gtk::Application, gamestate: GameState) {
    let window = gtk::ApplicationWindow::new(application);
    let layout = Fixed::new();

    let gamestate_ref = gamestate.borrow();
    for (tile_id, tile) in gamestate_ref.board.tiles.iter() {
        let tile_layout = make_tile_layout(tile);
        layout.add(&tile_layout);
        layout.move_(&tile_layout, (tile_id.0 * 200) as i32, (tile_id.0 * 100) as i32);
    }

    // TODO: remove hardcoded window width/height
    let (width, height) = (1600, 900);
    window.set_default_size(width, height);
    window.add(&layout);
    window.show_all();
}

pub fn show_ui(gamestate: GameState) {
    let application = gtk::Application::new(None, Default::default())
        .expect("Initialization failed...");

    application.connect_activate(move |app| {
        let gamestate = gamestate.clone();
        make_window(app, gamestate);
    });

    application.run(&[]);
}
