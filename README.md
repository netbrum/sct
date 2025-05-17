Dead simple ssh_config TUI. Not made for advanced usage!

### Usage

By default, `shtui` scans for the config file in `$HOME/.ssh/config`, you can however use a different config file, by setting the `--config <file>` option.

It supports alacritty (default) and konsole to open a new terminal with the SSH session, there isn't a specific reason for this, it's just the terminals I happen to use (I was too lazy to implement other terminals).
