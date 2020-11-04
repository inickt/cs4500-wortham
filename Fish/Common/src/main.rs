mod client;
mod common;
mod server;

use common::board::Board;
use client::player::InHousePlayer;
use server::referee::{ Player, run_game };

use std::rc::Rc;
use std::cell::RefCell;

fn main() {
    let players = vec![
        Player::InHouseAI(InHousePlayer::with_zigzag_minmax_strategy()),
        Player::InHouseAI(InHousePlayer::with_zigzag_minmax_strategy()),
    ];

    let board = Board::with_no_holes(5, 3, 1);
    let state = run_game(players, Some(board));

    let state = Rc::new(RefCell::new(state));
    client::show_ui(state);
}
