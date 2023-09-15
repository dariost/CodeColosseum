# Chess 

Chess is a two-player, abstract strategy board game that represents medieval warfare on an 8x8 board with alternating light and dark squares. Opposing pieces, traditionally designated White and Black, are initially lined up on either side. Each type of piece has a unique form of movement and capturing occurs when a piece, via its movement, occupies the square of an opposing piece. Players take turns moving one of their pieces in an attempt to capture, attack, defend, or develop their positions. Chess games can end in checkmate, resignation, or one of several types of draws. Chess is one of the most popular games in the world, played by millions of people worldwide at home, in clubs, online, by correspondence, and in tournaments. Between two highly skilled players, chess can be a beautiful thing to watch, and a game can provide great entertainment even for novices. There is also a large literature of books and periodicals about chess, typically featuring games and commentary by chess masters.

The game has its origins in the Indian game Chaturanga, and became Shatranj when introduced to the Persians. The current form of the game emerged in the second half of the 15th century when the Persians brought Shatranj to Southern Europe. The tradition of organized competitive chess began in the 16th century. The first official World Chess Champion, Wilhelm Steinitz, claimed his title in 1886. The current World Champion is Ding Liren, China. Chess is also a recognized sport of the International Olympic Committee.


## Implementation details
At the beginning of the game the board will be generated and both players receive the 16 starting pieces:

- the first line of the board contains 2 Rooks (R), 2 Knights (N), 2 Bishops (B), Queen (Q) and King (K);
- the second line of the board contains all 8 pawns.


The white player starts the turn and, in order to move a piece, he has to type in the console the two coordinates, starting tile and arrival tile, of a specific piece (for example a2 a3).
After this, if the move is correct, the board updates and the turn switches to the other player. 
The game continues until one player checkmates the other or one player proposes a draw or retires.

### Special moves
In chess there are 3 special moves which require some explanation:

- **castling x1 x2 y1 y2**: It consists of moving the king two squares toward a rook on the same rank and then moving the rook to the square that the king passed over. Castling is permitted only if neither the king nor the rook has previously moved.
- **enpassant x1 x2**: It describes the capture by a pawn of an enemy pawn on the same rank and an adjacent file that has just made an initial two-square advance.
- **promotion x1 x2 pieceType**: It is the replacement of a pawn with a new piece when the pawn is moved to its last rank. The player replaces the pawn immediately with a queen, rook, bishop, or knight of the same color.



### Example
This is an example of the streams of two players, `White Player` and `Black Player` for an hypothetical game.


Stream of `Black Player`:
```
> Joined "player1's game" (chess)
> Waiting for game to start
> Game has 0 spectators and 1/2 (0 bots) connected player: ["player2"]
> Game has 0 spectators and 2/2 (0 bots) connected players: ["player2", "player1"]
> Game started
player2
player1
1
a2 a3
a7 a6
a7 a6
e2 e4
a6 a5
a6 a5
d1 h5
a5 a4
a5 a4
f1 c4
a8 a5
a8 a5
h5 f7
CHECKMATE! You loose!
> Game ended
> Press ENTER to exit
```

Stream of `White Player`:
```
> Joined "player1's game" (chess)
> Waiting for game to start
> Game has 0 spectators and 1/2 (0 bots) connected player: ["player2"]
> Game has 0 spectators and 2/2 (0 bots) connected players: ["player2", "player1"]
> Game started
player2
player1
0
a2 a3
a2 a3
a7 a6
e2 e4
e2 e4
a6 a5
d1 h5
d1 h5
a5 a4
f1 c4
f1 c4
a8 a5
a8 a5
Invalid move
h5 f7
h5 f7
CHECKMATE! You win!
> Game ended
> Press ENTER to exit
```


#### Game parameters
There's only one game-specific parameter:

- `pace`: the minimum number of seconds between turns (default: `1.5`, min: `0`, max: `30`).

Additional information:

- the game can only be played by exactly `2` players;
- the default timeout is `90` seconds;
- no more than `1` server bot per game is allowed.
