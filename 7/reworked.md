# Milestone 2 Feedback

> unit tests for reachable-positions functionality does not check all directions

Added test to check all reachable directions in tile.rs:274

# Milestone 3 Feedback

> no tests for checking that the player turn changes correctly

Added test for our "advance_turn" function to ensure the turn is changed
correctly.

> unclear what happens if a wrong player tries to make a move; `Move` doesn't have a player info

Added clarification in games.md:28

> planning does not mention the ability to plan for multiple moves ahead

Added clarification in games.md:26-27

> Please, add an explanation of how holes are represented in the board interpretation.

board.rs:28