#!/usr/bin/env bash

# "Quick-and-Dirty" Topiary Playground

set -euo pipefail

PROGNAME="$(basename "$0")"
readonly PROGNAME

FORMATTING_QUERY="formatting.scm"
readonly FORMATTING_QUERY

fail() {
  local error="$*"

  cat >&2 <<-EOF
	Error: ${error}

	Usage: ${PROGNAME} LANGUAGE [QUERY_FILE] [INPUT_SOURCE]

	LANGUAGE can be one of the supported languages (e.g., "ocaml", "rust",
	etc.). The packaged formatting queries for this language can be
	overridden by specifying a QUERY_FILE.

	The INPUT_SOURCE is optional. If not specified, it defaults to trying
	to find the bundled integration test input file for the given language.
	EOF

  exit 1
}

get-default-query() {
  local language="$1"
  local query

  case "${language}" in
    "ocaml_interface")
      query=ocaml
      ;;

    *)
      # NOTE Query files use dashes, instead of underscores
      query="${language//_/-}"
      ;;
  esac

  printf "topiary-queries/queries/%s/%s" "${query}" "${FORMATTING_QUERY}"
}

get-sample-input() {
  local language="$1"

  # Only return the first result, presuming there is one
  # NOTE Sample files use dashes, instead of underscores
  find topiary-cli/tests/samples/input -name "${language//_/-}.*" \
  | head -1
}

format() {
  local language="$1"
  local query="$2"
  local input="$3"
  local skip_idempotence="${4-1}"

  local -a cargo_args=(
    --quiet
    --no-default-features
    --features "${language}"
  )

  local -a topiary_args=(
    --language "${language}"
    --query "${query}"
  )

  (( skip_idempotence )) && topiary_args+=(--skip-idempotence)

  cargo run "${cargo_args[@]}" -- fmt "${topiary_args[@]}" < "${input}"
}

idempotency() {
  local language="$1"
  local query="$2"
  local input="$3"

  if format "${language}" "${query}" "${input}" 0 >/dev/null 2>&1; then
    printf "Yes"
  elif (( $? == 7 )); then
    printf "No"
  else
    printf "n/a"
  fi
}

main() {
  local language
  local query
  local input

  case $# in
    1)
      language="$1"
      query="$(get-default-query "${language}")"
      input="$(get-sample-input "${language}")"
      ;;

    2)
      language="$1"

      if [[ "$2" =~ \.scm$ ]]; then
        query="$2"
        input="$(get-sample-input "${language}")"
      else
        query="$(get-default-query "${language}")"
        input="$2"
      fi
      ;;

    3)
      language="$1"
      query="$2"
      input="$3"
      ;;

    *)
      fail "Invalid command line arguments"
      ;;
  esac

  [[ -e "${query}" ]] || fail "Couldn't find language query file '${query}'"
  [[ -e "${input}" ]] || fail "Couldn't find input source file '${input}'"

  # Horizontal rule (this is a function because executing it in a TTY-
  # -less subshell, to assign it to a variable, sets COLUMNS to 0)
  hr() { printf "%${COLUMNS}s\n" "" | tr " " "-"; }

  while true; do
    clear

    hr
    cat <<-EOF
		Language      ${language}
		Query File    ${query}
		Input Source  ${input}
		EOF
    hr

    format "${language}" "${query}" "${input}" || true
    hr
    printf "Idempotent    %s\n" "$(idempotency "${language}" "${query}" "${input}")"

    # NOTE Different editors have different strategies for modifying
    # files, so we wait on multiple events. This *may* not be an
    # exhaustive list; you are encouraged to experiment if the
    # playground doesn't refresh when you expect it to!
    inotifywait \
      -qq \
      --event modify \
      --event move_self \
      "${query}" "${input}"
  done

}

main "$@"
