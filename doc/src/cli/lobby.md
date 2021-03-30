# lobby

This subcommand can be used to list waiting to start and running matches currently
present in the lobby:

```shell
$ coco -s wss://code.colosseum.cf/ lobby
```

This will print a table with information about the matches:

- **ID**: the unique identifier of the match;
- **Verified**: whether the game has been created by a server admin;
- **Name**: the name of the match;
- **Game**: the game to be played;
- **Players**: the number of currently connected players over the number of players needed to start the match;
- **Spectators**: the number of currently spectators connected to this match;
- **Timeout**: the timeout (in seconds) for players send some (valid) data before getting forcibly disconnected by the server;
- **Password**: whether a password is needed to join the match;
- **Timing**: expiration information for waiting to start matches and running time for running matches.
