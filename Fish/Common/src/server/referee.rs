//! This file contains all logic and data regarding the Referee component,
//! which runs complete games of Fish. To do this, it starts and runs the
//! game loop, sending the gamestate to all players each turn then retrieving
//! a player's move and validating it until the game is over.
use crate::common::action::Action;
use crate::common::board::Board;
use crate::common::gamestate::GameState;
use crate::common::gamephase::GamePhase;
use crate::common::game_tree::GameTree;
use crate::common::player::PlayerId;

use crate::server::serverclient::ClientProxy;

use std::cell::RefCell;
use std::rc::Rc;

/// A referee is in charge of starting, running, and managing a game of fish.
/// This entails looping until the game is over and on each turn sending the
/// full gamestate to all player's then getting the action of the current
/// player and validating it. If this action is invalid the player is kicked,
/// otherwise the placement/move is made.
///
/// There is expected to be 1 referee per game of fish.
/// 
/// The referee will kick clients who do any of the following: 
/// 1. Send a well-formed but illegal placement to the referee
/// 2. Send a well-formed but illegal move to the referee
/// 3. Send non-well-formed JSON data to the Referee
/// 4. [Future] Take more than 30 seconds to send their move on their turn
struct Referee {
    /// Client input/output stream data, indexed on GameState's PlayerId.
    /// This Vec is in turn_order for each player.
    /// Each player is an Rc<RefCell<Client>> because the clients are mutably
    /// shared with the tournament component.
    clients: Vec<(PlayerId, Rc<RefCell<ClientProxy>>)>,

    /// The state of current game, separated by the current phase it is in.
    phase: GamePhase,
}

/// The final GameState of a finished game, along with each player and
/// whether they won, lost, or were kicked.
pub struct GameResult {
    /// This list is in the same order and of the same length
    /// as the Referee's original clients list and turn_order. So, each entry
    /// directly corresponds to the game outcome for a particular player.
    pub final_statuses: Vec<ClientStatus>,

    /// This is the final state of the game, which may be used to delve
    /// into statistics detail about each player, such as their score
    /// and end positions.
    pub final_state: GameState
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ClientStatus {
    Won,
    Lost,
    Kicked
}

/// Runs a complete game of Fish, setting up the board and
/// waiting for player input for gameplay to occur, and terminating
/// when a player (or multiple) have won. Check out Planning/player-protocol.md
/// for more information on the Fish game.
/// 
/// Clients will know the game has started when the referee sends each player
/// the initial game state before the first turn.
/// 
/// Returns the Win,Loss,Kicked status of each player and the final GameState
pub fn run_game(clients: Vec<ClientProxy>, board: Option<Board>) -> GameResult {
    let clients: Vec<_> = clients.into_iter()
        .map(|player| Rc::new(RefCell::new(player))).collect();
    run_game_shared(&clients, board)
}

/// Runs a game with a Vec of mutably shared clients so that clients
/// isn't consumed when the game is over.
///
/// Runs a complete game of Fish, setting up the board and
/// waiting for player input for gameplay to occur, and terminating
/// when a player (or multiple) have won. Check out Planning/player-protocol.md
/// for more information on the Fish game.
/// 
/// Clients will know the game has started when the referee sends each player
/// the initial game state before the first turn.
/// 
/// Returns the Win,Loss,Kicked status of each player and the final GameState
pub fn run_game_shared(clients: &[Rc<RefCell<ClientProxy>>], board: Option<Board>) -> GameResult {
    let board = board.unwrap_or(Board::with_no_holes(5, 5, 3));
    let mut referee = Referee::new(clients.to_vec(), board);

    while !referee.is_game_over() {
        referee.send_gamestate_to_all_clients();
        referee.do_player_turn();
    }

    referee.get_game_result()
}

impl Referee {
    fn new(clients: Vec<Rc<RefCell<ClientProxy>>>, board: Board) -> Referee {
        let state = GameState::new(board, clients.len());
        let clients = state.turn_order.iter().copied().zip(clients.into_iter()).collect();
        let phase = GamePhase::PlacingPenguins(state);
        Referee { clients, phase }
    }

    /// Returns the winners, losers, and kicked players of the game, along
    /// with the final game state of the game.
    /// 
    /// Assumes that the game this referee was hosting has been played to
    /// completion - otherwise no winners will be returned.
    fn get_game_result(self) -> GameResult {
        let Referee { clients, phase } = self;

        let final_statuses = clients.into_iter().map(|(id, client)| {
            if client.borrow().is_kicked() {
                ClientStatus::Kicked
            } else if phase.get_state().winning_players.as_ref()
                    .map_or(false, |winning_players| winning_players.contains(&id)) {

                ClientStatus::Won
            } else {
                ClientStatus::Lost
            }
        }).collect();

        GameResult {
            final_state: phase.take_state(),
            final_statuses,
        }
    }
    
