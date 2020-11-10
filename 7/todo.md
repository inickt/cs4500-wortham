# Milestone 2 Feedback

> [x] Unit tests for reachable positions functionality does not check all directions from a given tile.

Added test to check all reachable directions in tile.rs:274

# Milestone 3 Feedback

> [x] Unit tests are needed for ensuring that advancing the player's turn works correctly.

Added test for our "advance_turn" function to ensure the turn is changed
correctly.

> [x] Clarify what happens if a wrong player attempts to make a move.
*   Describe why we used PenguinID instead of PlayerID.

Added clarification in games.md:28

> [x] Describe how a game tree is used to plan multiple moves ahead.

Added clarification in games.md:26-27

> [x] Please, add an explanation of how holes are represented in the board interpretation.

Added documentation at board.rs:28

# MILESTONE 4

> [x] insufficient description of the player's API wrt to a referee
  mention the end game

Added clarification to player-protocol.md:28

MILESTONE 5

-10 choosing turn action: purpose statement and/or data definition/interpretation
  of <INSERT SMTH LIKE Move or Action> is not clear.
  The function should return possible turn action that a player can take in current state.
  Function name suggests that turn is not changed while the purpose statement suggests otherwise.
  Neither does it tell, when significance this is function has. It has a lot of unanswered questions.

------------------------------------------------------------
Design Inspection 25/30
------------------------------------------------------------

-5 the document does not specify how the referee deals with players
 that fail to act  (e.g. player crashes, or doesn't respond within a time limit)
 In such a case, no message would be received from player's side!
