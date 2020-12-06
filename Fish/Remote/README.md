### File Organization

Due to cargo requiring all code go in a src/ directory, there is unfortunately
no code in this Remote/ directory. Instead, all signup-related code is in
../Common/src/server/signup.rs

### Modifications

Quite a few changes were needed to adapt the code to using the new messages and
deserialization:

- PenguinIds were removed because the json messages did not contain them. Penguins
  were changed to be identified by the id of the tile they were on instead.
  - Went back and fixed the integration tests that failed to compile as a result
    of this change
- The ClientProxy enum was replaced with the Client trait partially due to past
  code feedback and partially to have a better client interface for the new messages
  rather than a send/receive function which only took Strings.
- ServerToClient and ClientToServer enums were made with the new message types.
- The previous ClientProxy::Remote was changed to RemoteClient and now has a corresponding
  ClientToServerProxy for the client-side end of this connection. This client-side can
  take in another Client which lets you have remote ai players or remote human players.
- Referee no longer sends the current game state to all players before every turn. Instead
  it only sends the current game state to the client whose turn it currently is.
