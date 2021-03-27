#!/usr/bin/env python3

from os import environ
from spectator import Board
from random import choice
from sys import stdout, stdin

if __name__ == "__main__":
    pipein = environ.get("COCO_PIPEIN")
    pipeout = environ.get("COCO_PIPEOUT")
    fin = stdin if pipein is None else open(pipein, "r")
    fout = stdout if pipeout is None else open(pipeout, "w")
    p = [fin.readline().strip() for _ in range(2)]
    me = int(fin.readline().strip())
    board = Board(*p)
    turn = 0
    while board.winner() is None:
        roll = sum(map(int, fin.readline().strip().split(" ")))
        if turn == me:
            vm = board.valid_moves(me, roll)
            if len(vm) > 0:
                m = choice(vm)
                print(m, file=fout, flush=True)
                if board.make_move(me, m, roll):
                    turn = 1 - turn
        else:
            if board.valid_moves(1 - me, roll):
                m = fin.readline().strip()
                if m == "RETIRE":
                    break
                if board.make_move(1 - me, int(m), roll):
                    turn = 1 - turn
        turn = 1 - turn
