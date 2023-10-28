# Dama

The checker takes place between two players, arranged on opposite sides of a damsel, who move their pieces alternately; one player has white pieces, the other black ones. The pieces move diagonally, only on the dark squares not occupied by other pieces and have the opportunity to capture (familiarly, "eat") those opponents, bypassing them. Captured pieces are removed from the damiera and excluded from the game. The player to whom all the pieces are captured or who, on his turn of move, is unable to move, has lost.

Simple pieces (checkers) can move only one box at a time, diagonally and forward; they can also take an opposing piece by making a movement of two squares in the same direction, jumping over the opposing piece in the middle box. If in the arrival box the checker is able to make a new grip, you are in the presence of a multiple grip, which must be completed in the same game turn.

When a checker reaches the opposing base, which is the most distant line in its direction of travel, it becomes a checker. The checker is marked by placing an additional piece above the first and enjoys special powers: unlike the checkers, can move and capture both forward and backward. Ladies can’t be eaten by pawns.

To conclude, the ancient rule of the "breath", that is to capture the opposing piece that even having right, for distraction or choice had not eaten, was abolished by the Federation Checkers in 1934.

## Implementation details

At the beginning of the game the names of the players are printed on screen and it is indicated if you have the white or black checkers, then the damiera is printed and the game begins the player who owns the white checkers.

The spectators will receive the names of the players, the round and the damsel.

Each player is notified when it is his turn and during the latter will have to make a move or capture if possible. In case the move or a catch is not valid you will have to try again.

The game ends when one of the two players runs out of tokens and is later declared the winner.

### Example

This is an example of the streams of two players, `PlayerA` and `PlayerB`, and the spectators for an hypothetical game.

#### Move

Stream of `PlayerA`:

```text
PlayerA
PlayerB
Avvio la partita di dama...

   A  B  C  D  E  F  G  H
1 [ ][n][ ][n][ ][n][ ][n] 1
2 [n][ ][n][ ][n][ ][n][ ] 2
3 [ ][n][ ][n][ ][n][ ][n] 3
4 [ ][ ][ ][ ][ ][ ][ ][ ] 4
5 [ ][ ][ ][ ][ ][ ][ ][ ] 5
6 [b][ ][b][ ][b][ ][b][ ] 6
7 [ ][b][ ][b][ ][b][ ][b] 7
8 [b][ ][b][ ][b][ ][b][ ] 8
   A  B  C  D  E  F  G  H

Sei i Bianchi
Turno bianco!
E' il tuo turno.

Inserisci la pedina che vuoi muovere e poi le mosse che vuoi fare
Es > 6A 5B oppure 6A 4C 2A oppure 6A 4C 2A ...
6e 5d

   A  B  C  D  E  F  G  H
1 [ ][n][ ][n][ ][n][ ][n] 1
2 [n][ ][n][ ][n][ ][n][ ] 2
3 [ ][n][ ][n][ ][n][ ][n] 3
4 [ ][ ][ ][ ][ ][ ][ ][ ] 4
5 [ ][ ][ ][b][ ][ ][ ][ ] 5
6 [b][ ][b][ ][ ][ ][b][ ] 6
7 [ ][b][ ][b][ ][b][ ][b] 7
8 [b][ ][b][ ][b][ ][b][ ] 8
   A  B  C  D  E  F  G  H
```

Stream of `PlayerB`:

```text
PlayerA
PlayerB
Avvio la partita di dama...

   A  B  C  D  E  F  G  H
1 [ ][n][ ][n][ ][n][ ][n] 1
2 [n][ ][n][ ][n][ ][n][ ] 2
3 [ ][n][ ][n][ ][n][ ][n] 3
4 [ ][ ][ ][ ][ ][ ][ ][ ] 4
5 [ ][ ][ ][ ][ ][ ][ ][ ] 5
6 [b][ ][b][ ][b][ ][b][ ] 6
7 [ ][b][ ][b][ ][b][ ][b] 7
8 [b][ ][b][ ][b][ ][b][ ] 8
   A  B  C  D  E  F  G  H

Sei i Neri
Turno bianco!


   A  B  C  D  E  F  G  H
1 [ ][n][ ][n][ ][n][ ][n] 1
2 [n][ ][n][ ][n][ ][n][ ] 2
3 [ ][n][ ][n][ ][n][ ][n] 3
4 [ ][ ][ ][ ][ ][ ][ ][ ] 4
5 [ ][ ][ ][b][ ][ ][ ][ ] 5
6 [b][ ][b][ ][ ][ ][b][ ] 6
7 [ ][b][ ][b][ ][b][ ][b] 7
8 [b][ ][b][ ][b][ ][b][ ] 8
   A  B  C  D  E  F  G  H
```

Stream of spectators:

