# royalur (Royal Game of Ur)

The Royal Game of Ur is one the oldest board games, dating back to at least 5000 years ago. [This excellent video](https://www.youtube.com/watch?v=WZskjLq040I) gives a thorough introduction to this game, explaining its rules from [3:41](https://www.youtube.com/watch?v=WZskjLq040I&t=221s) to 5:10.

{{#include royalur-board.svg}}

The Royal Game of Ur is a race game. Each player has 7 tokens, and it wants to send all tokens to the end of the track before the opponent does. Each player has a track of 14 cells (not counting start and end), with the cells from the fifth to the twelfth (included) shared between players. The game is played in turns alternating between the two players until one of them make all of their tokens exit the track, winning the game.

At the start of its turn, the player flips 4 two-sided coins, which will give a number \\( n \\) of heads. The player must then choose a token to move \\( n \\) cells forward. For a token, to exit the track, the exact number of cells remaining is needed (e.g. if a token is on the fourteenth cell, only a \\( 1 \\) can make it exit the track). If no moves are possible for the player (such as when getting a \\( 0 \\), but it's not the only such case) then it skips the turn, giving it to the other player.

The player cannot move a token in a cell already occupied by one of its token, however it can move it in a cell occupied by an opponent's token. In this case, the opponent's token gets instantly sent back to the opponent's track start, and the cell becomes occupied by the player token.

If a token lands on the fourth, eighth or fourteenth cell, the player gets to play also for the next turn, otherwise the opponent's turn begins. A token cannot land on one of these 3 cells if it's already occupied (even by an opponent's token).

## Implementation details
At the beginning of the game both players will receive 3 lines:

- the first line contains the name of the first player;
- the second line contains the name of the second player;
- the third line contains `0` if the receiver is the first player, `1` otherwise.

Spectators will only receive the first two lines. In the first turn the first player will play.

Each turn is subdivided into two sub-turns:

- `roll`;
- `move`.

In the `roll` sub-turn both players and spectators will receive a single line containing 4 space-separated binary digits. The `1`s represent a head, the `0` a tail. The amount \\( n \\) is calculated as the number of `1`s obtained.

If the player has no valid moves that move a token \\( n \\) cells forward, the the `move` sub-turn is skipped and the `roll` sub-turn immediately starts for the other player.

Otherwise, the `move` sub-turn begins. Each player has `7` tokens, numbered from `0` to `6` (inclusive). The player playing in this turn must write a single line with the number of the token he wants to move forward \\( n \\) cells, ended with a `LF` (aka `\n`). The move must be valid. If the move is valid, the other player and the spectators will receive the sent number, otherwise they will receive `RETIRE`, which indicates that the game has ended with a win for the opponent.

If the token lands on the fourth, eighth or fourteenth cell than the player has another turn, thus moving again to the `roll` sub-turn. Otherwise, the turns pass to the opponent, which begins its `roll` turn.

### Example
This is an example of the streams of two players, `PlayerA` and `PlayerB`, and the spectators for an hypothetical game.

Note that all lines prepended with a `>` indicate that the line is sent rather than received.

Stream of `PlayerA`:
```text
PlayerA
PlayerB
0
0 0 1 1
>3
1 1 1 1
0
1 0 1 0
0
0 1 0 0
>3
0 0 0 0
1 0 1 1
>3
0 0 1 1
0
0 0 0 1
>4
1 0 1 0
RETIRE
```

Stream of `PlayerB`:
```text
PlayerA
PlayerB
1
0 0 1 1
3
1 1 1 1
>0
1 0 1 0
>0
0 1 0 0
3
0 0 0 0
1 0 1 1
3
0 0 1 1
>0
0 0 0 1
4
1 0 1 0
>2
```

Stream of spectators:
```text
PlayerA
PlayerB
0 0 1 1
3
1 1 1 1
0
1 0 1 0
0
0 1 0 0
3
0 0 0 0
1 0 1 1
3
0 0 1 1
0
0 0 0 1
4
1 0 1 0
RETIRE
```


## Game parameters
There's only one game-specific parameter:

- `pace`: the minimum number of seconds between turns (default: `1.5`, min: `0`, max: `30`).

Additional information:

- the game can only be played by exactly `2` players;
- the default timeout is `90` seconds;
- no more than `1` server bot per game is allowed.
