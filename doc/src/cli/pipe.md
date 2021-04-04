# pipe channel

The `pipe` communication channel uses [named pipes](https://en.wikipedia.org/wiki/Named_pipe)
as the communication mean to and from the program. Since the implementation of
named pipes is wildly different between Unix-like operating systems and Windows,
they shall be discussed separately.

## Unix-like (Linux, macOS, FreeBSD, etc.)
On Unix-like systems named pipes can be read and written as ordinary files. Thus,
their use is no different than reading and writing using the common libraries that
programming languages usually provide.

If a program is started by `coco` with the `pipe` channel, `coco` will pass to the
program two environment variables: `COCO_PIPEIN` and `COCO_PIPEOUT`. These
two variables contain the names of the files to be used for input and output,
respectively.

The program shall read the file names from these environment variables and open
the respective files in read and write mode as the first operation of the program.

**`coco` will not consider the program as started until both files are opened.**

Since the program sees these pipes as files, writing operations to the output pipe
will be buffered even in the presence of a `LF`, since the rules for buffering are
different between files and `stdout`.
**Thus, the output stream must be flushed manually when written.**

In spectator mode only `COCO_PIPEIN` is provided to the program.

A simple usage example in python of the `pipe` communication channel is provided:
```python
#!/usr/bin/env python3

from os import environ

if __name__ == "__main__":
    fin = open(environ["COCO_PIPEIN"], "r")
    fout = open(environ["COCO_PIPEOUT"], "w")
    from_server = fin.readline().strip()
    print("to_server", file=fout, flush=True)
    print("debug print")
```

## Windows
The `pipe` communication channel is currently unavailable on Windows.
