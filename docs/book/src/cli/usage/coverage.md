# Query coverage

This subcommand checks how much of the language query file is used to
process the input. Specifically, it checks the percentage of queries in
the query file that match the given input, and prints the queries that
don't match anything.

<!-- DO NOT REMOVE THE "usage:{start,end}" COMMENTS -->
<!-- usage:start -->
```
Checks how much of the tree-sitter query is used

Usage: topiary coverage [OPTIONS] <--language <LANGUAGE>|FILE>

Arguments:
  [FILE]
          Input file (omit to read from stdin)

          Language detection and query selection is automatic, mapped from file extensions
          defined in the Topiary configuration.

Options:
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

The `coverage` subcommand will exit with error code `1` if the coverage
is less than 100%.
