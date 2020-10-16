//! The client module handles rendering the GameState as an interactable
//! window through which players can play the game. Currently this just
//! involves translating the Board to a series of images within a gtk::Application,
//! but in the future this will also handle player input and receiving server
//! updates in separate submodules within client.
use crate::common::tile::{ TileId, Tile };
use crate::common::gamestate::{ GameState, SharedGameState };
use crate::common::player::PlayerColor;
use crate::common::board::Board;
use crate::common::boardposn::BoardPosn;

use gdk_pixbuf::InterpType;
use gio::prelude::*;
use gtk::prelude::*;
use gtk::{ StateFlags, Image, Fixed };
use gdk::RGBA;

const FISH_FILENAME_TEMPLATE: &str = "assets/fish";
const HEXAGON_FILENAME: &str = "assets/hexagon.png";

const BLUE_PENGUIN_FILENAME: &str = "assets/penguin-blue.png";
const GREEN_PENGUIN_FILENAME: &str = "assets/penguin-green.png";
const PINK_PENGUIN_FILENAME: &str = "assets/penguin-pink.png";
const PURPLE_PENGUIN_FILENAME: &str = "assets/penguin-purple.png";

/// Text to display above the current turn player image.
const CURRENT_TURN_TEXT: &str = "Current Turn:";

/// Rough height of the current turn text, used to move the text above
/// the image of the player whose turn it currently is.
const CURRENT_TURN_TEXT_HEIGHT: i32 = 15;

/// Width and height of the current window in pixels.
const WINDOW_SIZE: (i32, i32) = (1600, 900);

/// Size of the "current turn" player image in pixels
const PLAYER_IMAGE_SIZE: (i32, i32) = (66, 100);

/// How many pixels from the bottom-right of the screen the "current turn"
/// player image is placed.
const PLAYER_IMAGE_MARGIN: (i32, i32) = (
    (PLAYER_IMAGE_SIZE.0 as f32 * 1.5) as i32,
    (PLAYER_IMAGE_SIZE.1 as f32 * 1.5) as i32
);

/// Creates a single gtk::Image containing 1-5 fish
/// This function will panic if given 0 fish.
/// If asked for > 5 fish, this function will return an image of only 5 fish.
fn make_fish_image(fish_count: u8) -> Image {
    assert_ne!(fish_count, 0);

    // Limit tiles to displaying a max of 5 fish - that is all we have images for.
    let fish_count = std::cmp::min(5, fish_count);
    let filename = format!("{}{}.png", FISH_FILENAME_TEMPLATE, fish_count);
    Image::new_from_file(filename)
}

/// Creates a single gtk::Image containing a penguin of the given color
fn get_penguin_image(color: PlayerColor, width: i32, height: i32) -> Image {
    let filename = match color {
        PlayerColor::Blue => BLUE_PENGUIN_FILENAME,
        PlayerColor::Green => GREEN_PENGUIN_FILENAME,
        PlayerColor::Pink => PINK_PENGUIN_FILENAME,
        PlayerColor::Purple => PURPLE_PENGUIN_FILENAME,
    };

    let pixbuf = Image::new_from_file(filename).get_pixbuf().unwrap();
    let scaled = pixbuf.scale_simple(width, height, InterpType::Hyper);
    Image::new_from_pixbuf(scaled.as_ref())
}

/// Adds the given Image to the layout, centering the image on the hexagonal tile.
fn add_image_centered_on_tile(layout: &Fixed, image: &Image, hexagon_size: (i32, i32)) {
    let image_size = get_image_size(image);
    layout.add(image);
    layout.move_(image,
        hexagon_size.0 / 2 - image_size.0 / 2,
        hexagon_size.1 / 2 - image_size.1 / 2);
}

/// Generates a GTK drawing of a specific Tile
/// Returns the drawing and a tuple of (width, height) in px of the tile
fn make_tile_layout(tile: &Tile, penguin_color: Option<PlayerColor>) -> (Fixed, (i32, i32)) {
    let layout = Fixed::new();
    let hexagon = Image::new_from_file(HEXAGON_FILENAME);
    let hexagon_size = get_image_size(&hexagon);
    layout.add(&hexagon);

    if let Some(color) = penguin_color {
        // Scale the large penguin image down to (1/4 of the tile width, 1/2 of the tile height)
        // This size is rather arbitrary, it was just picked since it looks decent and is small
        // enough to show the fish underneath the penguin.
        let penguin = get_penguin_image(color, hexagon_size.0 / 4, hexagon_size.1 / 2);
        add_image_centered_on_tile(&layout, &penguin, hexagon_size);
    }

    let fish_count = tile.get_fish_count();
    if fish_count > 0 {
        let fish = make_fish_image(fish_count);
        add_image_centered_on_tile(&layout, &fish, hexagon_size);
    }

    (layout, hexagon_size)
}

