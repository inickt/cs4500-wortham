//! This file contains all logic and data regarding the Referee component,
//! which runs complete games of Fish.
use crate::common::action::{ Action, Move, Placement };
use crate::common::board::Board;
use crate::common::gamestate::GameState;
use crate::common::gamephase::GamePhase;
use crate::common::game_tree::GameTree;
use crate::common::player::PlayerId;

use crate::client::player::InHousePlayer;

use serde::Deserialize;
use serde_json::{ Deserializer, de::IoRead };

use std::io::{ Read, Write };
use std::collections::HashMap;

/// Cases to implement: 
/// - timeout
/// - 
pub struct PlayerConnection {
    input_deserializer: Deserializer<IoRead<Box<dyn Read>>>,
    output_stream: Box<dyn Write>,
}

impl PlayerConnection {
    pub fn new(input_stream: Box<dyn Read>, output_stream: Box<dyn Write>) -> PlayerConnection {
        let input_deserializer = Deserializer::from_reader(input_stream);
        PlayerConnection {
            input_deserializer,
            output_stream
        }
    }
}

pub enum Player {
    Remote(PlayerConnection),
    InHouseAI(InHousePlayer),
}

impl Player {
    /// Get an action of the given player, either waiting for a remote player
    /// or prompting an ai player to take a turn.
    /// 
    /// TODO: Add 1 minute timeout for remote players
    pub fn get_action(&mut self) -> Option<Action> {
        match self {
            Player::Remote(connection) => {
                // Wait for the player to send their Action
                Action::deserialize(&mut connection.input_deserializer).ok()
            },
            Player::InHouseAI(ai) => {
                ai.take_turn();
                serde_json::from_str(&mut ai.output_stream).ok()
            }
        }
    }

    /// Send a message to the player's input stream.
    /// 
    /// Since the possible server message to a player is that containing
    /// the current gamestate, it is expected the contents of this message
    /// contains the serialized gamestate.
    /// 
    /// Returns Ok(num_bytes_written) or otherwise returns an io error if
    /// the stream could not be written to.
    pub fn send(&mut self, message: &[u8]) -> Result<usize, std::io::Error> {
        match self {
            Player::Remote(connection) => {
                connection.output_stream.write(message)
            },
            Player::InHouseAI(ai) => { 
                ai.receive_gamestate(message);
                Ok(message.len())
            },
        }
    }
}

/// ????
struct Referee {
    /// Player input/output stream data, indexed on GameState's PlayerId
    players: HashMap<PlayerId, Player>,

    /// State of current game
    phase: GamePhase,
}

/// Runs a complete game of Fish, setting up the board and
/// waiting for player input for gameplay to occur, and terminating
/// when a player (or multiple) have won. Check out Planning/player-protocol.md
/// for more information on the Fish game.
/// 
/// Returns the winning players of the game
pub fn run_game(players: Vec<Player>, board: Option<Board>) -> GameState {
    let board = board.unwrap_or(Board::with_no_holes(5, 5, 3));
    let mut referee = Referee::new(players, board);

    while !referee.is_game_over() {
        referee.send_gamestate_to_all_players();
        referee.do_player_turn();
    }

    referee.phase.get_state().clone() //.winning_players.clone()
}

impl Referee {
    fn new(players: Vec<Player>, board: Board) -> Referee {
        let state = GameState::new(board, players.len());
        let players = state.turn_order.iter().copied().zip(players.into_iter()).collect();
        let phase = GamePhase::PlacingPenguins(state);
        Referee { players, phase }
    }
    
    /// Sends the serialized gamestate to each output stream in self.players
    /// If there was any error writing to any player, the referee assumes that
    /// player has disconnected and kicks them from the game, removing their penguins.
    fn send_gamestate_to_all_players(&mut self) {
        let mut disconnected_players = vec![];
        for (player_id, player) in self.players.iter_mut() {
            let serialized = serde_json::to_string(&self.phase.get_state()).unwrap();

            // Write to the player and if there was an error in doing so, kick them.
            if let Err(_) = player.send(serialized.as_bytes()) {
                disconnected_players.push(*player_id);
            }
        }

        for player_id in disconnected_players {
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
        let current_player = self.players.get_mut(&self.phase.current_turn())?;
        let placement = match current_player.get_action()? {
            Action::PlacePenguin(placement) => Some(placement),
            Action::MovePenguin(_) => None,
        }?;

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
        let current_player = self.players.get_mut(&self.phase.current_turn())?;
        let move_ = match current_player.get_action()? {
            Action::MovePenguin(move_) => Some(move_),
            Action::PlacePenguin(_) => None
        }?;

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

    /// Kick the given player from the game, removing all their penguins and
    /// their position in the turn order. This does not notify the player that
    /// they were kicked.
    fn kick_player(&mut self, player: PlayerId) {
        self.phase.get_state_mut().remove_player(player);
        self.players.remove(&player);

        if self.players.is_empty() {
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
    use crate::client::player::InHousePlayer;
    use crate::client::strategy::Strategy;
    use crate::common::tile::TileId;
    use crate::common::penguin::PenguinId;

    pub struct CheatingStrategy;

    impl Strategy for CheatingStrategy {
        fn find_placement(&mut self, gamestate: &GameState) -> Placement {
            Placement::new(TileId(0))
        }

        fn find_move(&mut self, game: &mut GameTree) -> Move {
            Move::new(PenguinId(0), TileId(0))
        }
    }

    /// Runs a game where the first player should win if they're looking ahead enough
    /// turns. For more info on this specific game, see the explanation in
    /// client/strategy.rs, fn test_move_penguin_minmax_lookahead
    #[test]
    fn test_run_game_normal() {
        // set up players
        let players = vec![
            Player::InHouseAI(InHousePlayer::with_zigzag_minmax_strategy()),
            Player::InHouseAI(InHousePlayer::with_zigzag_minmax_strategy()),
        ];

        let board = Board::with_no_holes(3, 5, 1);
        let result = dbg!(run_game(players, Some(board)));
        assert!(result.is_game_over());
        // TODO ASSERT FIRST PLAYER WINS
    }

    // Test a game with multiple winners

    /// Runs a game with one cheating player who should get kicked from the game,
    /// and one who plays the normal minmax strategy and should thus win.
    /// It runs the same game twice, each time with cheaters in different positions
    /// in the turn order.
    fn test_run_game_cheater() {
        let players_cheater_second = vec![
            Player::InHouseAI(InHousePlayer::with_zigzag_minmax_strategy()),
            Player::InHouseAI(InHousePlayer::new(Box::new(CheatingStrategy))),
        ];
        
        let result = run_game(players_cheater_second, None);
        assert!(result.is_game_over());
        // TODO ASSERT FIRST PLAYER WINS


        let players_cheater_first = vec![
            Player::InHouseAI(InHousePlayer::new(Box::new(CheatingStrategy))),
            Player::InHouseAI(InHousePlayer::with_zigzag_minmax_strategy()),
        ];
        let result = run_game(players_cheater_first, None);
        assert!(result.is_game_over());
        // TODO ASSERT SECOND PLAYER WINS
    }
}