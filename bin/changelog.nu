#!/usr/bin/env nu

const ROOT_DIR: path = (path self | path dirname --num-levels 2)
const CHANGELOG_MD: path = $ROOT_DIR | path join "CHANGELOG.md"

export def main [
    pr_ref?: oneof<int, string> # PR number, url, or feature branch name
    --save # save the changes back to CHANGELOG.md
    --section: string = "Changed" # the "Unreleased" h3 section to add the entry to
    --no-cc-header # strip the leading conventional commit header: 'feat(core): bla bla'
] {
    # default to feature branch name if $pr_ref is not provided
    let $pr_ref = $pr_ref | default (git rev-parse --abbrev-ref HEAD)

    let changelog_raw = open $CHANGELOG_MD --raw
    let changelog_md: table = $changelog_raw
        | from md --verbose
        | enumerate # index our AST
        | flatten

    # find index of first h3 that matches section
    # we assume this is under Unreleased: the first h2
    let section_idx = $changelog_md
        | where type == "h3" and children.0.attrs.value == $section
        | first | get index

    # find the first list where the following item is not a list element
    let last_list_item_line = $changelog_md
        | skip ($section_idx + 1)
        | where $it.type == "list" and (
            $changelog_md
            | get --optional ($it.index + 1)
            | let next # ensure the next element is not a list
            | $next.type? != "list"
        )
        | first
        | get children
        | flatten --all
        | last
        | get position.end.line

    let pr: record<title, url, number> = (
        gh pr view $pr_ref --json title,url,number
        | from json
    )


    # strip conventional commit header and titlecase the first word
    let title = if $no_cc_header {
        $pr.title | str replace --regex '^\S+: ?' ''
    } else {
        $pr.title
    } | str replace --regex '^(\w)' {|match| $match | str uppercase}
    let list_item = $"- [#($pr.number)]\(($pr.url)) ($title)"

    # Split the raw changelog into lines
    mut changelog_lines: list<string> = ($changelog_raw | split row "\n")

    # Insert the new list item after the last existing list item (at last_list_item_line, which is 1-indexed)
    let changelog_output = (
        $changelog_lines | insert $last_list_item_line $list_item | str join "\n"
    )

    if $save {
        $changelog_output | save --force $CHANGELOG_MD
        git diff $CHANGELOG_MD
    } else {
        {addition: $list_item}
    }
}
