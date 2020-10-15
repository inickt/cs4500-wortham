use std::collections::HashSet;

use serde_json::Deserializer;
use serde::Deserialize;

use fish::common::board::Board;
use fish::common::boardposn::BoardPosn;

/// Represents the input given to the xboard program through the command line
/// Exists to provide a data representation of the JSON string passed in
#[derive(Deserialize)]
struct JSONBoardAndPosn {
    position: [u32; 2],
    board: Vec<Vec<u32>>,
}

impl JSONBoardAndPosn {
    // Deserializes a JSON string into a JSONBoardAndPosn
    // Assumes the reader will contain only valid JSON
    pub fn from_reader<R: std::io::Read>(reader: R) -> JSONBoardAndPosn {
        let mut de = Deserializer::from_reader(reader);
        JSONBoardAndPosn::deserialize(&mut de).ok().unwrap()
    }
}

fn board_from_json(board_and_posn: JSONBoardAndPosn) -> Board {
    let board = board_and_posn.board;
    let rows = board.len();
    let columns = board.get(0).map_or(0, |columns| columns.len());
    let mut holes = Vec::new();

    for (row_i, row) in board.iter().enumerate() {
        for (col_i, &num_fish) in row.iter().enumerate() {
            if num_fish == 0 {
                holes.push(BoardPosn::from((col_i as u32, row_i as u32)));
            }
        }
    }

    Board::with_holes(rows as u32, columns as u32, holes, 0)
}

fn main() {
    let stdin = std::io::stdin();
    let json_board_posn = JSONBoardAndPosn::from_reader(stdin.lock());
    let starting_position = json_board_posn.position;
    let board = board_from_json(json_board_posn);

    let starting_tile = board.get_tile(
        starting_position[1] as u32,
        starting_position[0] as u32
    ).unwrap();

    // print the number of reachable tiles with an empty set of 
    println!("{}", starting_tile.all_reachable_tiles(&board, &HashSet::new()).len())
}
