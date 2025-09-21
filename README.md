# rnv

```
❯ rnv --help
Generate renovatable line for git repository

Usage: rnv [OPTIONS] <DIR> <COMMAND>

Commands:
  gen    Generate renovatable line.
  lock   Get the commit hash from the renovatable line.
  batch  Batch operations.
  help   Print this message or the help of the given subcommand(s)

Arguments:
  <DIR>  Path to the target repository

Options:
      --git <GIT>  Git command [default: git]
  -h, --help       Print help
  -V, --version    Print version
```

## Usage

Make the repository renovatable:

``` shell
rnv path/to/repo gen > renovate.lock
```

Add the custom manager to renovate.json:

``` json
{
  "customType": "regex",
  "fileMatch": ["renovate.lock"],
  "matchStrings": ["depName=(?<depName>.+) datasource=(?<datasource>[a-z-]+) value=(?<currentValue>.+)"]
}
```

After renovated, get the commit hash:

``` shell
rnv path/to/repo lock < renovate.lock
```
