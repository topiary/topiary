# Runtime dialogue

## Environment variables

Topiary needs to find [language query files](../getting-started/on-tree-sitter.md)
(`*.scm`) to function properly. By default, Topiary includes standard queries bundled directly into the binary at compile time.

However, if you are developing custom queries or need to override the default queries, Topiary looks for query directories in the following priority order:

1. The `--query-dir` CLI argument, if provided.
2. An adjacent `queries/` folder in the same directory as the loaded configuration file (e.g., `~/.config/topiary/queries/`).
3. `topiary-queries/queries` in the current working directory.
4. `topiary-queries/queries` in the parent of the current directory.
5. The compiled-in default queries.

That is to say, you can easily use your own queries by either placing them in a `queries/` folder next to your `languages.ncl` configuration file, or by explicitly passing `--query-dir` to the CLI:

```sh
topiary format --query-dir /home/me/tools/topiary/topiary-queries/queries ./projects/helloworld/hello.ml
```

See [the contributor's guide](../guides/contributing.md) for details on
setting up a development environment.

## Logging

By default, the Topiary CLI will only output error messages. You can
increase the logging verbosity with a respective number of
`-v`/`--verbose` flags:

| Verbosity Flag | Logging Level           |
| :------------- | :---------------------- |
| None           | Errors                  |
| `-v`           | ...and warnings         |
| `-vv`          | ...and information      |
| `-vvv`         | ...and debugging output |
| `-vvvv`        | ...and tracing output   |

## Exit codes

The Topiary process will exit with a zero exit code upon successful
formatting. Otherwise, the following exit codes are defined:

| Reason                       | Code |
| :--------------------------- | ---: |
| Negative result              |    1 |
| CLI argument parsing error   |    2 |
| I/O error                    |    3 |
| Topiary query error          |    4 |
| Source parsing error         |    5 |
| Language detection error     |    6 |
| Idempotence error            |    7 |
| Unspecified formatting error |    8 |
| Multiple errors              |    9 |
| Unspecified error            |   10 |

Negative results with error code `1` happen when Topiary is called
with the `coverage` sub-command (if the input does not cover 100% of the
query), or with `format --check` (if the input is not already
formatted).

When given multiple inputs, Topiary will do its best to process them
all, even in the presence of errors. Should _any_ errors occur, Topiary
will return a non-zero exit code. For more details on the nature of
these errors, run Topiary at the `warn` logging level (with `-v`).
