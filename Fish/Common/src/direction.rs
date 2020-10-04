use self::Direction::*;

// Represents a direction from a tile on the game board.
// Note that tiles do not have tiles directly to the North and South.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Direction {
    Northeast,
    Northwest,
    East,
    West,
    Southeast,
    Southwest
}

impl Direction {
    pub fn iter() -> impl Iterator<Item = Direction> {
        vec![
            Northeast,
            Northwest,
            East,
            West,
            Southeast,
            Southwest
        ].into_iter()
    }

    pub fn opposite(self) -> Direction {
        match self {
            Northeast => Southwest,
            Northwest => Southeast,
            East => West,
            West => East,
            Southeast => Northwest,
            Southwest => Northeast,
        }
    }
}