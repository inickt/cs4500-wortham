
------------------------------------------------------------
Design Inspection 20/30
------------------------------------------------------------

-5 unclear what happens if a wrong player tries to make a move;
 `Move` doesn't have a player info

-5 planning does not mention the ability to plan for multiple moves ahead

------------------------------------------------------------
Misc Comments
------------------------------------------------------------

I wonder if it would be possible to make some helper functions to simplify
unit tests. (There is a fair bit of boilerplate.)

Please, add an explanation of how holes are represented in the board interpretation.

The code seems to be reasonably well structured.
Good testing.
Nice README.

MILESTONE 4

-10 no/insufficient/misleading interpretation of the game state
  how players are related to penguins and how penguins' locations are tracked,
  what is the order of players and how they take turns

-10 it is unclear if the game tree node can represent all three kinds of nodes:
 game-is-over, current-player-is-stuck, and current-player-can-move;
 How to distinguish between game-is-over and current-player-is-stuck

-3 insufficient description of the player's API wrt to a referee
  mention the end game


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
