//! This file contains all logic and data regarding the Referee component,
//! which runs complete games of Fish. To do this, it starts and runs the
//! game loop, sending the gamestate to all players each turn then retrieving
//! a player's move and validating it until the game is over.
use crate::common::action::PlayerMove;
use crate::common::board::Board;
use crate::common::gamestate::GameState;
use crate::common::gamephase::GamePhase;
use crate::common::game_tree::GameTree;
use crate::common::player::{ PlayerId, PlayerColor };
use crate::server::client::{ Client, ClientWithId };

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
    clients: Vec<ClientWithId>,

    /// The state of current game, separated by the current phase it is in.
    phase: GamePhase,

    /// The past moves that have been received by each client with the most
    /// recent being last. Empty until the MovePenguins phase and cleared when
    /// a player is kicked.
    move_history: Vec<PlayerMove>,
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
pub fn run_game(clients: Vec<Box<dyn Client>>, board: Option<Board>) -> GameResult {
    let clients: Vec<_> = clients.into_iter().enumerate()
        .map(|(id, player)| ClientWithId::new(id, player)).collect();
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
pub fn run_game_shared(clients: &[ClientWithId], board: Option<Board>) -> GameResult {
    let board = board.unwrap_or(Board::with_no_holes(5, 5, 3));
    let mut referee = Referee::new(clients.to_vec(), board);

    referee.initialize_clients();

    while !referee.is_game_over() {
        eprintln!("{:?}", referee.phase.get_state());
        referee.do_player_turn();
    }

    referee.get_game_result()
}

impl Referee {
    fn new(clients: Vec<ClientWithId>, board: Board) -> Referee {
        let client_ids = clients.iter().map(|client| client.id).collect();
        let state = GameState::with_players(board, client_ids);
        let phase = GamePhase::PlacingPenguins(state);
        Referee { clients, phase, move_history: vec![] }
    }

    fn get_client_player_color(&self, client: &ClientWithId) -> PlayerColor {
        let state = self.phase.get_state();
        state.players.get(&client.id).unwrap().color
    }

    fn current_client(&self) -> &ClientWithId {
        let current_player_id = self.phase.current_turn();
        self.clients.iter().find(|client| client.id == current_player_id).unwrap()
    }

    fn initialize_clients(&mut self) {
        let mut clients_to_kick = vec![];

        let state = self.phase.get_state();
        for client in self.clients.iter() {
            let color = self.get_client_player_color(client);
            let result = client.borrow_mut().initialize_game(state, color);
            if result.is_none() {
                clients_to_kick.push(client.id);
            }
        }

        for id in clients_to_kick {
            self.kick_player(id);
        }
    }

    /// Returns the winners, losers, and kicked players of the game, along
    /// with the final game state of the game.
    /// 
    /// Assumes that the game this referee was hosting has been played to
    /// completion - otherwise no winners will be returned.
    fn get_game_result(self) -> GameResult {
        let Referee { clients, phase, .. } = self;

        let final_statuses = clients.into_iter().map(|client| {
            if client.kicked {
                ClientStatus::Kicked
            } else if phase.get_state().winning_players.as_ref()
                    .map_or(false, |winning_players| winning_players.contains(&client.id)) {

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
        let placement = self.current_client().borrow_mut().get_placement(self.phase.get_state())?;
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
        let move_history = self.get_move_history_for_current_client();

        let move_ = self.current_client().borrow_mut().get_move(self.phase.get_state(), &move_history)?;
        let current_player_color = self.get_client_player_color(self.current_client());

        match &mut self.phase {
            GamePhase::MovingPenguins(gametree) => {
                let starting_state = gametree.get_state();
                let player_move = PlayerMove::new(current_player_color, move_, starting_state)?;

                self.phase.try_do_move(move_)?;
                self.move_history.push(player_move);
                Some(())
            },
            _ => unreachable!("do_player_move called outside of the MovingPenguins phase"),
        }
    }

    /// Send the move history from the last time this player moved. Most recent moves are last.
    fn get_move_history_for_current_client(&self) -> Vec<PlayerMove> {
        let current_client_color = self.get_client_player_color(self.current_client());

        let mut history = self.move_history.iter().rev()
            .take_while(|player_move| player_move.mover != current_client_color)
            .copied()
            .collect::<Vec<PlayerMove>>();

        history.reverse();
        history
    }

    /// Kick the given player from the game, removing all their penguins and
    /// their position in the turn order. This does not notify the player that
    /// they were kicked.
    fn kick_player(&mut self, player: PlayerId) {
        self.phase.get_state_mut().remove_player(player);

        eprintln!("Kicking player {}!", player.0);

        self.clients.iter_mut()
            .find(|client| client.id == player)
            .map(|client| client.kicked = true);

        // Must manually update after kicking a player to update the tree of valid moves in the game
        // tree, if needed
        self.phase.update_from_gamestate(self.phase.get_state().clone());

        // Clear the move history when we kick players so as to not retain moves
        // made by players that are no longer in the game
        self.move_history.clear();

        // The game ends early if all clients are kicked
        if self.clients.iter().all(|client| client.kicked) {
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
    use crate::server::strategy::Strategy;
    use crate::server::ai_client::AIClient;
    use crate::common::action::{ Move, Placement };
    use crate::common::tile::TileId;

    pub struct CheatingStrategy;

    impl Strategy for CheatingStrategy {
        fn find_placement(&mut self, _gamestate: &GameState) -> Placement {
            Placement::new(TileId(0))
        }

        fn find_move(&mut self, _game: &mut GameTree) -> Move {
            Move::new(TileId(0), TileId(0))
        }
    }

    /// Runs a game where the first player should win if they're looking ahead enough
    /// turns. For more info on this specific game, see the explanation in
    /// client/strategy.rs, fn test_move_penguin_minmax_lookahead
    #[test]
    fn run_game_normal() {
        // set up players
        let players: Vec<Box<dyn Client>> = vec![
            Box::new(AIClient::with_zigzag_minmax_strategy()),
            Box::new(AIClient::with_zigzag_minmax_strategy()),
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
        let players: Vec<Box<dyn Client>> = vec![
            Box::new(AIClient::with_zigzag_minmax_strategy()),
            Box::new(AIClient::with_zigzag_minmax_strategy()),
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
        let players: Vec<Box<dyn Client>> = vec![
            Box::new(AIClient::with_zigzag_minmax_strategy()),
            Box::new(AIClient::with_zigzag_minmax_strategy()),
        ];

        let board = Board::with_no_holes(4, 4, 1);
        let result = run_game(players, Some(board));
        assert!(result.final_state.is_game_over());
        assert_eq!(result.final_statuses, vec![Won, Won]);
    }

    /// Runs a game with one cheating player who should get kicked from the game,
    /// and one who plays the normal minmax strategy and should thus win.
    #[test]
    fn run_game_cheater() {
        let players_cheater_second: Vec<Box<dyn Client>> = vec![
            Box::new(AIClient::with_zigzag_minmax_strategy()),
            Box::new(AIClient::new(Box::new(CheatingStrategy))),
        ];
        
        let result = run_game(players_cheater_second, None);
        assert_eq!(result.final_statuses, vec![Won, Kicked]);
    }

    #[test]
    fn run_game_two_cheaters() {
        let players_cheater_first: Vec<Box<dyn Client>> = vec![
            Box::new(AIClient::new(Box::new(CheatingStrategy))),
            Box::new(AIClient::with_zigzag_minmax_strategy()),
            Box::new(AIClient::new(Box::new(CheatingStrategy))),
        ];
        let result = run_game(players_cheater_first, None);
        assert_eq!(result.final_statuses, vec![Kicked, Won, Kicked]);
    }

    #[test]
    fn run_game_all_cheating_players() {
        let players_cheater_first: Vec<Box<dyn Client>> = vec![
            Box::new(AIClient::new(Box::new(CheatingStrategy))),
            Box::new(AIClient::new(Box::new(CheatingStrategy))),
            Box::new(AIClient::new(Box::new(CheatingStrategy))),
        ];
        let result = run_game(players_cheater_first, None);
        assert_eq!(result.final_statuses, vec![Kicked, Kicked, Kicked]);
    }
}
