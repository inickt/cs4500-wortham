use crate::common::tile::TileId;
use std::sync::atomic::{ AtomicUsize, Ordering };

/// Amount of penguins generated in the current instance of this program.
/// Used for setting unique PenguinIds for each penguin.
static TOTAL_PENGUIN_COUNT: AtomicUsize = AtomicUsize::new(0);

/// Id for a Penguin. First penguin uid is 1.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct PenguinId(usize);

/// Represents a single Penguin in the Fish game, including its position
/// on the board and a unique ID. Its position can be None, meaning
/// it is not placed yet, or Some(BoardPosn), meaning it's placed at
/// the BoardPosn on the game board.
pub struct Penguin {
    pub penguin_id: PenguinId,
    pub tile: Option<TileId>,
}

impl Penguin {
    /// Creates a new penguin with a unique PenguinId, starting at 1.
    /// The penguin is initially unplaced, represented with None
    /// as its BoardPosn.
    pub fn new() -> Penguin {
        let id = TOTAL_PENGUIN_COUNT.fetch_add(1, Ordering::SeqCst);

        Penguin {
            penguin_id: PenguinId(id),
            tile: None,
        }
    }
}