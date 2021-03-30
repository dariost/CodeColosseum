# roshambo (Rock Paper Scissors)

**Rock Paper Scissors** (also called _roshambo_) is one of the most basic hand games.

It is played by two players, which in each round choose one of the gestures _paper_, _rock_ or _scissors_ and show them at the same time. If the two gestures chosen are different, the winner is computed as follows:

- _paper_ wins over _rock_
- _rock_ wins over _scissors_
- _scissors_ win over _paper_

For each round a point is awarded to the winner (if any). The player with most points at the end wins.

## Implementation details
Both players and the spectators will receive three lines at the beginning of the match, containing the two players names and the number of rounds. Each player will receive its own name as the first line.

For instance, if there are two players `Player0` and `Player1` in a match with `42` rounds, then:

`Player0` will receive:
```text
Player0
Player1
42
```

`Player1` will receive:
```text
Player1
Player0
42
```

Spectators will receive:
```text
Player0
Player1
42
```

Then, for the number of rounds specified, both players will have to send their choice of move to the server using one of the strings `ROCK`, `PAPER` or `SCISSORS` with a single `LF` (aka `\n`) at the end. After sending a move choice, they will have to read a single line containing the choice of the opponent. Other then the opponent choice, the string `RETIRE` can also be received, which means the opponent as retired and the player must exit the match. This cycle will then repeat for the remaining rounds.

Spectators will receive for each round two lines containing the choice of the two players. The lines are given in the same order as players names at the beginning of the stream.

For instance, if in a round `Player0` chooses `ROCK` and `Player1` chooses `PAPER`, then (sent lines are prefixed with a `>`):

`Player0` will receive:
```text
>ROCK
PAPER
```

`Player1` will receive:
```text
>PAPER
ROCK
```

Spectators will receive:
```text
ROCK
PAPER
```

A complete game can look like this:

`Player0` will receive:
```text
Player0
Player1
3
>ROCK
PAPER
>PAPER
PAPER
>ROCK
SCISSORS
```

`Player1` will receive:
```text
Player1
Player0
3
>PAPER
ROCK
>PAPER
PAPER
>SCISSORS
ROCK
```

Spectators will receive:
```text
Player0
Player1
3
ROCK
PAPER
PAPER
PAPER
ROCK
SCISSORS
```

## Game parameters
There are two game specific parameters available:

- `rounds`: specifies the number of rounds;
- `pace`: specifies a minimum time interval (in seconds) between rounds.
