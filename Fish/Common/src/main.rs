mod client;
mod common;

fn main() {
    let gamestate = common::gamestate::new_gamestate(4, 3, 3);
    client::show_ui(gamestate);
}
