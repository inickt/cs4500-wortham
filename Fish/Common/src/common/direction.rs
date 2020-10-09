//! The direction module contains the Direction type and utilities to
//! operate on it. A Direction is an enumeration of the 6 possible
//! directions that can be moved from each hexagonal tile. Directions
//! are commonly used in the tile module to access tile neighbors.
use self::Direction::*;

/// Represents a direction from a hexagonal tile on the game board.
/// Note that tiles do not have tiles directly to the East or West.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Direction {
    Northeast,
    Northwest,
    North,
    South,
    Southeast,
    Southwest
}

impl Direction {
    pub fn iter() -> impl ExactSizeIterator<Item = Direction> {
        vec![
            Northeast,
            Northwest,
            North,
            South,
            Southeast,
            Southwest
        ].into_iter()
    }

    pub fn opposite(self) -> Direction {
        match self {
            Northeast => Southwest,
            Northwest => Southeast,
            North => South,
            South => North,
            Southeast => Northwest,
            Southwest => Northeast,
        }
    }
}


#[test]
fn test_opposite() {
    assert_eq!(Direction::opposite(Northeast), Southwest);
    assert_eq!(Direction::opposite(Northwest), Southeast);
    assert_eq!(Direction::opposite(North), South);
    assert_eq!(Direction::opposite(South), North);
    assert_eq!(Direction::opposite(Southeast), Northwest);
    assert_eq!(Direction::opposite(Southwest), Northeast);
}

#[test]
fn test_iter() {
    let direction_iter = Direction::iter();
    assert_eq!(direction_iter.len(), 6);
    let direction_iter_collection : Vec<Direction> = direction_iter.collect();
    for dir in &[Northeast, Northwest, North, South, Southeast, Southwest] {
        assert!(direction_iter_collection.contains(dir));
    }
}