```text
PlayerA
PlayerB
Avvio la partita di dama...

   A  B  C  D  E  F  G  H
1 [ ][n][ ][n][ ][n][ ][n] 1
2 [n][ ][n][ ][n][ ][n][ ] 2
3 [ ][n][ ][n][ ][n][ ][n] 3
4 [ ][ ][ ][ ][ ][ ][ ][ ] 4
5 [ ][ ][ ][ ][ ][ ][ ][ ] 5
6 [b][ ][b][ ][b][ ][b][ ] 6
7 [ ][b][ ][b][ ][b][ ][b] 7
8 [b][ ][b][ ][b][ ][b][ ] 8
   A  B  C  D  E  F  G  H

Turno bianco!


   A  B  C  D  E  F  G  H
1 [ ][n][ ][n][ ][n][ ][n] 1
2 [n][ ][n][ ][n][ ][n][ ] 2
3 [ ][n][ ][n][ ][n][ ][n] 3
4 [ ][ ][ ][ ][ ][ ][ ][ ] 4
5 [ ][ ][ ][b][ ][ ][ ][ ] 5
6 [b][ ][b][ ][ ][ ][b][ ] 6
7 [ ][b][ ][b][ ][b][ ][b] 7
8 [b][ ][b][ ][b][ ][b][ ] 8
   A  B  C  D  E  F  G  H
```

#### Single catch


Stream of `PlayerA`:

```text
   A  B  C  D  E  F  G  H
1 [ ][n][ ][n][ ][n][ ][n] 1
2 [n][ ][n][ ][n][ ][n][ ] 2
3 [ ][ ][ ][n][ ][n][ ][ ] 3
4 [n][ ][b][ ][ ][ ][n][ ] 4
5 [ ][ ][ ][ ][ ][ ][ ][ ] 5
6 [b][ ][b][ ][b][ ][b][ ] 6
7 [ ][b][ ][ ][ ][b][ ][b] 7
8 [b][ ][b][ ][b][ ][b][ ] 8
   A  B  C  D  E  F  G  H

Turno nero!

   A  B  C  D  E  F  G  H
1 [ ][n][ ][n][ ][n][ ][n] 1
2 [n][ ][n][ ][n][ ][n][ ] 2
3 [ ][ ][ ][ ][ ][n][ ][ ] 3
4 [n][ ][ ][ ][ ][ ][n][ ] 4
5 [ ][n][ ][ ][ ][ ][ ][ ] 5
6 [b][ ][b][ ][b][ ][b][ ] 6
7 [ ][b][ ][ ][ ][b][ ][b] 7
8 [b][ ][b][ ][b][ ][b][ ] 8
   A  B  C  D  E  F  G  H
```

Stream of `PlayerB`:

```text
   A  B  C  D  E  F  G  H
1 [ ][n][ ][n][ ][n][ ][n] 1
2 [n][ ][n][ ][n][ ][n][ ] 2
3 [ ][ ][ ][n][ ][n][ ][ ] 3
4 [n][ ][b][ ][ ][ ][n][ ] 4
5 [ ][ ][ ][ ][ ][ ][ ][ ] 5
6 [b][ ][b][ ][b][ ][b][ ] 6
7 [ ][b][ ][ ][ ][b][ ][b] 7
8 [b][ ][b][ ][b][ ][b][ ] 8
   A  B  C  D  E  F  G  H

Turno nero!
E' il tuo turno.

Inserisci la pedina che vuoi muovere e poi le mosse che vuoi fare
Es > 6A 5B oppure 6A 4C 2A oppure 6A 4C 2A ...
3d 5b

   A  B  C  D  E  F  G  H
1 [ ][n][ ][n][ ][n][ ][n] 1
2 [n][ ][n][ ][n][ ][n][ ] 2
3 [ ][ ][ ][ ][ ][n][ ][ ] 3
4 [n][ ][ ][ ][ ][ ][n][ ] 4
5 [ ][n][ ][ ][ ][ ][ ][ ] 5
6 [b][ ][b][ ][b][ ][b][ ] 6
7 [ ][b][ ][ ][ ][b][ ][b] 7
8 [b][ ][b][ ][b][ ][b][ ] 8
   A  B  C  D  E  F  G  H
```

Stream of spectators:

```text
   A  B  C  D  E  F  G  H
1 [ ][n][ ][n][ ][n][ ][n] 1
2 [n][ ][n][ ][n][ ][n][ ] 2
3 [ ][ ][ ][n][ ][n][ ][ ] 3
4 [n][ ][b][ ][ ][ ][n][ ] 4
5 [ ][ ][ ][ ][ ][ ][ ][ ] 5
6 [b][ ][b][ ][b][ ][b][ ] 6
7 [ ][b][ ][ ][ ][b][ ][b] 7
8 [b][ ][b][ ][b][ ][b][ ] 8
   A  B  C  D  E  F  G  H

Turno nero!

   A  B  C  D  E  F  G  H
1 [ ][n][ ][n][ ][n][ ][n] 1
2 [n][ ][n][ ][n][ ][n][ ] 2
3 [ ][ ][ ][ ][ ][n][ ][ ] 3
4 [n][ ][ ][ ][ ][ ][n][ ] 4
5 [ ][n][ ][ ][ ][ ][ ][ ] 5
6 [b][ ][b][ ][b][ ][b][ ] 6
7 [ ][b][ ][ ][ ][b][ ][b] 7
8 [b][ ][b][ ][b][ ][b][ ] 8
   A  B  C  D  E  F  G  H
```

