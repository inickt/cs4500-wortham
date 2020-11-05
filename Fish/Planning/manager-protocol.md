# Tournament Manager Protocol

Authors: Jake Fecher and Ryan Drew
Repo: atlanta

## Stages of the Tournament
The tournament manager runs tournaments in "tournament rounds", which are
groups of games that include all players who remain in tournament. The remaining
players will be the non-cheating players who have not yet been eliminated
from the tournament; tournament elimination will be defined by a bracket-creation
stratgy passed into the Tournament Manager.

1. Tournament Setup
    - The tournament manager will receive the list of players for its tournament
    from the sign-up server. We assume that we will be given this API at a later
    date, as the sign-up server is being outsourced to a remote team.
    - The tournament manager will also receive its bracket strategy and the 
    list of tournament observers on initialization. This determines how each player
    in each round of the tournament should be organized into different games.
    This also determines when the tournament is finished. See BracketStrategy
    in manager-interface.rs for more information.

2. Tournament Rounds
    1. Plan brackets
        - The tournament manager will plan brackets according to the strategy it
        was initialized with. The strategy determines, among other things, the
        bracket structure and whether games should be run in parallel. See
        manager-protocol.rs for more information on the strategy. Bracket creation
        will not include cheating or disconnected players, regardless of the strategy
        used.
    2. Run games
        - At this stage, the tournament manager will create referees for each
        game in its bracket, and have the referees run those games. It does this
        using `referee::run_game`. Players will be notified they've been entered
        into a game via the referee sending the initial game state to all players
        in the game. Since we're calling the referee module's run_game, we assume
        games will terminate every time.
        - For each game, the tournament manager will add each tournament observer and
        optionally itself as a spectator for an individual game by communicating with
        the referee of that game. Observers will receive the same GameState updates at each turn
        that players resceive but are not able to make any actions.
    3. Collect data and run another round, or terminate
        - At this stage, the winners and cheaters of each game are reported to
        tournament observers, along with statistics information about each player,
        such as win/loss record. It may then use this information in the next bracket creation phase.
        - Will eventually terminate at this stage according to the bracket strategy.

3. Tournament End
    - Once the tournament finishes (as determined by the strategy) it will end
      with 0 or more victors.
    - The manager may optionally report the statistics information it collected,
    such as individual player win/loss counts, average points per game, longest
    or fastest games, etc. The exact statistics will be dependent on the strategy
    used.