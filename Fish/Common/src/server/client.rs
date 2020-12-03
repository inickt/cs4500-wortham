use crate::common::action::{ Placement, Move, PlayerMove };
use crate::common::gamestate::GameState;
use crate::common::player::{ PlayerColor, PlayerId };

use std::cell::RefCell;
use std::rc::Rc;

pub trait Client {
    // TODO documentation
    fn tournament_starting(&mut self) -> Option<()>;
    fn tournament_ending(&mut self, won: bool) -> Option<()>;

    fn initialize_game(&mut self, initial_gamestate: &GameState, player_color: PlayerColor) -> Option<()>;
    fn get_placement(&mut self, gamestate: &GameState) -> Option<Placement>;
    fn get_move(&mut self, gamestate: &GameState, previous: &[PlayerMove]) -> Option<Move>;
}

/// Represents the client's connection info along with an
/// id to identify that particular client across all tournament games.
#[derive(Clone)]
pub struct ClientWithId {
    pub id: PlayerId,

    /// This is the shared, mutable reference to the Client shared
    /// between the tournament manager and the referee components.
    pub client: Rc<RefCell<dyn Client>>,
}

impl ClientWithId {
    pub fn new<C: 'static + Client>(id: usize, client: C) -> ClientWithId {
        ClientWithId {
            id: PlayerId(id),
            client: Rc::new(RefCell::new(client)),
        }
    }
}
