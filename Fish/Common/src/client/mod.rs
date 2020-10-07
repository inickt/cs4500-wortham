use crate::common::tile::{ TileId, Tile };
use crate::common::gamestate::GameState;
use crate::common::board::Board;
use crate::common::boardposn::BoardPosn;

use gio::prelude::*;
use gtk::prelude::*;
use gtk::{ Image, Fixed };

const FISH_FILENAME_TEMPLATE: &str = "assets/fish";
const HEXAGON_FILENAME: &str = "assets/hexagon.png";

const WINDOW_WIDTH: i32 = 1600;
const WINDOW_HEIGHT: i32 = 900;

fn make_fish_image(fish_count: u8) -> Image {
    let filename = format!("{}{}.png", FISH_FILENAME_TEMPLATE, fish_count);
    Image::new_from_file(filename)
}

/// Generates a GTK drawing of a specific Tile
/// Returns the drawing and a tuple of (width, height) in px of the tile
fn make_tile_layout(tile: &Tile) -> (Fixed, (i32, i32)) {
    let layout = Fixed::new();
    let hexagon = Image::new_from_file(HEXAGON_FILENAME);

    let fish = make_fish_image(tile.fish_count);
    layout.add(&hexagon);
    layout.add(&fish);

    let hexagon_size = get_image_size(&hexagon);
    let fish_size = get_image_size(&fish);

    // Center the fish on the hexagon
    layout.move_(&fish,
        hexagon_size.0 / 2 - fish_size.0 / 2,
        hexagon_size.1 / 2 - fish_size.1 / 2);

    (layout, hexagon_size)
}

/// Gets the width and height of a gtk Image
/// Panics if image is not ImageType::Empty or ImageType::Pixbuf
fn get_image_size(img: &Image) -> (i32, i32) {
    img.get_pixbuf().map(|p| (p.get_width(), p.get_height())).unwrap()
}

/// Returns (x, y) tuple of position of tile in
fn get_tile_position_px(board: &Board, tile_id: TileId, (tile_width, tile_height): (i32, i32)) -> (i32, i32) {
    let BoardPosn { x: col, y: row } = board.get_tile_position(tile_id);
    dbg!(tile_id, col, row);
    let y = row as i32 * tile_height / 2;

    // odd rows are shifted an additional (2/3) to the right to interleave the hexagons in subsequent rows 
    let row_x_offset = if row % 2 != 0 { tile_width * 2 / 3 } else { 0 };
    let x = col as i32 * tile_width * 4 / 3 + row_x_offset;

    (x, y)
}

/// Makes and shows a window in a given application displaying the givn gamestate
fn make_window(application: &gtk::Application, gamestate: GameState) {
    let window = gtk::ApplicationWindow::new(application);
    let layout = Fixed::new();

    let gamestate_ref = gamestate.borrow();
    for (tile_id, tile) in gamestate_ref.board.tiles.iter() {
        let (tile_layout, tile_layout_size) = make_tile_layout(tile);
        layout.add(&tile_layout);
        let (new_x, new_y) = get_tile_position_px(&gamestate_ref.board, *tile_id, tile_layout_size);
        layout.move_(&tile_layout, new_x, new_y); // moves to absolute x/y pos
    }

    window.set_default_size(WINDOW_WIDTH, WINDOW_HEIGHT);
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
