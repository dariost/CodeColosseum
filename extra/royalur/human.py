#!/usr/bin/env python3

from board import Board
from os import environ
from sys import exit

if __name__ == "__main__":
    fin = open(environ["COCO_PIPEIN"], "r")
    fout = open(environ["COCO_PIPEOUT"], "w")
    p = [fin.readline().strip() for _ in range(2)]
    me = int(fin.readline().strip())
    print("Royal Game of Ur")
    print()
    print(f"{p[0]} vs {p[1]}")
    print()
    board = Board(*p)
    turn = 0
    turn_number = 0
    print(board.pretty(me))
    while board.winner() is None:
        turn_number += 1
        print()
        print(f"Turn #{turn_number}")
        print()
        roll = sum(map(int, fin.readline().strip().split(" ")))
        print(f"{p[turn]} rolled a {roll}")
        if not board.valid_moves(turn, roll):
            print(f"{p[turn]} has no valid moves available")
            print()
            turn = 1 - turn
            continue
        if turn == me:
            vm = board.valid_moves(me, roll)
            tb = board.at_start(me)
            assert len(vm) > 0
            if len(tb) > 0:
                print("Tokens at start: {}".format(" ".join(map(str, tb))))
            print("Movable tokens: {}".format(" ".join(map(str, vm))))
            m = -1
            while m not in vm:
                try:
                    m = int(input("Enter token to move: "))
                except:
                    continue
            print(m, file=fout, flush=True)
            if board.make_move(me, m, roll):
                turn = 1 - turn
            print()
            print(board.pretty(me))
        else:
            print(f"Waiting for {p[turn]}'s move")
            m = fin.readline().strip()
            if m == "RETIRE":
                print(f"{p[turn]} retired")
                exit(0)
            if board.make_move(1 - me, int(m), roll):
                turn = 1 - turn
            print()
            print(board.pretty(me))
        turn = 1 - turn
    print()
    for i in range(2):
        print(f"{p[i]} finished with score: {board.score(i)}")
    print()
    print(f"{p[board.winner()]} wins")
    print()
