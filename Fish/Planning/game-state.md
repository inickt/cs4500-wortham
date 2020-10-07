# Memo 10/7

## Game State

Our shared game state between client and server will be
a struct that includes the following fields:
- Board
    - This is a Board struct which holds references to the individual Tiles
    that comprise the Board.
    - Each Tile knows how many fish are contained on it as well as
    its position relative to other tiles.
    - The Board also knows the amount of rows and columns of tiles it holds.
- List of Players
    - This is a list of the structs representing the different players
    in the current game.
    - Each Player struct contains information known to all players,
    which includes public information such as names, PlayerIds, colors,
    and amount of fish collected, but not private information such as TCP 
    connections.
    - Each Player struct also contains the list of that player's penguins. We 
    represent Penguins using the Option type; if a penguin is on the board, we 
    use Some(id, x, y), whereas if it has yet to be placed, the penguin is 
    represented by None. The x, y tuple representing the Penguin's position
    is a board position, rather than a position in pixels.
    - Referees will have an external mapping from PlayerId to additional 
    information about each player that is private to the server 
    (TCP connection, age, etc.). This will not be part of the Game State.
- Turn Order
    - This is a list containing each PlayerId in the order in which
    they will take their turns.
- Current Turn
    - This is the index in Turn Order list representing current turn.
- Winning Players
    - This is an Optional type, where it is either Some(list) containing 
    PlayerIds of the winning players, or it is None when the game is ongoing.
- Game ID
    - This is the number representing the ID of this game. It is only guaranteed
    to be unique among games set up by a tournament server.
- Spectator Count
    - A non-negative integer representing the number of spectators currently
    viewing the game.


## External Interface

### Referee to Client Messaging Spec
At the beginning of each turn, each player is sent the
full updated game state from the referee. The Game State will be
serialized into a JSON message and sent over the TCP protocol. Since
the game state includes the current player's turn, tiles,
and penguin positions, players will always be able to make an
informed decision about their turn.

If the Winning Players list in the Game State takes
a Some value, the server will stop sending messages and the game is over.

### Client to Referee Messaging Spec
When a player wants to make a turn, they must send one of several messages.
Take note of the conditions under which each message can be sent;
failure to adhere to these conditions will result in termination
from the game. If the Winning Players list in the Game State takes
a Some value, the server will stop accepting messages and the game is over.

1. PlacePenguin: Places an unused penguin on the given tile. Can only
be sent when a player has unplaced penguins.
```json
{
    "type": "PlacePenguin",
    "tile_id": number
}
```

2. MovePenguin: Moves an existing penguin owned by the current player from the 
tile it is currently on to the specified tile. The specified tile must be in a 
straight line from the current tile with no holes, and this message can only be 
sent when a player has no unplaced penguins.
```json
{
    "type": "MovePenguin",
    "penguin_id": number,
    "next_tile_id": number
}
```

3. SpectateGame: Adds a spectator to the game, subscribing their TCP connection
to receive the referee Game State messages every turn. Spectators should
not send any other messages. The spectator is dropped when the game ends or
the connection is closed.
```json
{
    "type": "SpectateGame",
    "game_id": number
}
```