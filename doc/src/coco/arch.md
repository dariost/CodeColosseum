# Architecture

**Code Colosseum** has a peculiar architecture when compared with software with
similar purposes (e.g. contest management systems): _all user programs run on the_
_user's machine_. This is not _that_ unusual, in fact some competitions, such as
the [Facebook Hacker Cup](https://www.facebook.com/codingcompetitions/hacker-cup),
already evaluate all contestants code on the contestant's machine. However, these
competitions all share a common trait: there is no interaction between the user's
code and the remote evaluation program but the initial input download and the final
output upload. Such systems are unable to deal with interactive problems.

**Code Colosseum** provides a system that has both the ability to deal with the
interaction between the user's code and the remote evaluator and executes the user's
code on the user's machine. To achieve that, it creates a virtual connection between
the user's program and the remote evaluator, such that, in its simplest form, the
user's program _standard output_ is redirected into the _standard input_ of the
remote evaluator, and _vice versa_.

Note that, unlike more traditionals contest management systems, **Code Colosseum**'s
primary purpose is to deal with multiplayer games, as such the concept of the more
traditional evaluator is substituted with the concept of game manager, which is a
program that manages an instance of a game, collects inputs from \\( n \\) clients,
computes the game simulation accordingly, and sends back the updates.
