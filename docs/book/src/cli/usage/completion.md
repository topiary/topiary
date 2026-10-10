# Shell completion generation

Shell completion scripts for Topiary can be generated with the
`completion` subcommand. The output of which can be sourced into your
shell session or profile, as required.

<!-- DO NOT REMOVE THE "usage:{start,end}" COMMENTS -->
<!-- usage:start -->
```
Generate shell completion script

Usage: topiary completion [OPTIONS] [SHELL]

Arguments:
  [SHELL]  Shell (omit to detect from the environment) [possible values: bash, elvish,
           fish, powershell, zsh]

Options:
  -C, --configuration <CONFIGURATION>  Configuration file [env: TOPIARY_CONFIG_FILE]
  -M, --merge-configuration            Enable merging for configuration files
  -v, --verbose...                     Logging verbosity (increased per occurrence)
      --skip-language <LANGUAGE>       Skip formatting the given language(s)
  -h, --help                           Print help
```
<!-- usage:end -->

For example, in Bash:

```bash
source <(topiary completion)
```
