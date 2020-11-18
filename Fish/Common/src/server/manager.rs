//! This file contains the interface for the Tournament Manager,
//! which sets up games for and runs an entire tournament.
use crate::server::referee;
use crate::server::referee::ClientStatus;
use crate::server::serverclient::Client;
use crate::common::gamestate;
use crate::common::board::Board;
use crate::common::util;

use std::collections::BTreeMap;

/// The unique Id for a given client.
/// These are equal to the Client's index in the clients
/// Vec passed to run_tournament
#[derive(Copy, Clone, PartialEq, Eq, Ord, PartialOrd)]
pub struct ClientId(pub usize);

/// Represents the client's connection info along with an
/// id to identify that particular client across all tournament games.
struct TournamentClient {
    id: ClientId,
    client: Client,
}

/// Represents a single game within a bracket, with each client in the Vec
/// being a client in the game. The order of this grouping will be the same
/// as the turn order in the resulting game.
type PlayerGrouping = Vec<TournamentClient>;

/// Represents the state of a Bracket, either a Round containing one PlayerGrouping
/// per Fish game to play, or an End, which contains the winning players of the
/// tournament as a whole.
enum Bracket {
    Round { games: Vec<PlayerGrouping> },
    End,
}

/// Runs a complete tournament with the given players by dividing
/// players into Brackets and putting each PlayerGrouping into a
/// game managed by a referee.
/// 
/// Returns the list of players who won the tournament overall.
///
/// At each round, the tournament manager will determine
/// how to allocate players to games, and will create a referee
/// to run a game with each grouping of players.
/// 
/// After each game, sends post-game statistics to each observer.
/// Examples of post-game statistics may include win/loss count
/// of each player, total or average scores of each player, etc.
/// 
/// It is assumed that the given list of players should not have any
/// Kicked clients. 
pub fn run_tournament(clients: Vec<Client>, board: Option<Board>) -> Vec<ClientStatus> {
    let mut results = BTreeMap::new();

    let tournament_clients = clients.into_iter().enumerate().map(|(i, client)|
        TournamentClient { client, id: ClientId(i) }
    ).collect();

    run_tournament_rec(tournament_clients, board, None, &mut results);
    results.values().copied().collect()
}

/// Performs the recursion for run_tournament, keeping track of the number of winners
/// of the previous game which is used to end the game early if it is ever equal to the
/// number of players who won the most recent game.
fn run_tournament_rec(clients: Vec<TournamentClient>, board: Option<Board>,
    previous_winner_count: Option<usize>, results: &mut BTreeMap<ClientId, ClientStatus>)
{
    let client_count = clients.len();
    match next_bracket(clients, previous_winner_count) {
        Bracket::Round { games } => {
            let winners = run_round(games, board.clone(), results);
            run_tournament_rec(winners, board, Some(client_count), results);
        },
        Bracket::End => (),
    }
}

