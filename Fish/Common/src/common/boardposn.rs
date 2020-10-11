//! The boardposn module contains utilities for working with
//! BoardPosns - an x, y pair for interacting with the board.
//!
//! Among other things, BoardPosn is useful to help differentiate
//! arbitrary x, y screen positions from x, y board positions when
//! writing function signatures (see Board::with_holes for an example).

/// Represents the x and y position of a tile on the game,
/// in row (y) and column (x) index (NOT px) starting at 0
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct BoardPosn {
    pub x: u32,
    pub y: u32,
}

impl From<(u32, u32)> for BoardPosn {
    /// A BoardPosn can be made from a (u32, u32) tuple. For example:
    /// `BoardPosn::from((1, 2))` or `(1, 2).into()`
    fn from((x, y): (u32, u32)) -> BoardPosn {
        BoardPosn { x, y }
    }
}
