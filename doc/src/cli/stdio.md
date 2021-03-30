# stdio channel

The `stdio` communication channel uses the program `stdout` for output and
`stdin` for input. This is the classically used communication method for
most of the contents management systems.

Thus, to read from the server data stream is to read from `stdin` and to write
to the server data stream is to write to `stdout`. Note that it is a good practice
to flush `stdout` after each write, since some buffering might happend and the
written command would not be sent to the server.

Since the `stdin` and `stdout` streams of the program are managed by `coco` to
communicate with the server, they cannot be used to read or write strings from or
to the terminal. However, the `stderr` stream is free, and can be used to write
strings to the terminal. If more freedom is needed, refer to the `pipe`
communication channel.

In spectator mode only the `stdin` stream is captured by `coco`.

A simple usage example in python of the `stdio` communication channel is provided:
```python
#!/usr/bin/env python3

from sys import stderr

if __name__ == "__main__":
    from_server = input().strip()
    print("to_server", flush=True)
    print("debug print", file=stderr)
```
