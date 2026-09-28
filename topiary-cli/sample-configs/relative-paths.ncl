# Test configuration exercising query-source resolution.
#
# Merged on top of the built-in configuration (which supplies the grammar git
# sources), this file overrides only the query sources for three languages,
# mixing the two resolution paths:
#
#   * json -- a *relative local* `path`, anchored at the directory of this file
#     (topiary-cli/tests/config/), not the working directory. The `../../../`
#     climbs back to the repository root and points at the checked-in query.
#
#   * toml -- a *git-backed* `formatting` source: `path` names a file inside the
#     checkout Topiary fetches and caches on demand, so it exercises the fetch +
#     copy path in `Language::resolve_query_path_with` and must be left as written.
#
#   * markdown -- git-backed sources for *both* `formatting` and `injections`
#     (markdown injects other languages), covering git query resolution for more
#     than one query per language.
{
  pinned_git = {
    git = "https://github.com/topiary/topiary",
    rev = "2eb5bd2d801c767f9a500a48d79c1a9b6a1877a7",
  },

  languages = {
    json.queries.formatting.source.path = "../../../topiary-queries/queries/json/formatting.scm",
    toml.queries.formatting.source = {
      git = pinned_git,
      path = "topiary-queries/queries/toml/formatting.scm",
    },
    markdown.queries = {
      formatting.source = {
        git = pinned_git,
        path = "topiary-queries/queries/markdown/formatting.scm",
      },
      injections.source = {
        git = pinned_git,
        path = "topiary-queries/queries/markdown/injections.scm",
      },
    },
  },
}
