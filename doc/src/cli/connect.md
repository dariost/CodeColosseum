# connect

This subcommand can be used to join a waiting-to-be-started match or spectate
any match (either already running or not). The bare minimum needed to join or
spectate a match is the match unique ID and a local program to run which will
be connected to the server data stream. The protocol of such stream is discussed
in the description of each game.

To then join a match with ID `abacaba` using the program `prog.exe` the command would be:

```shell
$ coco -s wss://code.colosseum.cf/ connect "abacaba" -- prog.exe
```

And to spectate it the `-s` switch must be added:
```shell
$ coco -s wss://code.colosseum.cf/ connect -s "abacaba" -- prog.exe
```

Note that it is possible to provide arguments to the local program, for instance:
```shell
$ coco -s wss://code.colosseum.cf/ connect "abacaba" -- prog.exe arg1 arg2
```
This is useful when wanting to connect a program written in an interpreted language
such as python. For instance, to connect the program `prog.py` che command would be:
```shell
$ coco -s wss://code.colosseum.cf/ connect "abacaba" -- python prog.py
```

Note that it is possible to omit the local program, however this is discouraged,
as it will cause a `cat`-like program to be called instead.

If a match is password protected, the password can be provided with the `-p` switch:
```shell
$ coco -s wss://code.colosseum.cf/ connect -p "securepassword" "abacaba" -- prog.exe
```

When joining a match a custom username can be choosed by using the `-n` switch:
```shell
$ coco -s wss://code.colosseum.cf/ connect -n "verycoolname" "abacaba" -- prog.exe
```

Note that both the username and password options are ignored when spectating.

The communication with the local program is performed through a channel. There are
two channels types available, `stdio` and `pipe`. The default is `stdio`. The channel
can be choosed using the `-c` switch:
```shell
$ coco -s wss://code.colosseum.cf/ connect -c "pipe" "abacaba" -- prog.exe
```
The details of such channels are discussed in the further subsubsections.
