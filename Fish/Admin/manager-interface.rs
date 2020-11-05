//! This file contains the interface for the Tournament Manager,
//! which sets up games for and runs an entire tournament.

/// Runs a complete tournament with the given players by dividing
/// players into Brackets based on the given BracketStrategy and
/// putting each PlayerGrouping into a game managed by a referee.
/// 
/// Returns the list of players who won the tournament overall.
///
/// At each round, uses the BracketStrategy to determine
/// how to allocate players to games, then creates a referee
/// to run a game with each grouping of players.
/// 
/// After each game, sends post-game statistics to each observer.
/// Examples of post-game statistics may include win/loss count
/// of each player, total or average scores of each player, etc.
fn run_tournament<S>(players: &[Player], observers: &[TcpStream], strategy: S) -> Vec<Player>
    where S: BracketStrategy {}

/// A BracketStrategy determines how to divide a set of winning players
/// into a "bracket" - taken here to mean a set of n > 0 games each
/// containing between 2 and 4 players.
trait BracketStrategy {
    /// Divides players into groups of 2-4 players per game.
    /// If there are too few players to do this, Bracket::End
    /// is returned containing each player given.
    fn initial_bracket(&self, players: &[Player]) -> Bracket;

    /// Creates the next bracket from the final game state of each game
    /// of the previous bracket. This may reorganize the players in
    /// each of the given finished games in any way it wishes to create
    /// the next Bracket. We include the GameState so the strategy may
    /// creates brackets with players who lost, based on scores,
    /// final positions, etc.
    fn next_bracket(&self, game_results: &[GameState]) -> Bracket;
}

/// Represents a single game within a bracket, as the vector of Players
/// to pass into that game.
type PlayerGrouping = Vec<Player>;

/// Represents the state of a Bracket, either a Round containing one PlayerGrouping
/// per Fish game to play, or an End, which contains the winning players of the
/// tournament as a whole.
enum Bracket {
    Round { games: Vec<PlayerGrouping> },
    End { winning_players: Vec<Player> },
}
