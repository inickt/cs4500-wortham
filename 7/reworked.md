> [x] Add an explanation of how holes are represented in the board interpretation.
*   Clarify how holes are represented in both the board and in the neighbors of each tile.

#### Original Feedback:
> Please, add an explanation of how holes are represented in the board interpretation.

#### Approach to remove deficiency:
Added documentation at board.rs:28.
https://github.ccs.neu.edu/CS4500-F20/muleshoe/blob/3444536cf6243635922c85fce66a99f311966555/Fish/Common/src/common/board.rs#L28-L31

Explained that holes are just the absense of a tile from the map of tiles in the Board struct
and to create a hole, simply remove a tile from that map and unlink it from its neighbors.

---

> [x] Unit tests for reachable positions functionality does not check all directions from a given tile.

#### Original Feedback:
> unit tests for reachable-positions functionality does not check all directions

#### Approach to remove deficiency:
Added test to check all reachable directions in tile.rs:274.
https://github.ccs.neu.edu/CS4500-F20/muleshoe/blob/3444536cf6243635922c85fce66a99f311966555/Fish/Common/src/common/tile.rs#L274-L293

This test sets up a 5x3 board and checks all the reachable positions of the middle tile (tile ID 7). This tile is
completely surrounded on all sides.

---

> [x] Unit tests are needed for ensuring that advancing the player's turn works correctly.

#### Original Feedback:
> no tests for checking that the player turn changes correctly

#### Approach to remove deficiency:
Added test for "advance_turn" function in gamestate.rs:470.
https://github.ccs.neu.edu/CS4500-F20/muleshoe/blob/3444536cf6243635922c85fce66a99f311966555/Fish/Common/src/common/gamestate.rs#L469-L481

This test ensures the turn is changed correctly, including moving the turn from
the last player to the first player in the turn order.

---

> [x] Clarify what happens if a wrong player attempts to make a move.
*   Describe why we used PenguinID without also providing a PlayerID.

#### Original Feedback:
> unclear what happens if a wrong player tries to make a move;
 `Move` doesn't have a player info

#### Approach to remove deficiency:
Added clarification in games.md:29.
https://github.ccs.neu.edu/CS4500-F20/muleshoe/blame/3444536cf6243635922c85fce66a99f311966555/Fish/Planning/games.md#L29

PenguinIds uniquely identify a Player so having an extra PlayerID was
superfluous. Also explained that in a real game, a cheating player may
try to send a move out of turn for another player. The referee is in
charge of managing the connection information of each player to prevent
this.

---

> [x] Clarify how the referee handles the end of a game.

#### Original Feedback:
>  insufficient description of the player's API wrt to a referee
  mention the end game

#### Approach to remove deficiency:
Added clarification to player-protocol.md:28.
https://github.ccs.neu.edu/CS4500-F20/muleshoe/blame/master/Fish/Planning/player-protocol.md#L28-L33

Clarified that the standard server message sending the gamestate at
each turn also identifies when the game is over so a separate game over
message is unneeded. Also clarified that after a game is over, a
player doesn't need any API for communicating with a referee since there
is nothing left to do. A human player may wish to manually restart the game
or join a tournament to play again.

---

> [x] Specify how the referee handles players who do not make an action.

#### Original Feedback:
> the document does not specify how the referee deals with players
 that fail to act  (e.g. player crashes, or doesn't respond within a time limit)
 In such a case, no message would be received from player's side!

#### Approach to remove deficiency:
Added clarification in referee.md:52.
https://github.ccs.neu.edu/CS4500-F20/muleshoe/blame/master/Fish/Planning/referee.md#L52-L53

Added sentence clarifying if a player fails to send an action within the
timeout of 30 seconds then they are kicked from the game.

---

> [x] Describe how a game tree is used to plan multiple moves ahead.

#### Original Feedback:
> planning does not mention the ability to plan for multiple moves ahead

#### Approach to remove deficiency:
Added clarification in games.md:26-27
https://github.ccs.neu.edu/CS4500-F20/muleshoe/blame/3444536cf6243635922c85fce66a99f311966555/Fish/Planning/games.md#L26-L27

Added sentence clarifying how a game tree can be recursively analyzed to 
plan ahead with each child node representing 1 turn passing in the game.
