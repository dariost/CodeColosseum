# list

This subcommand can be used to list the available games on a server:

```shell
$ coco -s wss://code.colosseum.cf/ list
```

This will print a list of available games.

This subcommand can also be used to retrieve the description for a game. To do so
the game bame must be specified after `list`. For instance, to get the description
for the game `roshambo` the following command can be used:

```shell
$ coco -s wss://code.colosseum.cf/ list roshambo
```

Note that all descriptions should be written in MarkDown, thus, if `pandoc` is
available, a PDF can be generated from the description of a game using the following
command:

```shell
$ coco -s wss://code.colosseum.cf/ list roshambo | pandoc -o roshambo.pdf
```