/// Runs a single tournament round, returning the winning players.
/// The ordering of players returned does not change - save for the
/// players that were removed because they lost or cheated.
fn run_round(groups: Vec<PlayerGrouping>, board: Option<Board>,
    results: &mut BTreeMap<ClientId, ClientStatus>) -> Vec<TournamentClient>
{
    let mut winners = vec![];
    for group in groups {
        let first_id = group[0].id;
        let clients = group.into_iter().map(|tournament_client| tournament_client.client).collect();

        let game_results = referee::run_game(clients, board.clone());

        // Iterate through the result (Won | Lost | Kicked) of each client in the finished game
        // to update their overall tournament status
        for (i, (client, status)) in game_results.final_players.into_iter().enumerate() {
            let id = ClientId(first_id.0 + i);
            results.insert(id, status);
            if status == ClientStatus::Won {
                winners.push(TournamentClient { client, id });
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
fn next_bracket(clients: Vec<TournamentClient>, previous_player_count: Option<usize>) -> Bracket {
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
fn create_player_groupings(mut players: Vec<TournamentClient>) -> Vec<PlayerGrouping> {
    let mut groups = vec![];
    let mut players_per_game = gamestate::MAX_PLAYERS_PER_GAME;

    while !players.is_empty() {
        if players.len() < players_per_game {
            if !groups.is_empty() && players_per_game > gamestate::MIN_PLAYERS_PER_GAME {
                // backtrack
                players.append(&mut groups.pop().unwrap());
                players_per_game -= 1;
            } else {
                // Can't backtrack - not enough players to form a single game or we're already
                // at the minimum number of players
                panic!("Not enough players to create 1 more group: #groups = {}, #remaining-players = {}", groups.len(), players.len());
            }
        } else {
            groups.push(util::make_n(players_per_game, |_| players.remove(0)));
        }
    }

    groups
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::client::player::InHousePlayer;
    use crate::common::gamestate::GameState;
    use crate::common::game_tree::GameTree;
    use crate::common::action::{Placement, Move};
    use crate::client::strategy::Strategy;
    use crate::client::strategy::find_minmax_move;
    use crate::client::strategy::find_zigzag_placement;

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

    /// Create a player that uses a SimpleStrategy
    fn make_simple_strategy_player() -> InHousePlayer {
        InHousePlayer::new(Box::new(SimpleStrategy))
    }

    /// and players 1 and 0 4 will advance to
    /// a two player round. This round will result in player 0 winning.
    /// 
    /// The initial 3oard after penguins are placed looks as follows:
    /// 
    /// 3 fish on each tile
    /// p1    p2    p3    p4    p1
    ///    p2    p3    p4    3     3
    /// 3     3     3     3     3
    ///    3     3     3     3     3
    /// 3     3     3     3     3
    ///
    /// After round 1, the board looks as follows
    /// x     x     x     x     x
    ///    x     x     x     x     x
    /// x     x     x     x     x
    ///    x     x     x     x     x
    /// p1   p2    p3     p4     x
    ///
    /// Run a tournament using 8 players with simple strategies. The first player of each 4 player round will win, and then the first player wil
    #[test]
    fn test_run_tournament() {
        // make sure to test tournaments with > 2 rounds
        // set up players
        let players = util::make_n(8, |_|
            Client::InHouseAI(make_simple_strategy_player())
        );

        let board = Board::with_no_holes(5, 5, 2);
        let statuses = run_tournament(players, Some(board));
        let mut winners = vec![ClientStatus::Lost; 8];
        winners[0] = ClientStatus::Won;
        assert_eq!(statuses, winners);
    }

    #[test]
    fn test_run_round() {
        // assert_eq!(run_round(...), vec![...]);
    }

    #[test]
    fn test_run_bad_round() { }

    /// Partition 8 players into two games that both result in all winners. At the end of this test
    /// every player should come back a winner.
    #[test]
    fn test_tournament_ends_when_two_rounds_in_a_row_produce_same_winners() {
        // set up 8 players
        let players = util::make_n(8, |_|
            Client::InHouseAI(InHousePlayer::with_zigzag_minmax_strategy())
        );

        let board = Board::with_no_holes(2, 4, 1);
        let statuses = run_tournament(players, Some(board));
        assert_eq!(statuses, vec![ClientStatus::Won; 8]);
    }

    #[test]
    fn test_tournament_ends_when_too_few_players_for_single_game() { 

        // The only case where there are too few players (except for when there are none) is when there is only 1 player.
        let players = vec![
            Client::InHouseAI(InHousePlayer::with_zigzag_minmax_strategy()),
        ];

        let board = Board::with_no_holes(2, 4, 1);

        let statuses = run_tournament(players, Some(board));
        assert_eq!(statuses, vec![ClientStatus::Won]);
    }

    #[test]
    fn test_tournament_no_players() { 
        let board = Board::with_no_holes(2, 4, 1);

        let statuses = run_tournament(vec![], Some(board));
        assert_eq!(statuses, vec![ClientStatus::Won]);
    }

    #[test]
    fn test_tournament_ends_when_partipant_count_is_small_enough_to_have_one_final_game() { }

    #[test]
    fn test_allocate_backtracking() {
        // Test allocating 5 Clients
        // Backtrack from [4, 1] to [3, 2]

    }
}
