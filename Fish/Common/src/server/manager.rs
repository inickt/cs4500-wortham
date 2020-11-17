//! This file contains the interface for the Tournament Manager,
//! which sets up games for and runs an entire tournament.
use crate::server::referee;
use crate::server::referee::ClientStatus;
use crate::server::serverclient::Client;
use crate::common::gamestate;
use crate::common::board::Board;
use crate::common::util;

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
fn run_tournament(players: Vec<Client>, board: Option<Board>) -> Vec<Client> {
    // TODO tie checking
    match next_bracket(players) {
        Bracket::Round { games } => {
            let winners = run_round(games, board.clone());
            run_tournament(winners, board)
        },
        Bracket::End { winning_players } => winning_players,
    }
}

/// Runs a single tournament round, returning the winning players.
/// The ordering of players returned does not change - save for the
/// players that were removed because they lost or cheated.
fn run_round(groups: Vec<PlayerGrouping>, board: Option<Board>) -> Vec<Client> {
    let mut all_winners = vec![];
    for group in groups {
        let results = referee::run_game(group, board.clone()).final_players;

        let mut winners = results.into_iter()
            .filter(|(_, result)| result == &ClientStatus::Won)
            .map(|(client, _)| client)
            .collect();

        all_winners.append(&mut winners);
    }
    all_winners
}


/// Represents a single game within a bracket, as the vector of Players
/// to pass into that game.
type PlayerGrouping = Vec<Client>;

/// Represents the state of a Bracket, either a Round containing one PlayerGrouping
/// per Fish game to play, or an End, which contains the winning players of the
/// tournament as a whole.
enum Bracket {
    Round { games: Vec<PlayerGrouping> },
    End { winning_players: Vec<Client> },
}

/// Allocate players to games and return a bracket representing the tournament round to be run.
/// The allocation will assign players to games with the maximum number of players allowed for
/// an individual game. In the case of remaining players, the list of allocated games will
/// be backtracked and players will be removed, one-by-one, to form games of size one less
/// than the maximal number. This will occur until all players are assigned.
/// 
/// It is assumed that the given slice of players is sorted in ascending order of age. If the number
/// of player initially given is too small to create a game, Bracket::End is returned.
fn next_bracket(players: Vec<Client>) -> Bracket {

    if players.len() < gamestate::MIN_PLAYERS_PER_GAME {
        return Bracket::End { winning_players: players };
    }

    Bracket::Round { games: create_player_groupings(players) }
}

/// Create a list of player groupings to be used in a bracket. Players will be grouped into groups
/// of size gamestate::MAX_PLAYERS_PER_GAME. This function will also handle the case where there are remaining
/// players that cannot form a group of gamestate::MIN_PLAYERS_PER_GAME or more, in which case the allocated games 
/// will be backtracked and players will be removed, one-by-one, to form games of size one less than the maximal
/// number. This will occur until all players are assigned.
/// 
/// The given list of players is assumed to be sorted in ascending age order. This function will panic if the initial list of players
/// does not contain enough players to form a single game.
fn create_player_groupings(mut players: Vec<Client>) -> Vec<PlayerGrouping> {
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

#[test]
fn test_run_tournament() { }

#[test]
fn test_run_round() { }

#[test]
fn test_run_bad_round() { }

#[test]
fn test_tournament_ends_when_two_rounds_in_a_row_produce_same_winners() { }

#[test]
fn test_tournament_ends_when_too_few_players_for_single_game() { }

#[test]
fn test_tournament_ends_when_partipant_count_is_small_enough_to_have_one_final_game() { }

#[test]
fn test_allocate_backtracking() {
    // Test allocating 5 Clients
    // Backtrack from [4, 1] to [3, 2]
}
