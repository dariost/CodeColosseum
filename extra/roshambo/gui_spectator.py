#!/usr/bin/env python3

import cairo
import gi
gi.require_foreign('cairo')
gi.require_version("Gtk", "3.0")
from gi.repository import Gtk, Gdk, Gio
from os import environ
from sys import exit
from math import pi, sqrt

class MainWindow(Gtk.Window):
    def __init__(self, name, rounds, stream):
        Gtk.Window.__init__(self, title="Rock Paper Scissors")
        self.names = name
        self.rounds = rounds
        self.round = 1
        self.score = [0, 0]
        self.moves = [None, None]
        self.retired = [False, False]
        R = "ROCK"
        P = "PAPER"
        S = "SCISSORS"
        W = dict([(x, dict()) for x in (R, P, S)])
        W[R][R] = 0
        W[R][P] = 0
        W[R][S] = 1
        W[P][R] = 1
        W[P][P] = 0
        W[P][S] = 0
        W[S][R] = 0
        W[S][P] = 1
        W[S][S] = 0
        self.W = W
        self.set_default_size(1280, 720)
        self.connect("destroy", Gtk.main_quit)
        drawing_area = Gtk.DrawingArea()
        self.add(drawing_area)
        drawing_area.connect('draw', self.draw)
        stream.read_line_async(0, None, self.read, None)

    def draw(self, da, ctx):
        width = da.get_allocated_width()
        height = da.get_allocated_height()
        def print_centered(x, y, text):
            (xs, ys, w, h, dx, dy) = ctx.text_extents(text)
            ctx.move_to(x - w / 2, y + h / 2)
            ctx.show_text(text)
        fc = self.get_style_context().get_color(Gtk.StateFlags.NORMAL)
        pc = [(0.0, 1.0, 0.0, 1.0), (1.0, 0.0, 0.0, 1.0)]
        ratio = width / height
        ctx.select_font_face("sans-serif")
        FONT_HEIGHT = height / 25
        ctx.set_font_size(FONT_HEIGHT)
        # Draw board
        ctx.set_source_rgba(*fc)
        ctx.set_line_width(width / 200)
        ctx.move_to(0.5 * width, 0.25 * height)
        ctx.line_to(0.5 * width, 0.75 * height)
        ctx.stroke()
        print_centered(0.1 * width, 0.1 * height, f"Round {self.round}/{self.rounds}")
        if self.score[0] > self.score[1]:
            c = [pc[0], pc[1]]
        elif self.score[1] > self.score[0]:
            c = [pc[1], pc[0]]
        else:
            c = [fc, fc]
        ctx.set_source_rgba(*c[0])
        print_centered(0.25 * width, 0.2 * height, f"{self.names[0]} | {self.score[0]}")
        ctx.set_source_rgba(*c[1])
        print_centered(0.75 * width, 0.2 * height, f"{self.names[1]} | {self.score[1]}")
        if any(self.retired):
            ctx.set_source_rgba(*fc)
            if self.retired[0]:
                print_centered(0.5 * width, 0.1 * height, f"{self.names[0]} retires")
            if self.retired[1]:
                print_centered(0.5 * width, 0.9 * height, f"{self.names[1]} retires")
        elif self.moves[0] is not None:
            if self.W[self.moves[0]][self.moves[1]]:
                c = [pc[0], pc[1]]
            elif self.W[self.moves[1]][self.moves[0]]:
                c = [pc[1], pc[0]]
            else:
                c = [fc, fc]
            ctx.set_source_rgba(*c[0])
            print_centered(0.25 * width, 0.5 * height, f"{self.moves[0]}")
            ctx.set_source_rgba(*c[1])
            print_centered(0.75 * width, 0.5 * height, f"{self.moves[1]}")
            ctx.set_source_rgba(*fc)
            if self.round == self.rounds:
                if self.score[0] > self.score[1]:
                    print_centered(0.5 * width, 0.9 * height, f"{self.names[0]} wins")
                elif self.score[1] > self.score[0]:
                    print_centered(0.5 * width, 0.9 * height, f"{self.names[1]} wins")
                else:
                    print_centered(0.5 * width, 0.9 * height, f"Tie")

    def read(self, stream, data, *args):
        m = [None, None]
        m[0] = stream.read_line_finish_utf8(data)[0].strip()
        m[1] = stream.read_line_utf8()[0].strip()
        for i in range(2):
            if m[i] == "RETIRED":
                self.retired[i] = True
            else:
                self.moves[i] = m[i]
        if not any(self.retired):
            self.score[0] += self.W[m[0]][m[1]]
            self.score[1] += self.W[m[1]][m[0]]
        self.round += 1
        if self.round < self.rounds and not any(self.retired):
            stream.read_line_async(0, None, self.read, None)
        self.queue_draw()

if __name__ == "__main__":
    stream = Gio.DataInputStream.new(Gio.File.new_for_path(environ["COCO_PIPEIN"]).read())
    name = [stream.read_line_utf8()[0].strip() for i in range(2)]
    rounds = int(stream.read_line_utf8()[0].strip())
    window = MainWindow(name, rounds, stream)
    window.show_all()
    Gtk.main()
