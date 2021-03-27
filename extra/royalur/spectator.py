#!/usr/bin/env python3

from os import environ
from sys import stdin, exit
from io import StringIO

BOARD = """                {} (X)
+---+---+---+---+       +---+---+
|#Z#| Z | Z | Z | Z   Z |#Z#| Z |
+---+---+---+---+---+---+---+---+
| Z | Z | Z |#Z#| Z | Z | Z | Z |
+---+---+---+---+---+---+---+---+
|#Z#| Z | Z | Z | Z   Z |#Z#| Z |
+---+---+---+---+       +---+---+
                {} (O)""".replace("Z", "{}")

class Board:
    def __init__(self, p0, p1):
        self.name = [p0, p1]
        self.board = [[None for j in range(8)] for i in range(3)]
        self.START = object()
        self.END = object()
        self.position = [[(self.START, i) for j in range(7)] for i in range(2)]

    def winner(self):
        for i in range(2):
            if self.score(i) == 7:
                return i
        return None

    def _advance_once(self, pos, p):
        if pos is None:
            return None
        elif pos[0] == self.END:
            return None
        elif pos[0] == self.START:
            return (p * 2, 3)
        elif pos == (0, 0) or pos == (2, 0):
            return (1, 0)
        elif pos == (0, 6) or pos == (2, 6):
            return (self.END, p)
        elif pos == (1, 7):
            return (p * 2, 7)
        elif pos[0] == 1:
            return (1, pos[1] + 1)
        else:
            return (pos[0], pos[1] - 1)

    def _advance(self, pos, cells, player):
        for i in range(cells):
            pos = self._advance_once(pos, player)
        return pos

    def _simulate_move(self, player, token, cells):
        pos = self.position[player][token]
        pos = self._advance(pos, cells, player)
        if pos is None:
            return None
        elif pos[0] == self.START:
            return None
        elif pos[0] == self.END and cells == 0:
            return None
        elif pos[0] == self.END:
            return pos
        elif self.board[pos[0]][pos[1]] is None:
            return pos
        elif self.board[pos[0]][pos[1]][0] == player:
            return None
        elif pos == (1, 3):
            return None
        else:
            return pos

    def valid_moves(self, player, cells):
        return [i for i in range(7) if self._simulate_move(player, i, cells) is not None]

    def make_move(self, player, token, cells):
        tpos = self._simulate_move(player, token, cells)
        cpos = self.position[player][token]
        assert tpos is not None
        if tpos[0] == self.END:
            assert tpos[1] == player
            self.board[cpos[0]][cpos[1]] = None
            self.position[player][token] = tpos
            return False
        if cpos[0] == self.START:
            assert cpos[1] == player
            assert self.board[tpos[0]][tpos[1]] is None
            self.board[tpos[0]][tpos[1]] = (player, token)
            self.position[player][token] = tpos
            return tpos[1] == 0
        if self.board[tpos[0]][tpos[1]] is not None:
            (p, t) = self.board[tpos[0]][tpos[1]]
            self.position[p][t] = (self.START, p)
        self.board[cpos[0]][cpos[1]] = None
        self.board[tpos[0]][tpos[1]] = (player, token)
        self.position[player][token] = tpos
        return tpos in ((0, 0), (2, 0), (1, 3), (0, 6), (2, 6))

    def score(self, player):
        return len(list(filter(lambda x: x[0] == self.END, self.position[player])))

    def at_start(self, player):
        return len(list(filter(lambda x: x[0] == self.START, self.position[player])))

    def __repr__(self):
        v = [self.name[0]]
        for r in range(3):
            for c in range(8):
                if r == 0 and c == 4:
                    v.append(self.at_start(0))
                elif r == 0 and c == 5:
                    v.append(self.score(0))
                elif r == 2 and c == 4:
                    v.append(self.at_start(1))
                elif r == 2 and c == 5:
                    v.append(self.score(1))
                elif self.board[r][c] is None:
                    v.append(" ")
                elif self.board[r][c][0] == 0:
                    v.append("X")
                else:
                    v.append("O")
        v += [self.name[1]]
        return BOARD.format(*v)

if __name__ == "__main__":
    pipe = environ.get("COCO_PIPEIN")
    fin = stdin if pipe is None else open(pipe, "r")
    p = [fin.readline().strip() for _ in range(2)]
    print("Game of Royal Ur")
    print()
    print(f"{p[0]} vs {p[1]}")
    print()
    board = Board(*p)
    turn = 0
    while board.winner() is None:
        roll = sum(map(int, fin.readline().strip().split(" ")))
        print(f"{p[turn]} rolled a {roll}")
        if not board.valid_moves(turn, roll):
            print(f"{p[turn]} has no valid moves available")
            print()
            turn = 1 - turn
            continue
        move = fin.readline().strip()
        if move == "RETIRE":
            print()
            print(f"{p[turn]} retires")
            print(f"{p[1 - turn]} wins")
            print()
            exit(0)
        if board.make_move(turn, int(move), roll):
            turn = 1 - turn
        print()
        print(board)
        print()
        turn = 1 - turn
    for i in range(2):
        print(f"{p[i]} finished with score: {board.score(i)}")
    print()
    print(f"{p[board.winner()]} wins")
    print()
