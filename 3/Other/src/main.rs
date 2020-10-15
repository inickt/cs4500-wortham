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
    board: Vec<Vec<u32>>, // vector of rows
}

impl JSONBoardAndPosn {
    /// Deserializes a JSON string into a JSONBoardAndPosn
    /// Assumes the reader will contain only valid JSON
    pub fn from_reader<R: std::io::Read>(reader: R) -> JSONBoardAndPosn {
        let mut de = Deserializer::from_reader(reader);
        JSONBoardAndPosn::deserialize(&mut de).ok().unwrap()
    }
}

/// Converts a JSON representation of a board to
/// the board module's Board representation.
fn board_from_json(board_and_posn: JSONBoardAndPosn) -> Board {
    let board = board_and_posn.board;
    let rows = board.len();
    let columns = board.iter().map(|row| row.len()).max().unwrap_or(0);
    let mut holes = Vec::new();

    for (row_i, row) in board.iter().enumerate() {
        for (col_i, &num_fish) in row.iter().enumerate() {
            if num_fish == 0 {
                holes.push(BoardPosn::from((col_i as u32, row_i as u32)));
            }
        }

        // Boards may not contain an equal number of columns in each row,
        // push the remains of any smaller rows as holes
        for col_i in row.len() .. columns {
            holes.push(BoardPosn::from((col_i as u32, row_i as u32)));
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

#[test]
fn test_board_from_json() {
    let input = JSONBoardAndPosn {
        position: [0, 2],
        board: vec![
            vec![2, 3, 1],
            vec![0, 2, 3],
            vec![1, 1, 0]
        ]
    };

    let position = input.position;

    let expected = Board::with_holes(3, 3, vec![(0,1).into(), (2,2).into()], 0);
    let output = board_from_json(input);

    assert_eq!(output.tiles.len(), 7); // 9 tiles - 2 holes
    assert_eq!(expected.tiles, output.tiles);

    let expected_tile = expected.get_tile(position[1] as u32, position[0] as u32).unwrap();
    let output_tile = output.get_tile(position[1] as u32, position[0] as u32).unwrap();

    assert_eq!(expected_tile, output_tile);
    assert_eq!(output_tile.all_reachable_tiles(&output, &HashSet::new()).len(), 3);
}

#[test]
fn test_board_from_json_uneven_rows() {
    let input = JSONBoardAndPosn {
        position: [1, 2],
        board: vec![
            vec![1, 2, 3, 3, 4],
            vec!  [4, 0, 5, 2],
            vec![1,  1, 0],
        ]
    };

    let position = input.position;

    let expected = Board::with_holes(3, 5, vec![
        (1,1).into(),
        (2,2).into(),
        (4,1).into(),
        (3,2).into(),
        (4,2).into(),
    ], 0);
    let output = board_from_json(input);

    assert_eq!(output.tiles.len(), 10); // 12 tiles - 2 holes
    assert_eq!(expected.tiles, output.tiles);

    let expected_tile = expected.get_tile(position[1] as u32, position[0] as u32).unwrap();
    let output_tile = output.get_tile(position[1] as u32, position[0] as u32).unwrap();

    assert_eq!(expected_tile, output_tile);
    assert_eq!(output_tile.all_reachable_tiles(&output, &HashSet::new()).len(), 2);
}
