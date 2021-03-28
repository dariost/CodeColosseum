#!/usr/bin/env python3

from os import environ
from sys import stdin, exit
from io import StringIO
from board import Board

if __name__ == "__main__":
    pipe = environ.get("COCO_PIPEIN")
    fin = stdin if pipe is None else open(pipe, "r")
    p = [fin.readline().strip() for _ in range(2)]
    print("Royal Game of Ur")
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
