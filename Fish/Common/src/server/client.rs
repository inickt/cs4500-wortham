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
    pub kicked: bool,

    /// This is the shared, mutable reference to the Client shared
    /// between the tournament manager and the referee components.
    pub client: Rc<RefCell<dyn Client>>,
}

impl ClientWithId {
    pub fn new(id: usize, client: Box<dyn Client>) -> ClientWithId {
        ClientWithId {
            id: PlayerId(id),
            kicked: false,
            client: Rc::new(RefCell::new(client)),
        }
    }

    pub fn borrow_mut(&self) -> std::cell::RefMut<'_, dyn Client + 'static> {
        self.client.borrow_mut()
    }
}

impl Client for Box<dyn Client> {
    fn tournament_starting(&mut self) -> Option<()> {
        self.as_mut().tournament_starting()
    }

    fn tournament_ending(&mut self, won: bool) -> Option<()> {
        self.as_mut().tournament_ending(won)
    }

    fn initialize_game(&mut self, initial_gamestate: &GameState, player_color: PlayerColor) -> Option<()> {
        self.as_mut().initialize_game(initial_gamestate, player_color)
    }

    fn get_placement(&mut self, gamestate: &GameState) -> Option<Placement> {
        self.as_mut().get_placement(gamestate)
    }

    fn get_move(&mut self, gamestate: &GameState, previous: &[PlayerMove]) -> Option<Move> {
        self.as_mut().get_move(gamestate, previous)
    }
}
