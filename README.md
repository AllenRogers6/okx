# okx

A zoxide like tool to open files from the terminal without having to cd to the dir.

## Install

```bash
git clone https://github.com/AllenRogers6/okx.git
cd okx
cargo install --path .
```

## Uninstall

```bash
cargo uninstall
```

## Usage

```bash
ok file  # opens the file
```

```bash
Usage: okx <COMMAND>

Commands:
  add          Add a file path to the database
  query        Query the best matching file (prints path)
  open         Open the best matching file
  remove       Remove an exact path from the database
  clean        Remove all tracked paths that no longer exist on disk
  list         List all tracked files ordered by frecency score
  shell-init   Print the shell integration function to eval in your shell config
  completions  Print shell tab-completion script
  help         Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```
