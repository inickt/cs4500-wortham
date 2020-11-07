## Self-Evaluation Form for Milestone 6

Indicate below where your TAs can find the following elements in your strategy and/or player-interface modules:

The implementation of the "steady state" phase of a board game
typically calls for several different pieces: playing a *complete
game*, the *start up* phase, playing one *round* of the game, playing a *turn*, 
each with different demands. The design recipe from the prerequisite courses call
for at least three pieces of functionality implemented as separate
functions or methods:

- the functionality for "place all penguins"
  - Our players are allowed to determine where to place their own penguins. 
  They place them in their turn order, rather than all at once. 
  - Player placement code: https://github.ccs.neu.edu/CS4500-F20/atlanta/blob/9f9a4b055e8b259d8c32c60c1bd6f8dfe4e021de/Fish/Common/src/client/player.rs#L52-L55
  - Referee placement code: https://github.ccs.neu.edu/CS4500-F20/atlanta/blob/9f9a4b055e8b259d8c32c60c1bd6f8dfe4e021de/Fish/Common/src/server/referee.rs#L149-L156
 
- a unit test for the "place all penguins" funtionality 
  - https://github.ccs.neu.edu/CS4500-F20/atlanta/blob/9f9a4b055e8b259d8c32c60c1bd6f8dfe4e021de/Fish/Common/src/client/player.rs#L111-L122

- the "loop till final game state"  function
  - https://github.ccs.neu.edu/CS4500-F20/atlanta/blob/9f9a4b055e8b259d8c32c60c1bd6f8dfe4e021de/Fish/Common/src/server/referee.rs#L62-L72
- this function must initialize the game tree for the players that survived the start-up phase
  - update_from_gamestate/update_gametree_position perform the actual game tree update: https://github.ccs.neu.edu/CS4500-F20/atlanta/blob/9f9a4b055e8b259d8c32c60c1bd6f8dfe4e021de/Fish/Common/src/common/gamephase.rs#L80-L112
  - These update functions are called from here in the player struct: https://github.ccs.neu.edu/CS4500-F20/atlanta/blob/9f9a4b055e8b259d8c32c60c1bd6f8dfe4e021de/Fish/Common/src/client/player.rs#L89-L99

- a unit test for the "loop till final game state"  function
  - https://github.ccs.neu.edu/CS4500-F20/atlanta/blob/9f9a4b055e8b259d8c32c60c1bd6f8dfe4e021de/Fish/Common/src/server/referee.rs#L252-L267


- the "one-round loop" function
  - Our code does not distinguish between a round and a turn - we just loop taking turns until the game is over. The closest we have is our loop in run_game:
  - https://github.ccs.neu.edu/CS4500-F20/atlanta/blob/9f9a4b055e8b259d8c32c60c1bd6f8dfe4e021de/Fish/Common/src/server/referee.rs#L66-L69

- a unit test for the "one-round loop" function
  - Our code does not distinguish between a round and a turn - we just loop taking turns until the game is over. The closest we have is our loop in run_game:
  - https://github.ccs.neu.edu/CS4500-F20/atlanta/blob/9f9a4b055e8b259d8c32c60c1bd6f8dfe4e021de/Fish/Common/src/server/referee.rs#L252-L267




- the "one-turn" per player function
  - referee::do_player_turn: https://github.ccs.neu.edu/CS4500-F20/atlanta/blob/9f9a4b055e8b259d8c32c60c1bd6f8dfe4e021de/Fish/Common/src/server/referee.rs#L127-L142
  - player's take_turn: https://github.ccs.neu.edu/CS4500-F20/atlanta/blob/9f9a4b055e8b259d8c32c60c1bd6f8dfe4e021de/Fish/Common/src/client/player.rs#L45-L62

- a unit test for the "one-turn per player" function with a well-behaved player 
  - The one turn function is private, so we tested this functionality using our public run_game function.
  - https://github.ccs.neu.edu/CS4500-F20/atlanta/blob/9f9a4b055e8b259d8c32c60c1bd6f8dfe4e021de/Fish/Common/src/server/referee.rs#L252-L267


- a unit test for the "one-turn" function with a cheating player
  - The one turn function is private so we don't directly test it. Our only public referee function is run_game, so we test a cheating player through that.
  - https://github.ccs.neu.edu/CS4500-F20/atlanta/blob/9f9a4b055e8b259d8c32c60c1bd6f8dfe4e021de/Fish/Common/src/server/referee.rs#L300-L335


- a unit test for the "one-turn" function with an failing player 
  - We assumed the first failing case (player timeout) could be done at a
  future date since the assignment said we could leave connection-related tasks at a later date. Here is our todo for timeouts: https://github.ccs.neu.edu/CS4500-F20/atlanta/blob/9f9a4b055e8b259d8c32c60c1bd6f8dfe4e021de/Fish/Common/src/server/referee.rs#L26
  - For the other failing case of sending invalid json however, we did not test this.


- for documenting which abnormal conditions the referee addresses 
  - https://github.ccs.neu.edu/CS4500-F20/atlanta/blob/9f9a4b055e8b259d8c32c60c1bd6f8dfe4e021de/Fish/Common/src/server/referee.rs#L22-L26


- the place where the referee re-initializes the game tree when a player is kicked out for cheating and/or failing 
  - We missed this case.
  - We mutate the gamestate but do not update the gametree here: https://github.ccs.neu.edu/CS4500-F20/atlanta/blob/9f9a4b055e8b259d8c32c60c1bd6f8dfe4e021de/Fish/Common/src/server/referee.rs#L188


**Please use GitHub perma-links to the range of lines in specific
file or a collection of files for each of the above bullet points.**

  WARNING: all perma-links must point to your commit "9f9a4b055e8b259d8c32c60c1bd6f8dfe4e021de".
  Any bad links will be penalized.
  Here is an example link:
    <https://github.ccs.neu.edu/CS4500-F20/atlanta/tree/9f9a4b055e8b259d8c32c60c1bd6f8dfe4e021de/Fish>

