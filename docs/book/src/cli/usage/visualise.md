# Visualise

`topiary visualise` converts the input's Tree-sitter parse tree to a
graph representation in the selected format. By default, Topiary outputs
a DOT file, which can be rendered using a visualisation tool such as the
Graphviz suite. For example, using Graphviz's `dot`: `topiary visualise
input.ocaml | dot -T png -o output.png`.

<!-- DO NOT REMOVE THE "usage:{start,end}" COMMENTS -->
<!-- usage:start -->
```
Visualise the input's Tree-sitter parse tree

Visualise generates a graph representation of the parse tree that can be rendered by
external visualisation tools, such as Graphviz. By default, the output is in the DOT
format.

Usage: topiary visualise [OPTIONS] <--language <LANGUAGE>|FILE>

Arguments:
  [FILE]
          Input file (omit to read from stdin)

          Language detection and query selection is automatic, mapped from file extensions
          defined in the Topiary configuration.

Options:
  -f, --format <FORMAT>
          Visualisation format

          Possible values:
          - dot:  GraphViz DOT serialisation
          - json: JSON serialisation

          [default: dot]

  -l, --language <LANGUAGE>
          Topiary language identifier (when formatting stdin)

  -q, --query <QUERY>
          Topiary query file override (when formatting stdin)

  -C, --configuration <CONFIGURATION>
          Configuration file

          [env: TOPIARY_CONFIG_FILE]

  -M, --merge-configuration
          Enable merging for configuration files

      --query-dir <QUERY_DIR>
          Override the directory where queries are found

  -v, --verbose...
          Logging verbosity (increased per occurrence)

  -h, --help
          Print help (see a summary with '-h')
```
<!-- usage:end -->

> **Note**\
> `vis`, `visualize` and `view` are recognised aliases of the
> `visualise` subcommand.

When visualising inputs from disk, language selection is detected from
the input file's extension. To visualise standard input, you must
specify the `--language` and, optionally, `--query` arguments, omitting
the input file. The visualisation output is written to standard out.

Valid language identifiers, as specified with `--language`, are defined
as part of your Topiary configuration. See the [configuration](../configuration.md)
chapter for more details.
