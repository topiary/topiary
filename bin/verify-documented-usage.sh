#!/usr/bin/env nix-shell
#!nix-shell -i bash --packages diffutils gnused
#shellcheck shell=bash

set -euo pipefail

readonly FENCE='```'

get-cli-usage() {
  # Get the help text from the CLI
  local subcommand="$1"

  case "${subcommand}" in
    "index") nix run .#topiary-wrapped -- --help;;
    *)       nix run .#topiary-wrapped -- "${subcommand}" --help;;
  esac
}

get-documented-usage() {
  # Get the help text from the Topiary Book usage chapters
  local subcommand="$1"
  local chapter="docs/book/src/cli/usage/${subcommand}.md"

  sed --quiet "
    /usage:start/, /usage:end/ {
      //d          # Delete the markers (last pattern)
      /${FENCE}/d  # Delete the code fences
      p            # Print anything else
    }
  " "${chapter}"
}

diff-usage() {
  # Generate a diff between the README and CLI help text
  local subcommand="$1"

  diff --text \
       --ignore-all-space \
       --side-by-side \
       <(get-documented-usage "${subcommand}") \
       <(get-cli-usage "${subcommand}")
}

main() {
  # NOTE "index" is for the top-level usage documentation.
  # Each element in this array should correspond with a Markdown file in
  # docs/book/src/cli/usage
  local -a subcommands=(index format visualise config completion coverage prefetch check-grammar)

  local _diff
  local _subcommand
  for _subcommand in "${subcommands[@]}"; do
    if ! _diff=$(diff-usage "${_subcommand}"); then
      >&2 echo "CLI usage is not correctly documented in docs/book/src/cli/usage/${_subcommand}.md!"
      echo "${_diff}"
      exit 1
    fi
  done

  >&2 echo "Usage is correctly documented in the Topiary Book"
}

main
