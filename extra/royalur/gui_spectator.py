#!/usr/bin/env python3

import cairo
import gi
gi.require_foreign('cairo')
gi.require_version("Gtk", "3.0")
from gi.repository import Gtk, Gdk, Gio
from board import Board
from os import environ
from sys import exit
from math import pi, sqrt

class MainWindow(Gtk.Window):
    def __init__(self, board, stream):
        Gtk.Window.__init__(self, title="Royal Game of Ur")
        self.board = board
        self.turn = 0
        self.roll = None
        self.turn_end = True
        self.retired = None
        self.round = 0
        self.again = True
        self.ended = False
        self.show_anyway = False
        self.set_default_size(1280, 720)
        self.connect("destroy", Gtk.main_quit)
        drawing_area = Gtk.DrawingArea()
        self.add(drawing_area)
        drawing_area.connect('draw', self.draw)
        stream.read_line_async(0, None, self.read, None)

    def draw(self, da, ctx):
        width = da.get_allocated_width()
        height = da.get_allocated_height()
        fc = self.get_style_context().get_color(Gtk.StateFlags.NORMAL)
        pc = [(1.0, 0.25, 0.0, 1.0), (0.0, 1.0, 0.5, 1.0)]
        ratio = width / height
        ctx.select_font_face("sans-serif")
        FONT_HEIGHT = height / 25
        ctx.set_font_size(FONT_HEIGHT)
        # Draw board
        ctx.set_source_rgba(*fc)
        ctx.set_line_width(width / 500)
        BX, BW = 0.2 * width, 0.6 * width
        CS = BW / 8
        BH = CS * 3
        BY = 0.5 * height - BH / 2
        for (r, c) in [(0, 0), (2, 0), (1, 3), (0, 6), (2, 6)]:
            ctx.arc(BX + (c + 0.5) * CS, BY + (r + 0.5) * CS, CS * 0.125, 0, 2 * pi)
            ctx.stroke()
            ctx.arc(BX + (c + 0.5) * CS, BY + (r + 0.5) * CS, CS * 0.25, 0, 2 * pi)
            ctx.stroke()
            ctx.arc(BX + (c + 0.5) * CS, BY + (r + 0.5) * CS, CS * 0.25 * sqrt(2) * 1.01, 0, 2 * pi)
            ctx.stroke()
            ctx.rectangle(BX + (c + 0.25) * CS, BY + (r + 0.25) * CS, CS / 2, CS / 2)
            ctx.stroke()
            ctx.move_to(BX + (c + 0.25) * CS, BY + (r + 0.25) * CS)
            ctx.line_to(BX + (c + 0.75) * CS, BY + (r + 0.75) * CS)
            ctx.stroke()
            ctx.move_to(BX + (c + 0.75) * CS, BY + (r + 0.25) * CS)
            ctx.line_to(BX + (c + 0.25) * CS, BY + (r + 0.75) * CS)
            ctx.stroke()
        for r in range(3):
            for c in range(8):
                if r in (0, 2) and c in (4, 5):
                    continue
                ctx.set_source_rgba(*fc)
                ctx.rectangle(BX + c * CS, BY + r * CS, CS, CS)
                ctx.stroke()
                if self.board.board[r][c] is not None:
                    ctx.arc(BX + (c + 0.5) * CS, BY + (r + 0.5) * CS, CS * 0.25, 0, 2 * pi)
                    ctx.stroke()
                    ctx.set_source_rgba(*pc[self.board.board[r][c][0]])
                    ctx.arc(BX + (c + 0.5) * CS, BY + (r + 0.5) * CS, CS * 0.25, 0, 2 * pi)
                    ctx.fill()
        # Draw info
        def print_centered(x, y, text):
            (xs, ys, w, h, dx, dy) = ctx.text_extents(text)
            ctx.move_to(x - w / 2, y + h / 2)
            ctx.show_text(text)
        ctx.set_source_rgba(*pc[self.turn])
        ctx.move_to(0.05 * width, 0.1 * height)
        ctx.show_text(f"Turn #{self.round}")
        ctx.set_source_rgba(*fc)
        if self.ended:
            ctx.move_to(0.05 * width, 0.9 * height + FONT_HEIGHT / 2)
            if self.retired is not None:
                winner = 1 - self.retired
            else:
                winner = self.board.winner()
            print_centered(0.5 * width, 0.1 * height, f"{self.board.name[winner]} wins")
        for i in range(2):
            ctx.set_source_rgba(*pc[i])
            print_centered(0.5 * width + CS, 0.5 * height + CS * 2 * (i * 2 - 1), f"{self.board.name[i]}")
            print_centered(0.5 * width + CS * 0.5, 0.5 * height + CS * (i * 2 - 1), f"{len(self.board.at_start(i))}")
            print_centered(0.5 * width + CS * 1.5, 0.5 * height + CS * (i * 2 - 1), f"{self.board.score(i)}")
        ctx.set_source_rgba(*fc)
        if self.retired is not None:
            ctx.set_source_rgba(*fc)
            print_centered(0.5 * width, 0.9 * height, f"{self.board.name[self.retired]} retires")
        elif not self.turn_end or self.show_anyway:
            if self.roll is not None:
                print_centered(0.5 * width, 0.9 * height, f"{self.board.name[self.turn]} rolled {sum(self.roll)}{' (cannot move)' if self.show_anyway else ''}")

    def read(self, stream, data, *args):
        data = stream.read_line_finish_utf8(data)[0].strip()
        if self.turn_end:
            self.show_anyway = False
            self.round += 1
            self.turn_end = False
            if not self.again:
                self.turn = 1 - self.turn
            self.again = False
            self.roll = [int(x) for x in data.split(" ")]
            roll = sum(self.roll)
            if not self.board.valid_moves(self.turn, roll):
                self.turn_end = True
                self.show_anyway = True
        else:
            if data == "RETIRE":
                self.retired = self.turn
                self.ended = True
            else:
                if self.board.make_move(self.turn, int(data), sum(self.roll)):
                    self.again = True
                self.turn_end = True
                if self.board.winner() is not None:
                    self.ended = True
        if not self.ended:
            stream.read_line_async(0, None, self.read, None)
        self.queue_draw()

if __name__ == "__main__":
    stream = Gio.DataInputStream.new(Gio.File.new_for_path(environ["COCO_PIPEIN"]).read())
    p = [stream.read_line_utf8()[0].strip() for i in range(2)]
    board = Board(*p)
    window = MainWindow(board, stream)
    window.show_all()
    Gtk.main()
