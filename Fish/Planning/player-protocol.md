# Player API Protocol

Authors: Jake Fecher and Ryan Drew
Repo: atlanta

## Stages of the Game
1. Connection
    - Players will connect to the game server using "PlayerConnect" messages.
    - A maximum of four players can connect to a game server.
    - The game will start when there are two to four connected players,
    based on the configuration settings for the server.
    - Once a player has received a PlayerConnect message,
    they will not receive any other messages from the server until the
    game starts and the Avatar Placement phase begins.
2. Avatar Placement
    - Begins when players receive first NewGameState message.
    - Players can only send PlacePenguin messages, and will be
    kicked if they send a message when it is not their turn.
3. Gameplay
    - Starts when each player has placed all of their penguins,
    i.e. no player in the GameState has a penguin with null for
    the penguin's tile_id in the serialized GameState json.
    - Players can only send MovePenguin messages, and will be
    kicked if they send a message when it is not their turn.
4. End
    - When the GameState in the NewGameState message contains a
    winning_players list that is non-empty, the game ends.
    - Once this NewGameState message is sent, the connection will be
    closed by the server and the game will have ended.

## Possible Player->Server Messages
1. PlayerConnect: Connects a player to the game. This
message may fail if the game is full, or the game has
already started. As such, it should only be sent to an active game
server before the game has started.
```json
{
    "type": "PlayerConnect",
    "age": number
}
```

The server will respond with a PlayerConnected message (see below).

2. PlacePenguin: Places an unused penguin on the given tile. Can only
be sent when a player has unplaced penguins. The player that will place
the penguin is determined by the TCP connection info of the client that
sent this message. If the placement is invalid, that player will be kicked from the
game and have all their penguins removed.
```json
{
    "type": "PlacePenguin",
    "tile_id": number
}
```

3. MovePenguin: Moves an existing penguin owned by the current player from the 
tile it is currently on to the specified tile. The specified tile must be in a 
straight line from the current tile with no holes, and this message can only be 
sent when a player has no unplaced penguins. The player that will move
the penguin is determined by the TCP connection info of the client that
sent this message. If the move is invalid, that player will be kicked from the
game and have all their penguins removed.
```json
{
    "type": "MovePenguin",
    "penguin_id": number,
    "next_tile_id": number
}
```

## Server->Client Message

1. PlayerConnected

Sent to a client after the client sends a PlayerConnect message to join a game.
This message indicates if the join was successful or not. If the connection was
successful, the player's PlayerId is given as a natural number. Otherwise, an
error message is returned instead of the player id. Note that the server will
only let player's join during the Connect stage of the game. If the game is in
any other stage, the server will always send back a PlayerConnected message
containing an error_message rather than a player_id.

```json
// Successfully connected:
{
    "type": "PlayerConnected",
    "player_id": natural
}
// Could not connect:
{
    "type": "PlayerConnected",
    "error_message": string,
}
```

2. NewGameState

At the start of each turn the server will send the entire game state to the clients.
This message will only be sent in the "Avatar Placement", "GamePlay", and "End" game
stages. After taking their turn, clients should continuously receive these messages to see how
the gamestate changes as a result of each player's turn in sequence. When the server sends
a gamestate with the `gamestate.current_turn` matching the client's PlayerId, it is now
that client's turn and they should then take their turn by sending a PlacePenguin or
MovePenguin message.

```json
{
    "type": "NewGameState",
    "state": GameState
}

The precise JSON spec of the serialized GameState is non-final and is thus not listed here
as it is subject to change. Players using the Fish client as a rust library may freely use
the provided Serialize/Deserialize impls for GameState. Otherwise, players using only
the json interface to communicate may wish to take in an arbitrary json value to make sure
they're compatible with any GameState json given by the fish game server.
```