# Lobby

**Code Colosseum** provides no user authentication by design. The rationale is
to keep its codebase and its setup as simple as possible. This, however, comes
at the cost of having a (bit) more complicated match setup.

**Code Colosseum** keeps a lobby of waiting to start and running matches. The lobby
serves as the sole access point for players to join a match waiting to start.

The command line client provides a way to show the current status of the lobby.
Other clients could provide it as well.

## Match Creation

A new match can be created by anyone, by either using the command line client or
some other mean. When creating a match the user must specify the following parameters:

- game to be played
- name of the match
- number of players
- number of server bots
- timeout for player inactivity
- an optional password to join the match
- optionally the server password to create a verified game
- other parameters specific to the game requested

Note that, depending on the client used, some parameters could have defaults, and
as such will not be needed to be specified. Also note that, depending on the game
chosen, not all possible values of such parameters will be available (e.g. for
Rock Paper Scissors creating a match with a number of players rather than 2 will
cause an error and the match will not be created). Such restrictions will be
indicated on the description of each game.

A match starts as soon as the number of players needed to start it has been reached.
If a waiting-to-start match is inactive for some time, it will be removed from the
lobby. The command line client provides an indication of such expiration time.

When created, the server will return the match ID, which is the identifier needed
to join or spectate a match.

## Spectators

**Code Colosseum** provides the ability to spectate any match, either be waiting
to start or already running. Spectators can spectate password protected matches
without needing a password. Thus, in the spectator sense, all matches are public.

Spectators will get a read-only stream of the match, the contents of which are
specified in the description of each game. Note that, like the stream for playing
parties, the stream is not supposed to be human-readable, and as such will likely
need to be connected to a program that renders it in a human-readable way.

Note that some games provide such rendering programs, some even with a graphical
user interface.
