# Player API Protocol

Authors: Jake Fecher and Ryan Drew
Repo: atlanta

## Stages of the Game
1. Setup
    - The server/referee sets up a Board and TCP connections to each player.
    - Between 2-4 players may be connected for a given game.
    - Players do not need to interact with the server while they're
      waiting for the game to be setup - they need only to interact with
      the tournament server to initially register (not covered in this document).
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
    - Note that since the NewGameState message sent every turn already
    tells the player if the game has ended (and because the player cannot
    communicate with a referee after the game ends) there is no special
    API needed for a player after the game has ended. If they want to
    play another game they can restart the program or enter a tournament
    where they will be queued into multiple games.

## Possible Player->Server Messages
1. PlacePenguin: Places an unused penguin on the given tile. Can only
be sent when a player has unplaced penguins. The player that will place
the penguin is determined by the TCP connection info of the client that
sent this message. If the placement is invalid, that player will be kicked from the
game and have all their penguins removed.
```json
{
    "PlacePenguin": {
        "tile_id": number
    }
}
```

2. MovePenguin: Moves an existing penguin owned by the current player from the 
tile it is currently on to the specified tile. The specified tile must be in a 
straight line from the current tile with no holes, and this message can only be 
sent when a player has no unplaced penguins. The player that will move
the penguin is determined by the TCP connection info of the client that
sent this message. If the move is invalid, that player will be kicked from the
game and have all their penguins removed.
```json
{
    "MovePenguin": {
        "penguin_id": number,
        "tile_id": number
    }
}
```

## Server->Client Message
1. NewGameState

At the start of each turn the server will send the entire game state to the clients.
This message will only be sent in the "Avatar Placement", "GamePlay", and "End" game
stages. After taking their turn, clients should continuously receive these messages to see how
the gamestate changes as a result of each player's turn in sequence. When the server sends
a gamestate with the `gamestate.current_turn` matching the client's PlayerId, it is now
that client's turn and they should then take their turn by sending a PlacePenguin or
MovePenguin message.

The full gamestate message is:
```json
{
    "board": Board,
    "players": [{ "0": PlayerId, "1": Player }, ...],  // Invariant: length is between 2 and 4 inclusive
    "turn_order": [PlayerId, ...],  // Invariant: length is between 2 and 4 inclusive
    "current_turn": PlayerId,
    "winning_players": [PlayerId, ...] | null
}
```

Note that the `[Elem, ...]` syntax above describes a json array where each element is an `Elem`.

With PlayerId = `number`
and PenguinId = `number`
and TileId = `number`
and Board =
```json
{
    "tiles": [Tile, ...],
    "width": number,  // Number of columns this board has
    "height": number  // Number of rows this board has
}
```

and Tile =
```json
{
    tile_id: TileId,
    fish: number,

    northwest: TileId | null,
    north: TileId | null,
    northeast: TileId | null,
    southwest: TileId | null,
    south: TileId | null,
    southeast: TileId | null,
}
```

and Player =
```json
{
    "player_id": PlayerId,
    "color": "blue" | "green" | "pink" | "purple",
    "penguins": [Penguin, ...]
}
```

and Penguin =
```json
{
    "penguin_id": PenguinId,
    "tile": TileId | null,
}
```