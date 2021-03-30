# Command Line Client

**Code Colosseum** provdes `coco`, a command line client that provides an user
interface to interact with a **Code Colosseum** server.

## Prebuilt binaries

Prebuilt binaries for `coco` can be found in the [_Releases_](https://github.com/dariost/CodeColosseum/releases/latest) section of the
[GitHub page](https://github.com/dariost/CodeColosseum/).
It is advised to put the downloaded and extracted executable in a directory
included in the system `PATH`.

If a prebuilt binary for the desired operating system and architecture is not
found, then the only option is to build `coco` from the source code.

## Building from source

The command line client is written in the [Rust](https://www.rust-lang.org/)
programming language. Thus, to build it the first step is to
[install Rust](https://www.rust-lang.org/tools/install).

Once the Rust compiler has been installed, the source code can be downloaded
[here](https://github.com/dariost/CodeColosseum/releases/latest). Alternatively,
it can be obtained by cloning the GitHub repository by using the following command:

```shell
$ git clone https://github.com/dariost/CodeColosseum.git
```

Then, the client can be built using the following command:

```shell
$ cargo build --release --bin coco
```

The resulting compiled binary file will be found in the `target/release/`
directory, and will be named `coco.exe` on Windows and `coco` on other operating
systems. It is advised to put such program in a directory included in the system
`PATH`.


## Basic usage

`coco` provides several subcommands that will be discussed in the following
subsections. It is advised to read such subsections as well as using the `--help`
feature of `coco` to explore its options:

```shell
$ coco --help
```

The only common option to all of `coco` subcommands is the server directive, used
to specify the server to which `coco` will connect and passed using the `-s` prefix.

The official server of **Code Colosseum** is located at `wss://code.colosseum.cf/`,
thus all examples will feature such server. As a first example, the invocation of
`coco` to list the available games on the official server would be:

```shell
$ coco -s wss://code.colosseum.cf/ list
```
