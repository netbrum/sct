Dead simple ssh_config TUI written in ~100 lines.

### Install

```sh
cargo install --git https://github.com/netbrum/sct
```

### Usage

By default, `sct` scans for the config file in `$HOME/.ssh/config`, you can however use a different config file, by setting the `--config` option.

Supports any terminal that uses the `-e` option to execute a command (It choses the terminal set in `$TERM` by default).

The following command is what is being executed:

```sh
$TERM -e bash -c 'ssh user@host'
```
