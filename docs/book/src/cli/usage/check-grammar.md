# Grammar checking

Check if an input parses to the respective Tree-sitter grammar

<!-- DO NOT REMOVE THE "usage:{start,end}" COMMENTS -->
<!-- usage:start -->
```
Check if an input parses to the respective Tree-sitter grammar

Usage: topiary check-grammar [OPTIONS] <--language <LANGUAGE>|FILES>

Arguments:
  [FILES]...
          Input files and directories (omit to read from stdin)

          Language detection and query selection is automatic, mapped from file extensions
          defined in the Topiary configuration.

Options:
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
