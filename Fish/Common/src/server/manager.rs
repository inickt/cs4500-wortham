//! This file contains the interface for the Tournament Manager,
//! which sets up games for and runs an entire tournament.
use crate::server::referee;
use crate::server::referee::ClientStatus;
use crate::server::serverclient::{ Client, ClientProxy };
use crate::common::gamestate;
use crate::common::board::Board;
use crate::common::util;
use crate::common::player::PlayerId;

use serde_json::json;

use std::collections::BTreeMap;

/// Represents a single game within a bracket, with each client in the Vec
/// being a client in the game. The order of this grouping will be the same
/// as the turn order in the resulting game.
type PlayerGrouping = Vec<Client>;

/// Represents one round of Games, either a Round containing one PlayerGrouping
/// per Fish game to play, or an End, which represents the end of the whole tournament.
enum Bracket {
    Round { games: Vec<PlayerGrouping> },
    End,
}

/// Runs a complete tournament with the given clients by dividing
/// players into Brackets and putting each PlayerGrouping into a
/// game managed by a referee each round until there is one final
/// game or the set of winners stays the same 2 games in a row.
/// 
/// Returns the list of statuses (whether each client Won, Lost, or
/// was Kicked from the tournament as a whole) for each client
/// in the same order as the given clients list.
///
/// See next_bracket for how clients are divided up into brackets each round.
/// 
/// Players should expect a tournament to begin when they first 
/// receive a game state from the referee managing their first round.
/// 
/// When the tournament finishes, all active players (i.e. those who are not
/// kicked) are notified as to whether they won or lost. Winners who fail to
/// accept this message are converted to players who lost before the final list
/// of statuses is returned.
/// 
/// It is assumed that the given list of players should not have any
/// Kicked clients. 
pub fn run_tournament(proxies: Vec<ClientProxy>, board: Option<Board>) -> Vec<ClientStatus> {
    let mut results = BTreeMap::new();

    let clients = proxies.into_iter().enumerate().map(|(id, client)| {
        // Clients win by default until they lose a game or are kicked.
        // This means for the tournament of a single player, they win by default
        // even though they played 0 games
        results.insert(PlayerId(id), ClientStatus::Won);
        Client::new(id, client)
    }).collect::<Vec<_>>();

    let clients = notify_tournament_started(&clients, &mut results);

    run_tournament_rec(&clients, board, None, &mut results);
    let statuses = results.values().copied().collect();

    notify_tournament_finished(clients, statuses)
}

/// Notify the given clients that the tournament has started. If a client fails to accept the message,
/// then their status is changed to Kicked. The players that successfully accepted the starting
/// message are returned in the same order.
fn notify_tournament_started(clients: &[Client], results: &mut BTreeMap<PlayerId, ClientStatus>) -> Vec<Client> {
    clients.iter().filter_map(|client| {
        let message = json!({
            "type": "StartTournament",
            "assigned_player_id": client.id.0,
        });

        let serialized_msg = serde_json::to_string(&message).unwrap();

        match client.proxy.borrow_mut().send(serialized_msg.as_bytes()) {
            Ok(_) => Some(client.clone()),
            Err(_) => {
                results.insert(client.id, ClientStatus::Kicked);
                None
            }
        }
    }).collect()
}

/// Notify the given clients that the tournament has finished. If a winning client fails to accept the message,
/// then their status is changed to Lost. This change is reflected in the returned client statuses
/// which is in the same ordering as the given statuses vector.
fn notify_tournament_finished(clients: Vec<Client>, mut statuses: Vec<ClientStatus>) -> Vec<ClientStatus> {
    let winners = clients.iter().zip(statuses.iter())
        .filter(|(_, status)| **status == ClientStatus::Won)
        .map(|(client, _)| client.id)
        .collect::<Vec<PlayerId>>();

    let message = json!({
        "type": "TournamentFinished",
        "winners": winners
    });

    let serialized_msg = serde_json::to_string(&message).unwrap();

    for (i, tournament_client) in clients.iter().enumerate() {
        if tournament_client.proxy.borrow_mut().send(serialized_msg.as_bytes()).is_err()
                && statuses[i] == ClientStatus::Won {
            statuses[i] = ClientStatus::Lost;
        }
    }

    statuses
}

