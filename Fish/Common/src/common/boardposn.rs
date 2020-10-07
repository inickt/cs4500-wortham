/// Represents the column and row of a tile on the game 
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct BoardPosn {
    pub x: u32,
    pub y: u32,
}

impl BoardPosn {
    pub fn new(x: u32, y: u32) -> BoardPosn {
        BoardPosn { x, y }
    }
}

impl From<(u32, u32)> for BoardPosn {
    fn from((x, y): (u32, u32)) -> BoardPosn {
        BoardPosn { x, y }
    }
}