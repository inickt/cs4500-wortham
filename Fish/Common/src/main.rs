mod client;
mod common;
mod server;

use common::board::Board;
use client::player::InHousePlayer;
use server::referee::run_game;
use server::serverplayer::Client;

use std::rc::Rc;
use std::cell::RefCell;

fn main() {
    let players = vec![
        Client::InHouseAI(InHousePlayer::with_zigzag_minmax_strategy()),
        Client::InHouseAI(InHousePlayer::with_zigzag_minmax_strategy()),
    ];

    let board = Board::with_no_holes(5, 3, 1);
    let state = run_game(players, Some(board));

    let state = Rc::new(RefCell::new(state));
    client::show_ui(state);
}
