> [x] Add an explanation of how holes are represented in the board interpretation.
*   Clarify how holes are represented in both the board and in the neighbors of each tile.

#### Original Feedback:
> Please, add an explanation of how holes are represented in the board interpretation.

#### Approach to remove deficiency:
Added documentation at board.rs:28

---

> [x] Unit tests for reachable positions functionality does not check all directions from a given tile.

#### Original Feedback:
> unit tests for reachable-positions functionality does not check all directions

#### Approach to remove deficiency:
Added test to check all reachable directions in tile.rs:274

---

> [x] Unit tests are needed for ensuring that advancing the player's turn works correctly.

#### Original Feedback:
> no tests for checking that the player turn changes correctly

#### Approach to remove deficiency:
Added test for our "advance_turn" function to ensure the turn is changed
correctly.

---

> [x] Clarify what happens if a wrong player attempts to make a move.
*   Describe why we used PenguinID without also providing a PlayerID.

#### Original Feedback:
> unclear what happens if a wrong player tries to make a move;
 `Move` doesn't have a player info

#### Approach to remove deficiency:
Added clarification in games.md:28

---

> [x] Clarify how the referee handles the end of a game.

#### Original Feedback:
>  insufficient description of the player's API wrt to a referee
  mention the end game

#### Approach to remove deficiency:
Added clarification to player-protocol.md:28

---

> [x] Specify how the referee handles players who do not make an action.

#### Original Feedback:
> the document does not specify how the referee deals with players
 that fail to act  (e.g. player crashes, or doesn't respond within a time limit)
 In such a case, no message would be received from player's side!

#### Approach to remove deficiency:
Added clarification in referee.md:52

---

> [x] Describe how a game tree is used to plan multiple moves ahead.

#### Original Feedback:
> planning does not mention the ability to plan for multiple moves ahead

#### Approach to remove deficiency:
Added clarification in games.md:26-27