#### Multiple capture

Stream of `PlayerA`:

```text
   A  B  C  D  E  F  G  H
1 [ ][n][ ][n][ ][n][ ][n] 1
2 [n][ ][n][ ][n][ ][n][ ] 2
3 [ ][ ][ ][n][ ][n][ ][ ] 3
4 [n][ ][b][ ][ ][ ][n][ ] 4
5 [ ][ ][ ][ ][ ][ ][ ][ ] 5
6 [b][ ][b][ ][b][ ][b][ ] 6
7 [ ][b][ ][ ][ ][b][ ][b] 7
8 [b][ ][b][ ][b][ ][b][ ] 8
   A  B  C  D  E  F  G  H

Turno nero!

   A  B  C  D  E  F  G  H
1 [ ][n][ ][n][ ][n][ ][n] 1
2 [n][ ][n][ ][n][ ][n][ ] 2
3 [ ][ ][ ][ ][ ][n][ ][ ] 3
4 [n][ ][ ][ ][ ][ ][n][ ] 4
5 [ ][ ][ ][ ][ ][ ][ ][ ] 5
6 [b][ ][ ][ ][b][ ][b][ ] 6
7 [ ][b][ ][n][ ][b][ ][b] 7
8 [b][ ][b][ ][b][ ][b][ ] 8
   A  B  C  D  E  F  G  H
```

Stream of `PlayerB`:

```text
   A  B  C  D  E  F  G  H
1 [ ][n][ ][n][ ][n][ ][n] 1
2 [n][ ][n][ ][n][ ][n][ ] 2
3 [ ][ ][ ][n][ ][n][ ][ ] 3
4 [n][ ][b][ ][ ][ ][n][ ] 4
5 [ ][ ][ ][ ][ ][ ][ ][ ] 5
6 [b][ ][b][ ][b][ ][b][ ] 6
7 [ ][b][ ][ ][ ][b][ ][b] 7
8 [b][ ][b][ ][b][ ][b][ ] 8
   A  B  C  D  E  F  G  H

Turno nero!
E' il tuo turno.

Inserisci la pedina che vuoi muovere e poi le mosse che vuoi fare
Es > 6A 5B oppure 6A 4C 2A oppure 6A 4C 2A ...
3d 5b 7d

   A  B  C  D  E  F  G  H
1 [ ][n][ ][n][ ][n][ ][n] 1
2 [n][ ][n][ ][n][ ][n][ ] 2
3 [ ][ ][ ][ ][ ][n][ ][ ] 3
4 [n][ ][ ][ ][ ][ ][n][ ] 4
5 [ ][ ][ ][ ][ ][ ][ ][ ] 5
6 [b][ ][ ][ ][b][ ][b][ ] 6
7 [ ][b][ ][n][ ][b][ ][b] 7
8 [b][ ][b][ ][b][ ][b][ ] 8
   A  B  C  D  E  F  G  H
```

Stream of spectators:

```text
   A  B  C  D  E  F  G  H
1 [ ][n][ ][n][ ][n][ ][n] 1
2 [n][ ][n][ ][n][ ][n][ ] 2
3 [ ][ ][ ][n][ ][n][ ][ ] 3
4 [n][ ][b][ ][ ][ ][n][ ] 4
5 [ ][ ][ ][ ][ ][ ][ ][ ] 5
6 [b][ ][b][ ][b][ ][b][ ] 6
7 [ ][b][ ][ ][ ][b][ ][b] 7
8 [b][ ][b][ ][b][ ][b][ ] 8
   A  B  C  D  E  F  G  H

Turno nero!

   A  B  C  D  E  F  G  H
1 [ ][n][ ][n][ ][n][ ][n] 1
2 [n][ ][n][ ][n][ ][n][ ] 2
3 [ ][ ][ ][ ][ ][n][ ][ ] 3
4 [n][ ][ ][ ][ ][ ][n][ ] 4
5 [ ][ ][ ][ ][ ][ ][ ][ ] 5
6 [b][ ][ ][ ][b][ ][b][ ] 6
7 [ ][b][ ][n][ ][b][ ][b] 7
8 [b][ ][b][ ][b][ ][b][ ] 8
   A  B  C  D  E  F  G  H
```

## Additional information

- the game can only be played by exactly `2` players
- no more than `1` server bot per game is allowed.
