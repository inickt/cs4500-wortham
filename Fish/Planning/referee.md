# The Referee

Authors: Jake Fecher and Ryan Drew

Repo: atlanta

## Overview

The referee is the component responsible for managing a single
game of Fish. It is handed the players at the beginning of the game,
along with their connection info. It creates the board, either by its
own criteria or from an existing Board. It then accepts placement messages,
then movement messages, sending gamestates in response to each to show
how the board changes throughout the game. Finally, it sends
a GameState with some winning players, signaling that the game is over.
It then closes the connections, and ends the Referee lifecycle.

## Capabilities

Each of the following functions roughly follow the stages of the game
as given in player-protocol.md. 

### Set up the game
1. Create/decide the initial game state and then run the game with the
   given vector of player connections ordered by their turn order.
   - If Some(board) is passed the game will use that board, otherwise
   the referee may start the game with any board it wishes that is
   at least large enough to accommodate the penguins of every player.
   - This function is expected to continue until the game finishes,
   handling each subsequent step in this document internally.
   - This function is expected to create any Referee-specific data
   it needs to run the game internally (e.g. a map from PlayerId -> TcpStream).
   All subsequent functions have an `&mut self` parameter that
   refers to this Referee specific data.

    ```rust
    fn run_game(players: Vec<TcpStream>, board: Option<Board>) {}
    ```

### Run the gameplay (The Avatar Placeament & Gameplay Stages)
On each turn:
1. Send the current gamestate to all players and spectators via the
   registered TCP connections of each player given on the call to run_game.

    ```rust
    fn send_state_to_all_observers(&mut self) {}
    ```

2. Accept place penguin messages from the current player if they have
   unplaced penguins, or move messages otherwise. The format of these messages
   is given in player-protocol.md. Failure to give a message in the correct
   format or failure to give a message at all within the timeout of 30 seconds
   will result in the player being kicked from the game and their
   penguins removed from the board.
   - Process and validate the received message sent from a player
     according to the rules of Fish. If the message is invalid in any
     way, kick the player from the game. If the message is valid and
     correct, place or move the penguin as specified in the message.
   - A player will know their move succeeded if they receive a GameState
     on the next turn. If they made an illegal move, their connection will
     be closed immediately (they will be kicked) by the referee and their
     penguins will be removed from the board.

    ```rust
    fn accept_player_message(&mut self, json: Json) {}
    ```

3. End the game if no players have any moves left (See next section).
   Otherwise proceed to the next turn.

### Shut down the game (End Stage)
1. Notify every observer (players + spectators) of the winners, losers, and
   cheaters of the game by sending
   the final GameState containing this as well the scores of all players, final
   Board state, and all other fields of the GameState struct.
   This is done with the aforementioned `send_state_to_all_observers` function.
2. Close TCP connections with all players.
   ```rust
   fn end_game(&mut self) {}
   ```