    /// Sends the serialized gamestate to each output stream in self.clients
    /// If there was any error writing to any player, the referee assumes that
    /// player has disconnected and kicks them from the game, removing their penguins.
    fn send_gamestate_to_all_clients(&mut self) {
        let mut disconnected_clients = vec![];
        for (player_id, player) in self.clients.iter_mut() {
            let serialized = serde_json::to_string(&self.phase.get_state()).unwrap();

            // Write to the player and if there was an error in doing so, kick them.
            if let Err(_) = player.borrow_mut().send(serialized.as_bytes()) {
                disconnected_clients.push(*player_id);
            }
        }

        for player_id in disconnected_clients {
            self.kick_player(player_id);
        }
    }

    /// Waits for input from the current player in the GameState,
    /// then acts upon that input
    fn do_player_turn(&mut self) {
        let success = match &self.phase {
            GamePhase::Starting => Some(()),
            GamePhase::PlacingPenguins(_) => self.do_player_placement(),
            GamePhase::MovingPenguins(_) => self.do_player_move(),
            GamePhase::Done(_) => Some(()),
        };

        if success.is_none() {
            self.kick_current_player();
        }

        self.update_gamephase_if_needed();
    }

    /// Retrieve a player's next placement from their input stream then tries to take that placement.
    /// If the placement cannot be received from the input stream (e.g. due to a timeout) or the
    /// placement is invalid in any way then None will be returned. Otherwise, Some is returned.
    /// 
    /// Invariant: If None is returned then the current_turn does not change.
    fn do_player_placement(&mut self) -> Option<()> {
        let placement = self.get_player_action()?.as_placement()?;

        match &mut self.phase {
            GamePhase::PlacingPenguins(gamestate) => gamestate.place_avatar_for_current_player(placement),
            _ => unreachable!("do_player_placement called outside of the PlacingPenguins phase"),
        }
    }

    /// Retrieve a player's next move from their input stream then try to take that move.
    /// If the move is invalid in any way or if the move cannot be parsed from the input
    /// stream (e.g. if the stream timeouts) then None is returned. Otherwise Some is returned.
    /// 
    /// Invariant: If None is returned then the current_turn does not change.
    fn do_player_move(&mut self) -> Option<()> {
        let move_ = self.get_player_action()?.as_move()?;

        match &mut self.phase {
            GamePhase::MovingPenguins(gametree) => {
                let tree = gametree.get_game_after_move(move_)?;
                let state = tree.get_state().clone();
                self.phase.update_from_gamestate(state);
                Some(())
            },
            _ => unreachable!("do_player_move called outside of the MovingPenguins phase"),
        }
    }

    /// Retrieves the Action of the player whose turn it currently is.
    fn get_player_action(&mut self) -> Option<Action> {
        let current_player_id = &self.phase.current_turn();
        let current_player = &mut self.clients.iter_mut().find(|(id, _)| id == current_player_id)?.1;
        current_player.borrow_mut().get_action()
    }

    /// Kick the given player from the game, removing all their penguins and
    /// their position in the turn order. This does not notify the player that
    /// they were kicked.
    fn kick_player(&mut self, player: PlayerId) {
        self.phase.get_state_mut().remove_player(player);
        self.clients.iter_mut()
            .find(|(id, _)| *id == player)
            .map(|(_, client)| *client.borrow_mut() = ClientProxy::Kicked);

        // Must manually update after kicking a player to update the tree of valid moves in the game
        // tree, if needed
        self.phase.update_from_gamestate(self.phase.get_state().clone());

        // The game ends early if all clients are kicked
        if self.clients.iter().all(|(_, client)| client.borrow().is_kicked()) {
            self.phase = GamePhase::Done(self.phase.get_state().clone());
        }
    }

    /// Kick the player whose turn it currently is. See kick_player for
    /// the details of kicking a player.
    fn kick_current_player(&mut self) {
        let current_player = self.phase.get_state().current_turn;
        self.kick_player(current_player);
    }

    /// Player placements and moves will update the current
    /// GameState/GameTree but we still need to check if we've
    /// finished the placement/moves phase and update the current
    /// GamePhase as appropriate here.
    fn update_gamephase_if_needed(&mut self) {
        if let GamePhase::PlacingPenguins(state) = &mut self.phase {
            if state.all_penguins_are_placed() {
                self.phase = GamePhase::MovingPenguins(GameTree::new(state));
            }
        }

        // Test if MovingPenguins is finished even after testing the above in case we
        // start a game after placing penguins where immediately no penguin can move.
        if let GamePhase::MovingPenguins(GameTree::End(state)) = &self.phase {
            self.phase = GamePhase::Done(state.clone());
        }
    }

