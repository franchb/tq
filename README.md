# tq
Command-line TOML processor

Inspired by [tombl](https://github.com/snyball/tombl), but returns a shell-escaped output. Could be useful for various DevOps automation tasks.


```shell
USAGE:
    tq [OPTIONS] [PATTERN]

ARGS:
    <PATTERN>

OPTIONS:
    -e, --eval <EVAL>         Evaluate pattern
    -f, --file <FILEPATH>     Reads TOML from the named file.
    -h, --help                Print help information
    -V, --version             Print version information

EXAMPLES:
    $ cat Cargo.toml
    [profile.target]
    lto = true
    debug = 1

    $ tq -f Cargo.toml profile.target.lto
    true

    $ cat Cargo.toml | tq profile.target.lto
    true

    $ cat Cargo.toml | tq profile.target.debug
    1
```
