# xa

*simplified **XA**rgs*

- pipe the output of a command (from stdin) to another command via xa

## Examples

- search all Rust files in the current directory (via ![sf](https://github.com/Phydon/sf))
- pipe the results to the next command via ![xa](https://github.com/Phydon/xa)
- search in all found matches for the word 'todo' (case-insensitive) (via ![mg](https://github.com/Phydon/mg))

```shell
sf `"`" . -e rs -p | xa -rp mg todo '{}' -ip
```

## Usage

### Short Usage

```
Usage: xa [OPTIONS] [COMMAND]... [COMMAND]

Commands:
  log, -L, --log  Show content of the log file
  help            Print this message or the help of the given subcommand(s)

Arguments:
  [COMMAND]...  The command to execute with an argument from stdin

Options:
  -p, --parallel  Process input in parallel if possible
  -r, --replace   Replace the given placeholder [curly braces: '{}'] with the string from stdin
  -h, --help      Print help (see more with '--help')
  -V, --version   Print version
```

### Long Usage

```
Usage: xa [OPTIONS] [COMMAND]... [COMMAND]

Commands:
  log, -L, --log  Show content of the log file
  help            Print this message or the help of the given subcommand(s)

Arguments:
  [COMMAND]...
          The command to execute with an argument from stdin
          Must be the last argument (everything will be treated as a literal string)

Options:
  -p, --parallel
          Process input in parallel if possible
          The input order will most likely change

  -r, --replace
          Replace the given placeholder [curly braces: '{}'] with the string from stdin
          You have to surround the curly braces with single quotes ['{}']

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version    
```

## Installation

### Windows

via Cargo or get the ![binary](https://github.com/Phydon/xa/releases)