/// Performs the recursion for run_tournament, keeping track of the number of winners
/// of the previous game which is used to end the game early if it is ever equal to the
/// number of players who won the most recent game.
fn run_tournament_rec(clients: &[Client], board: Option<Board>,
    previous_winner_count: Option<usize>, results: &mut BTreeMap<PlayerId, ClientStatus>)
{
    match next_bracket(clients, previous_winner_count) {
        Bracket::Round { games } => {
            let winners = run_round(games, board.clone(), results);
            run_tournament_rec(&winners, board, Some(clients.len()), results);
        },
        Bracket::End => (),
    }
}

/// Runs a single tournament round, returning the winning players.
/// The ordering of players returned does not change - save for the
/// players that were removed because they lost or cheated.
fn run_round(groups: Vec<PlayerGrouping>, board: Option<Board>,
    results: &mut BTreeMap<PlayerId, ClientStatus>) -> Vec<Client>
{
    let mut winners = vec![];
    for group in groups {
        let game_results = referee::run_game_shared(&group, board.clone());

        // Iterate through the result (Won | Lost | Kicked) of each client in the finished game
        // to update their overall tournament status
        for (client, status) in group.iter().zip(game_results.final_statuses.into_iter()) {
            results.insert(client.id, status);
            if status == ClientStatus::Won {
                winners.push(client.clone());
            }
        }
    }
    winners
}

/// Allocate players to games and return a bracket representing the tournament round to be run.
/// The allocation will assign players to games with the maximum number of players allowed for
/// an individual game. In the case of remaining players, the list of allocated games will
/// be backtracked and players will be removed, one-by-one, to form games of size one less
/// than the maximal number. This will occur until all players are assigned.
/// 
/// It is assumed that the given slice of players is sorted in ascending order of age. If the number
/// of player initially given is too small to create a game, Bracket::End is returned.
fn next_bracket(clients: &[Client], previous_player_count: Option<usize>) -> Bracket {
    if clients.len() < gamestate::MIN_PLAYERS_PER_GAME {
        return Bracket::End;
    }

    if previous_player_count.map_or(false, |count| count == clients.len()) {
        return Bracket::End;
    }

    Bracket::Round { games: create_player_groupings(clients) }
}