/// Gets the width and height of a gtk Image
/// Panics if image is not ImageType::Empty or ImageType::Pixbuf
fn get_image_size(img: &Image) -> (i32, i32) {
    img.get_pixbuf().map(|p| (p.get_width(), p.get_height())).unwrap()
}

/// Returns (x, y) tuple of position of tile in screen pixels where (0, 0)
/// is the top-left most point and (SCREEN_WIDTH, SCREEN_HEIGHT) is the bottom right.
fn get_tile_position_px(board: &Board, tile_id: TileId, (tile_width, tile_height): (i32, i32)) -> (i32, i32) {
    let BoardPosn { x: col, y: row } = board.get_tile_position(tile_id);
    let y = row as i32 * tile_height / 2;

    // odd rows are shifted an additional (2/3) to the right to interleave the hexagons in subsequent rows 
    let row_x_offset = if row % 2 != 0 { tile_width * 2 / 3 } else { 0 };
    let x = col as i32 * tile_width * 4 / 3 + row_x_offset;

    (x, y)
}

/// Creates a widget layout containing a penguin icon with the color of the current
/// player as well as a "current turn" text widget to indicate whose turn it is.
fn make_current_turn_widget(gamestate: &GameState) -> gtk::Fixed {
    let current_player = &gamestate.players[&gamestate.current_turn];
    let player_image = get_penguin_image(current_player.color, PLAYER_IMAGE_SIZE.0, PLAYER_IMAGE_SIZE.1);

    let layout = Fixed::new();
    layout.add(&player_image);
    layout.move_(&player_image,
                 WINDOW_SIZE.0 - PLAYER_IMAGE_MARGIN.0,
                 WINDOW_SIZE.1 - PLAYER_IMAGE_MARGIN.1);

    let text = gtk::TextView::new();
    let buffer = text.get_buffer().unwrap();
    buffer.set_text(CURRENT_TURN_TEXT);
    layout.add(&text);
    layout.move_(&text,
                 WINDOW_SIZE.0 - PLAYER_IMAGE_MARGIN.0,
                 WINDOW_SIZE.1 - PLAYER_IMAGE_MARGIN.1 - CURRENT_TURN_TEXT_HEIGHT);
    layout
}

/// Creates and displays a window in a given application displaying the given gamestate.
/// The window draws itself each frame and holds a copy of the gamestate. Resultingly,
/// any changes made to the shared gamestate will automatically be updated in the window
/// the next time it is redrawn.
fn make_window(application: &gtk::Application, gamestate: SharedGameState) {
    let window = gtk::ApplicationWindow::new(application);
    let layout = Fixed::new();

    window.override_background_color(StateFlags::NORMAL, Some(&RGBA::blue()));

    // Draw each board tile
    let gamestate_ref = gamestate.borrow();
    for (tile_id, tile) in gamestate_ref.board.tiles.iter() {
        let penguin_color_on_tile = gamestate_ref.get_color_on_tile(*tile_id);
        let (tile_layout, tile_layout_size) = make_tile_layout(tile, penguin_color_on_tile);
        layout.add(&tile_layout);
        let (new_x, new_y) = get_tile_position_px(&gamestate_ref.board, *tile_id, tile_layout_size);
        layout.move_(&tile_layout, new_x, new_y); // moves to absolute x/y pos
    }

    // Add an icon and text representing whose turn it is to the bottom-left.
    layout.add(&make_current_turn_widget(&gamestate_ref));

    window.set_default_size(WINDOW_SIZE.0, WINDOW_SIZE.1);
    window.add(&layout);
    window.show_all();
}

/// Builds and shows the client side UI for the game.
/// This takes care of window creation as well.
pub fn show_ui(gamestate: SharedGameState) {
    let application = gtk::Application::new(None, Default::default())
        .expect("Initialization failed...");

    application.connect_activate(move |app| {
        make_window(app, gamestate.clone());
    });

    application.run(&[]);
}
