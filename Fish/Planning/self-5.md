## Self-Evaluation Form for Milestone 5

Under each of the following elements below, indicate below where your
TAs can find:

- the data definition, including interpretation, of penguin placements for setups 
  - N/a, We only take in a penguin id and a tile id in our place function:
    https://github.ccs.neu.edu/CS4500-F20/atlanta/blob/b32dcc7f08171a614af16d1f35993b7fb5976aaf/Fish/Common/src/common/gamestate.rs#L100-L106
    

- the data definition, including interpretation, of penguin movements for turns
  - https://github.ccs.neu.edu/CS4500-F20/atlanta/blob/b32dcc7f08171a614af16d1f35993b7fb5976aaf/Fish/Common/src/common/action.rs#L9-L21

  - Invalid move information is given in the move_avatar function that actually does the checks:
    https://github.ccs.neu.edu/CS4500-F20/atlanta/blob/b32dcc7f08171a614af16d1f35993b7fb5976aaf/Fish/Common/src/common/gamestate.rs#L119-L128

- the unit tests for the penguin placement strategy 
  - https://github.ccs.neu.edu/CS4500-F20/atlanta/blob/b32dcc7f08171a614af16d1f35993b7fb5976aaf/Fish/Common/src/client/strategy.rs#L118-L168

- the unit tests for the penguin movement strategy; 
  given that the exploration depth is a parameter `N`, there should be at least two unit tests for different depths 
  - 1 lookahead: https://github.ccs.neu.edu/CS4500-F20/atlanta/blob/b32dcc7f08171a614af16d1f35993b7fb5976aaf/Fish/Common/src/client/strategy.rs#L173-L200
  - 20 lookahead: https://github.ccs.neu.edu/CS4500-F20/atlanta/blob/b32dcc7f08171a614af16d1f35993b7fb5976aaf/Fish/Common/src/client/strategy.rs#L202-L252
  
- any game-tree functionality you had to add to create the `xtest` test harness:
  - where the functionality is defined in `game-tree.PP`
  - where the functionality is used in `xtree`
  - you may wish to submit a `git-diff` for `game-tree` and any auxiliary modules 

  - N/a: We did not have to add any game-tree functionality for xtest or xtree.
    We did add more to the documentation explaining why we had no is-stuck state
    as well as making one function public that we forgot beforehand. The following
    git diff is from the time of writing this self-eval to before we wrote xtree:
    ```
        diff --git a/Fish/Common/src/common/game_tree.rs b/Fish/Common/src/common/game_tree.rs
        index f8c5565..566ffed 100644
        --- a/Fish/Common/src/common/game_tree.rs
        +++ b/Fish/Common/src/common/game_tree.rs
        @@ -12,6 +12,10 @@ use std::collections::HashMap;
        /// game at that point in time.
        /// Uses lazy evaluation to avoid storing the entire data structure
        /// in memory. See the LazyGame struct for more info.
        +/// 
        +/// Note that there is no case when a player is stuck; we simply
        +/// skip their turn if they have no moves and move
        +/// to the next Turn state.
        pub enum Game {
            Turn { state: GameState, valid_moves: HashMap<Move, LazyGame> },
            End(GameState),
        @@ -55,7 +59,7 @@ impl Game {
        
            /// Returns the `Game` that would be produced as a result of taking the given Move.
            /// If the move is invalid (not in valid_moves or self is `End`) then None is returned
        -    fn get_game_after_move(&mut self, move_: Move) -> Option<&Game> {
        +    pub fn get_game_after_move(&mut self, move_: Move) -> Option<&Game> {
                match self {
                    Game::Turn { valid_moves, .. } => {
                        valid_moves.get_mut(&move_).map(|lazy_game| lazy_game.get_evaluated())
    ```

**Please use GitHub perma-links to the range of lines in specific
file or a collection of files for each of the above bullet points.**

  WARNING: all perma-links must point to your commit "b32dcc7f08171a614af16d1f35993b7fb5976aaf".
  Any bad links will result in a zero score for this self-evaluation.
  Here is an example link:
    <https://github.ccs.neu.edu/CS4500-F20/atlanta/tree/b32dcc7f08171a614af16d1f35993b7fb5976aaf/Fish>

