# Player API Protocol

Authors: Jake Fecher and Ryan Drew
Repo: atlanta

## Stages of the Tournament
1. Sign-up
    - The tournament manager accepts SignUpMessages from players.
    ```json
    {
        "type": "SignUp",
    }
    ```
    - It responds to these with SignUpResponses.
    ```json
    {
        "error": <string> | false
    }
    ```

2. Tournament Rounds

3. Statistics Collection
