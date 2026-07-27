#!/usr/bin/env bash
#shellcheck shell=bash

set -euo pipefail

if [[ "${1:-}" == "--no-nix-shell" ]]; then
  export NO_NIX_SHELL=1
  shift
fi

if [[ -z "${NO_NIX_SHELL:-}" && -z "${IN_NIX_SHELL:-}" ]] && command -v nix-shell >/dev/null; then
  # for macos: brew install gnu-sed awk
  # PATH="/opt/homebrew/opt/gnu-sed/libexec/gnubin:$PATH"
  exec nix-shell --pure --packages bash diffutils gnused --run "NO_NIX_SHELL=1 bash '$0' $*"
fi

export __TOPIARY_TERM_WIDTH=90

readonly FENCE='```'

# call `cargo build --bin topiary` once and reuse ./target/debug/topiary for on every `--help` call
if [[ -z "${TOPIARY:-}" ]]; then
  cargo build -q --bin topiary
  TOPIARY="${CARGO_TARGET_DIR:-target}/debug/topiary"
fi
readonly TOPIARY

get-cli-usage() {
  # Get the help text from the CLI
  local subcommand="$1"

  case "${subcommand}" in
    "index") "${TOPIARY}" --help;;
    *)       "${TOPIARY}" "${subcommand}" --help;;
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