/// Create a list of player groupings to be used in a bracket. Players will be grouped into groups
/// of size gamestate::MAX_PLAYERS_PER_GAME. This function will also handle the case where there are remaining
/// players that cannot form a group of gamestate::MIN_PLAYERS_PER_GAME or more, in which case the allocated games 
/// will be backtracked and players will be removed, one-by-one, to form games of size one less than the maximal
/// number. This will occur until all players are assigned.
/// 
/// The given list of players is assumed to be sorted in ascending age order. This function will panic if the initial list of players
/// does not contain enough players to form a single game.
fn create_player_groupings(clients: &[Client]) -> Vec<PlayerGrouping> {
    let mut groups = vec![];
    let mut clients_per_game = gamestate::MAX_PLAYERS_PER_GAME;
    let mut clients = clients.to_vec();

    while !clients.is_empty() {
        if clients.len() < clients_per_game {
            if clients.len() >= gamestate::MIN_PLAYERS_PER_GAME {
                // Enough clients for one more game, push them all
                groups.push(clients);
                clients = vec![];
            } else if !groups.is_empty() && clients_per_game > gamestate::MIN_PLAYERS_PER_GAME {
                // backtrack
                clients.append(&mut groups.pop().unwrap());
                clients_per_game -= 1;
            } else {
                // Can't backtrack - not enough clients to form a single game or we're already
                // at the minimum number of players
                panic!("Not enough players to create 1 more group: #groups = {}, #remaining-players = {}", groups.len(), clients.len());
            }
        } else {
            groups.push(util::make_n(clients_per_game, |_| clients.remove(0)));
        }
    }

    groups
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::player::InHousePlayer;
    use crate::common::gamestate::GameState;
    use crate::common::tile::TileId;
    use crate::common::boardposn::BoardPosn;
    use crate::common::penguin::PenguinId;
    use crate::common::game_tree::GameTree;
    use crate::common::action::{Placement, Move};
    use crate::client::strategy::Strategy;
    use crate::client::strategy::find_minmax_move;
    use crate::client::strategy::find_zigzag_placement;
    use crate::server::connection::PlayerConnection;
    use crate::server::referee::ClientStatus::*;

    use std::net::{ TcpListener, TcpStream };

    /// A simple strategy for testing that works similarly to ZigZagMinMaxStrategy, except only has a lookahead of 1
    pub struct SimpleStrategy;

    impl Strategy for SimpleStrategy {
        fn find_placement(&mut self, gamestate: &GameState) -> Placement {
            find_zigzag_placement(gamestate)
        }

        fn find_move(&mut self, game: &mut GameTree) -> Move {
            find_minmax_move(game, 1)
        }
    }

    /// A strategy used to simulate a cheating player.
    pub struct CheatingStrategy;

    impl Strategy for CheatingStrategy {
        fn find_placement(&mut self, gamestate: &GameState) -> Placement {
            find_zigzag_placement(gamestate)
        }

        fn find_move(&mut self, _game: &mut GameTree) -> Move {
            Move::new(PenguinId(0), TileId(11))
        }
    }

    /// Create a player that uses a SimpleStrategy
    fn make_simple_strategy_player() -> ClientProxy {
        ClientProxy::InHouseAI(InHousePlayer::new(Box::new(SimpleStrategy)))
    }

    /// Creating a player that uses a cheating strategy
    fn make_cheating_player() -> ClientProxy {
        ClientProxy::InHouseAI(InHousePlayer::new(Box::new(CheatingStrategy)))
    }

    fn make_player_fails_to_accept(port: usize) -> ClientProxy {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).expect("Could not create listener");
        listener.set_nonblocking(true).ok();
        ClientProxy::Remote(PlayerConnection::new(listener))        
    }


    /// Run a full tournament of fish, with 8 players and a total of 2 rounds. The initial board after penguins are placed looks as follows:
    /// p1    p2    p3    p4 
    ///    p1    p2    p3    p4
    /// 1     x     x     x 
    /// 
    /// Where there are 1 fish per tile, and x denotes a removed tile.
    ///
    /// After round 1, the board looks as follows:
    /// p1    p2    p3    p4 
    ///    x     p2    p3    p4
    /// p1    x     x     x 
    /// 
    /// Player 1 of each individual game will be the winner. This will correspond to players 1 and 5 of the tournament.
    ///
    /// After the placement phase, the board at round 2 looks as follows:
    /// p1    p2    p1    p2
    ///    p1    p2    p1    p2
    /// 1     x     x     x 
    ///
    /// After round 2, the board looks as follows:
    /// p1    p2    p1    p2
    ///    x     p2    p1    p2
    /// p1     x     x     x 
    ///
    /// Thus, player 1 of the tournament will be the winner.
    ///
    /// Each player uses a simple strategy, with a min-max lookahead of 1.
    #[test]
    fn test_run_tournament() {
        // make sure to test tournaments with > 2 rounds
        // set up players
        let players = util::make_n(8, |_|
            make_simple_strategy_player()
        );

        let holes = vec![BoardPosn::from((1, 2)), BoardPosn::from((2, 2)), BoardPosn::from((3, 2))];
        let board = Board::with_holes(3, 4, holes, 1);
        let statuses = run_tournament(players, Some(board));
        let mut winners = vec![Lost; 8];
        winners[0] = Won;
        assert_eq!(statuses, winners);
    }

    /// Test the running of a single tournament round. The round is the same as the first round of
    /// `test_run_tournament`. As such, players with IDs 0 and 4 (i.e. the first player of each individual
    /// Fish game) will win, and all other players will lose.
    #[test]
    fn test_run_round() {
        let player_grouping = vec![
            util::make_n(4, |id| Client::new(id, make_simple_strategy_player())),
            util::make_n(4, |id| Client::new(id + 4, make_simple_strategy_player())),
        ];

        let holes = vec![BoardPosn::from((1, 2)), BoardPosn::from((2, 2)), BoardPosn::from((3, 2))];
        let board = Board::with_holes(3, 4, holes, 1);
        let mut results = BTreeMap::new();
       
        let winners = run_round(player_grouping, Some(board), &mut results);

        assert_eq!(winners.len(), 2);
        assert_eq!(winners[0].id.0, 0);
        assert_eq!(winners[1].id.0, 4);
    }

    // Test that tournament clients can be notified of the tournament starting at the beginning of a
    // tournament. This test checks that players that fail to respond to the starting message will
    // have their status updated to be kicked from the tournament.
    #[test]
    fn test_notify_tournament_started() {
        let clients = vec![
            Client::new(0, make_simple_strategy_player()), // player who will accept message
            Client::new(1, make_player_fails_to_accept(8081)), // player that will fail to accept message
            Client::new(2, make_simple_strategy_player()), // player that accepts message
            Client::new(3, make_cheating_player()), // player that accepts message
        ];

        // initial statuses reported by tournament manager
        let mut results = BTreeMap::new();
        let good_clients = notify_tournament_started(&clients, &mut results);
        assert_eq!(good_clients.len(), 3);
        assert_eq!(good_clients[0].id.0, 0);
        assert_eq!(good_clients[1].id.0, 2);
        assert_eq!(good_clients[2].id.0, 3);
        assert_eq!(results.len(), 1);
        assert_eq!(results[&PlayerId(1)], ClientStatus::Kicked);
    }

    /// Test that tournament clients can be successfully notified of the end of a tournament. This test also checks that
    /// winning players who fail to respond to this message are made into losing players. The test assumes that a tournament
    /// returned a list of 4 player clients, such that the first two players won, the third player lost, and the fourth player
    /// was kicked. The second player will fail to respond to the tournament finished message, and as such will become a losing player.
    /// All other players will accept the message and will not have their statuses changed.
    #[test]
    fn test_notify_tournament_finished() {
        let clients = vec![
            Client::new(0, make_simple_strategy_player()), // player who will win and accept message
            Client::new(1, make_player_fails_to_accept(8080)), // player that will win but fail to accept message
            Client::new(2, make_simple_strategy_player()), // player that loses and accepts message
            Client::new(3, make_cheating_player()), // player that is kicked (does not do anything with message)
        ];

        // initial statuses reported by tournament manager
        let statuses = vec![Won, Won, Lost, Kicked];
        let new_statuses = notify_tournament_finished(clients, statuses);
        assert_eq!(new_statuses, vec![Won, Lost, Lost, Kicked]);
    }

    /// Run a round of fish with 4 players where the first player is attempting to cheat.
    ///
    /// The initial board after penguins are placed looks as follows:
    /// p1    p2    p3    p4 
    ///    p1    p2    p3    p4
    /// x     5     x     x11 
    ///
    /// Where there are 1 fish per tile, x denotes a removed tile, and a number denotes the tile ID.
    /// x11 denotes the removed tile with TileID 11 that the cheating player will always attempt to move to.
    ///
    /// Player 1 will be kicked upon its first move. The board will then look as follows:
    /// 0    p2    p3    p4 
    ///    1    p2    p3    p4
    /// x    5     x     x 
    ///
    /// At the end of the round, the board looks as follows:
    /// 0    x     p3    p4 
    ///   p2    x    p3    p4
    /// x    p2     x     x 
    /// 
    /// Player 2 will be the winner.
    #[test]
    fn test_run_bad_round() {
        let players: Vec<ClientProxy> = vec![
            make_cheating_player(),
            make_simple_strategy_player(),
            make_simple_strategy_player(),
            make_simple_strategy_player()
        ];

        let holes = vec![BoardPosn::from((0, 2)), BoardPosn::from((2, 2)), BoardPosn::from((3, 2))];
        let board = Board::with_holes(3, 4, holes, 1);

        let statuses = run_tournament(players, Some(board));
        let winners = vec![
            ClientStatus::Kicked,
            ClientStatus::Won,
            ClientStatus::Lost,
            ClientStatus::Lost
        ];

        assert_eq!(statuses, winners);
    }

    /// Partition 8 players into two games that both result in all winners. At the end of this test
    /// every player should come back a winner.
    #[test]
    fn test_tournament_ends_when_two_rounds_in_a_row_produce_same_winners() {
        // set up 8 players
        let players = util::make_n(8, |_|
            ClientProxy::InHouseAI(InHousePlayer::with_zigzag_minmax_strategy())
        );

        // Only 8 spaces to place penguins with a total of 8 penguins in the game.
        // No one can move so everyone has the same score and everyone wins.
        let board = Board::with_no_holes(2, 4, 1);
        let statuses = run_tournament(players, Some(board));
        assert_eq!(statuses, vec![ClientStatus::Won; 8]);
    }

    #[test]
    fn test_tournament_ends_when_too_few_players_for_single_game() { 
        // The only case where there are too few players (except for when there are none) is when there is only 1 player.
        let players = vec![
            ClientProxy::InHouseAI(InHousePlayer::with_zigzag_minmax_strategy()),
        ];

        let board = Board::with_no_holes(2, 4, 1);

        let statuses = run_tournament(players, Some(board));
        assert_eq!(statuses, vec![ClientStatus::Won]);
    }

    #[test]
    fn test_tournament_no_players() { 
        let board = Board::with_no_holes(2, 4, 1);
        let statuses = run_tournament(vec![], Some(board));
        assert_eq!(statuses, vec![]);
    }

    /// Test a tournament where there are enough players for a just a single round. This test is identical to the second round of
    /// `test_run_tournament`. Player 1 will win this tournament.
    #[test]
    fn test_tournament_ends_when_partipant_count_is_small_enough_to_have_one_final_game() {
        let players = vec![
            make_simple_strategy_player(),
            make_simple_strategy_player(),
        ];
        
        let board = Board::with_no_holes(5, 5, 2);
        let statuses = run_tournament(players, Some(board));
        let winners = vec![ClientStatus::Won, ClientStatus::Lost];
        assert_eq!(statuses, winners);
    }

    /// Test a tournament where players need to be reallocated in order to ensure that
    /// there are enough players in each game. Assume a list of players [1, 2, 3, 4, 5].
    /// The final allocation of the games should be [1, 2, 3] and [4, 5].
    #[test]
    fn test_allocate_backtracking() {
        // set up players
        let clients: Vec<_> = util::make_n(5, |id| Client::new(id, make_simple_strategy_player()));

        match next_bracket(&clients, None) {
            Bracket::Round { games } => {
                assert_eq!(games.len(), 2);
                assert_eq!(games[0].len(), 3);
                assert_eq!(games[1].len(), 2);
            },
            Bracket::End => {
                unreachable!("Allocate backtracking for 5 players always results in at least 1 round");
            }
        }
    }

    #[test]
    fn test_allocate_ends_when_too_few_players_for_single_game() { 
        let clients = vec![Client::new(0, make_simple_strategy_player())];

        // next_bracket of 1 player
        match next_bracket(&clients, None) {
            Bracket::Round { .. } => panic!("Expected next_bracket to return Bracket::End, found Bracket::Round"),
            Bracket::End => (),
        }

        // next_bracket of 0 players
        match next_bracket(&[], None) {
            Bracket::Round { .. } => panic!("Expected next_bracket to return Bracket::End, found Bracket::Round"),
            Bracket::End => (),
        }
    }
}
