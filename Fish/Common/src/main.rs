mod client;
mod common;
mod server;

use common::board::Board;
use server::ai_client::AIClient;
use server::referee::run_game;
use server::client::Client;

use std::rc::Rc;
use std::cell::RefCell;

fn main() {
    let players = vec![
        Box::new(AIClient::with_zigzag_minmax_strategy()) as Box<dyn Client>,
        Box::new(AIClient::with_zigzag_minmax_strategy()),
    ];

    let board = Board::with_no_holes(5, 3, 1);
    let result = run_game(players, Some(board));

    let state = Rc::new(RefCell::new(result.final_state));
    client::show_ui(state);
}
