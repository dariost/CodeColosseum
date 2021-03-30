# new

This subcommand can be used to create a new match. The bare minimum needed to
create a match is to provide the game to be played. For instance, creating a match
of `roshambo` can be done using the following command:

```shell
$ coco -s wss://code.colosseum.cf/ new roshambo
```

If the creation is succesul the unique ID of the match will be printed, otherwise
an error message will be shown.

The creation of a new match can be further customised.

To add a custom name for the match you can add a positional argument at the end:
```shell
$ coco -s wss://code.colosseum.cf/ new roshambo "Test Match"
```

To specify the number of players for the match, the `-n` switch can be used:
```shell
$ coco -s wss://code.colosseum.cf/ new roshambo -n 2
```

To specify the number of server provided bots for the match, the `-b` switch can be used:
```shell
$ coco -s wss://code.colosseum.cf/ new roshambo -b 1
```

To specify the timeout for player inactivity, the `-t` switch can be used:
```shell
$ coco -s wss://code.colosseum.cf/ new roshambo -t 5
```

To specify a password to join the game, the `-p` switch can be used:
```shell
$ coco -s wss://code.colosseum.cf/ new roshambo -p "securepassword"
```

To specify further arguments specific to the game, the `-a` switch can be used
(even multiple times), following the format `-a key=val`:
```shell
$ coco -s wss://code.colosseum.cf/ new roshambo -a rounds=100
```

To create a verified game, if in posses of the server master password, the `-v` switch can be used:
```shell
$ coco -s wss://code.colosseum.cf/ new roshambo -v "servermasterpassword"
```
