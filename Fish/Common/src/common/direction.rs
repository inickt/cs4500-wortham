use self::Direction::*;

// Represents a direction from a tile on the game board.
// Note that tiles do not have tiles directly to the North and South.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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


#[test]
fn opposite() {
    assert_eq!(Direction::opposite(Direction::Southeast), Direction::Northwest);
    assert_eq!(Direction::opposite(Direction::Northwest), Direction::Southeast);
    assert_eq!(Direction::opposite(Direction::Northeast), Direction::Southwest);
    assert_eq!(Direction::opposite(Direction::Southwest), Direction::Northeast);
    assert_eq!(Direction::opposite(Direction::West), Direction::East);
    assert_eq!(Direction::opposite(Direction::East), Direction::West);
}

#[test]
fn iter() {
    let direction_iter = Direction::iter();
    assert_eq!(direction_iter.size_hint(), (6, Some(6)));
    let direction_iter_collection : Vec<Direction> = direction_iter.collect();
    for dir in vec![
        Direction::Northeast, 
        Direction::Southeast,
        Direction::Northwest,
        Direction::Southwest,
        Direction::East,
        Direction::West
    ] {
        assert!(direction_iter_collection.contains(&dir));
    }
}