    /// Is this referee's game over?
    fn is_game_over(&self) -> bool {
        self.phase.is_game_over()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::ClientStatus::*;
    use crate::client::player::InHousePlayer;
    use crate::client::strategy::Strategy;
    use crate::common::action::{ Move, Placement };
    use crate::common::tile::TileId;
    use crate::common::penguin::PenguinId;

    pub struct CheatingStrategy;

    impl Strategy for CheatingStrategy {
        fn find_placement(&mut self, _gamestate: &GameState) -> Placement {
            Placement::new(TileId(0))
        }

        fn find_move(&mut self, _game: &mut GameTree) -> Move {
            Move::new(PenguinId(0), TileId(0))
        }
    }

    /// Runs a game where the first player should win if they're looking ahead enough
    /// turns. For more info on this specific game, see the explanation in
    /// client/strategy.rs, fn test_move_penguin_minmax_lookahead
    #[test]
    fn run_game_normal() {
        // set up players
        let players = vec![
            ClientProxy::InHouseAI(InHousePlayer::with_zigzag_minmax_strategy()),
            ClientProxy::InHouseAI(InHousePlayer::with_zigzag_minmax_strategy()),
        ];

        let board = Board::with_no_holes(3, 5, 1);
        let result = run_game(players, Some(board));
        assert!(result.final_state.is_game_over());
        assert_eq!(result.final_statuses, vec![Won, Lost]);
    }

    /// Runs a game that should start with no possible player moves, although
    /// they can each place all of their penguins.
    #[test]
    fn run_game_initially_over() {
        // set up players
        let players = vec![
            ClientProxy::InHouseAI(InHousePlayer::with_zigzag_minmax_strategy()),
            ClientProxy::InHouseAI(InHousePlayer::with_zigzag_minmax_strategy()),
        ];

        let board = Board::with_no_holes(2, 4, 1);
        let result = run_game(players, Some(board));
        assert!(result.final_state.is_game_over());
        assert_eq!(result.final_statuses, vec![Won, Won]);
    }

    // Runs a game that should end with both players winning.
    #[test]
    fn run_game_both_players_win() {
        // set up players
        let players = vec![
            ClientProxy::InHouseAI(InHousePlayer::with_zigzag_minmax_strategy()),
            ClientProxy::InHouseAI(InHousePlayer::with_zigzag_minmax_strategy()),
        ];

        let board = Board::with_no_holes(4, 4, 1);
        let result = run_game(players, Some(board));
        assert!(result.final_state.is_game_over());
        assert_eq!(result.final_statuses, vec![Won, Won]);
    }

    /// Runs a game with one cheating player who should get kicked from the game,
    /// and one who plays the normal minmax strategy and should thus win.
    /// It runs the same game twice, each time with cheaters in different positions
    /// in the turn order.
    #[test]
    fn run_game_cheater() {
        let players_cheater_second = vec![
            ClientProxy::InHouseAI(InHousePlayer::with_zigzag_minmax_strategy()),
            ClientProxy::InHouseAI(InHousePlayer::new(Box::new(CheatingStrategy))),
        ];
        
        let result = run_game(players_cheater_second, None);
        assert_eq!(result.final_statuses, vec![Won, Kicked]);
    }

    #[test]
    fn run_game_two_cheaters() {
        let players_cheater_first = vec![
            ClientProxy::InHouseAI(InHousePlayer::new(Box::new(CheatingStrategy))),
            ClientProxy::InHouseAI(InHousePlayer::with_zigzag_minmax_strategy()),
            ClientProxy::InHouseAI(InHousePlayer::new(Box::new(CheatingStrategy))),
        ];
        let result = run_game(players_cheater_first, None);
        assert_eq!(result.final_statuses, vec![Kicked, Won, Kicked]);
    }

    #[test]
    fn run_game_all_cheating_players() {
        let players_cheater_first = vec![
            ClientProxy::InHouseAI(InHousePlayer::new(Box::new(CheatingStrategy))),
            ClientProxy::InHouseAI(InHousePlayer::new(Box::new(CheatingStrategy))),
            ClientProxy::InHouseAI(InHousePlayer::new(Box::new(CheatingStrategy))),
        ];
        let result = run_game(players_cheater_first, None);
        assert_eq!(result.final_statuses, vec![Kicked, Kicked, Kicked]);
    }
}
