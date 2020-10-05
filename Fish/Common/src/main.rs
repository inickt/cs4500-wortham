mod client;
mod common;

fn main() {
    let tile = common::tile::Tile::new(0, 3);
    client::show_tile(&tile);
}