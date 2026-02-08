# Runtime dialogue

## Environment variables

Topiary needs to find [language query files](../getting-started/on-tree-sitter.md)
(`*.scm`) to function properly. By default, Topiary looks for these
under the following search paths, from highest to lowest priority:

<!-- This probably should change: see Issue #1003 -->
1. Per the `TOPIARY_LANGUAGE_DIR` environment variable, as set at
   runtime.
2. A built-in value, which was set by the `TOPIARY_LANGUAGE_DIR`
   environment variable at build time.
3. `topiary-queries/queries` in the current working directory.
4. `topiary-queries/queries` in the parent of the current directory.

That is to say, if you are running Topiary from a directory other than
its repository, **you must set the environment variable
`TOPIARY_LANGUAGE_DIR` to point to the directory where Topiary's
language query files are located**.

By default, you should set it to `<local path of the Topiary
repository>/topiary-queries/queries`, for example:

```sh
export TOPIARY_LANGUAGE_DIR=/home/me/tools/topiary/topiary-queries/queries
topiary format ./projects/helloworld/hello.ml
```

`TOPIARY_LANGUAGE_DIR` can alternatively be set at build time. Topiary
will pick the correspond path up and embed it into the `topiary` binary.
In that case, you don't have to worry about making
`TOPIARY_LANGUAGE_DIR` available at runtime any more. When
`TOPIARY_LANGUAGE_DIR` has been set at build time and is set at runtime
as well, the runtime value takes precedence.

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

Negative results with error code `1` only happen when Topiary is called
with the `coverage` sub-command, if the input does not cover 100% of the
query.

When given multiple inputs, Topiary will do its best to process them
all, even in the presence of errors. Should _any_ errors occur, Topiary
will return a non-zero exit code. For more details on the nature of
these errors, run Topiary at the `warn` logging level (with `-v`).
