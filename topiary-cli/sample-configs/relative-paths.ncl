# * json:     formatting.source.path is anchored relative to this configuration file
# * toml:     git-backed `formatting` query source
# * markdown: git-backed sources for *both* `formatting` and `injections` queries
{
  pinned_git = {
    git = "https://github.com/topiary/topiary",
    rev = "2eb5bd2d801c767f9a500a48d79c1a9b6a1877a7",
  },

  languages = {
    json.queries.formatting.source.path = "../../topiary-queries/queries/json/formatting.scm",
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
