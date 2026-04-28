# Format

<!-- DO NOT REMOVE THE "usage:{start,end}" COMMENTS -->
<!-- usage:start -->
```
Format inputs

Usage: topiary format [OPTIONS] <--language <LANGUAGE>|FILES>

Arguments:
  [FILES]...
          Input files and directories (omit to read from stdin)

          Language detection and query selection is automatic, mapped from file extensions
          defined in the Topiary configuration.

Options:
  -c, --check
          Verify inputs are already formatted (exit non-zero with diff if not)

  -t, --tolerate-parsing-errors
          Consume as much as possible in the presence of parsing errors

  -s, --skip-idempotence
          Do not check that formatting twice gives the same output

  -l, --language <LANGUAGE>
          Topiary language identifier (when formatting stdin)

  -q, --query <QUERY>
          Topiary query file override (when formatting stdin)

  -L, --follow-symlinks
          Follow symlinks (when formatting files)

  -C, --configuration <CONFIGURATION>
          Configuration file

          [env: TOPIARY_CONFIG_FILE]

  -M, --merge-configuration
          Enable merging for configuration files

  -v, --verbose...
          Logging verbosity (increased per occurrence)

  -h, --help
          Print help (see a summary with '-h')
```
<!-- usage:end -->

> **Note**\
> `fmt` is a recognised alias of the `format` subcommand.

When formatting inputs from disk, language selection is detected from
the input files' extensions. To format standard input, you must specify
the `--language` and, optionally, `--query` arguments, omitting any
input files.

Valid language identifiers, as specified with `--language`, are defined
as part of your Topiary configuration. See the [configuration](../configuration.md)
chapter for more details.

<div class="warning">

Topiary will not accept a process substitution (or any other named pipe)
as formatting input. Instead, arrange for a redirection into Topiary's
standard input:

```sh
# This won't work
topiary format <(some_command)

# Do this instead
some_command | topiary format --language LANGUAGE
```

</div>

## Checking formatting

The `--check` flag (or `-c`) verifies that inputs are already formatted
without modifying them. If any input differs from its formatted form,
Topiary prints a diff to stderr and exits with code 1. This is useful
for CI pipelines:

```bash
# Check a single file
topiary format --check src/main.rs

# Check all files in a directory
topiary format --check src/

# Check via stdin
echo '{"foo":"bar"}' | topiary format --check --language json
```

<div class="warning">

Topiary will skip over some input files under certain conditions,
which are logged at varying levels:

| Condition                                     | Level   |
| :-------------------------------------------- | :------ |
| Cannot access file                            | Error   |
| Not a regular file (e.g., FIFO, socket, etc.) | Warning |
| A symlink without `--follow-symlinks`         | Warning |
| File with multiple (hard) links               | Error   |
| File does not exist (e.g., broken symlink)    | Error   |

</div>
