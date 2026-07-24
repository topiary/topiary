# User-side override that sets ONLY the path on a `QuerySource`.
# The path-only alternative of the `any_of` in `any-of-base.ncl` should
# match, but Nickel currently rejects this record with
# `error: missing definition for "git"` instead of falling through.
{
  languages.markdown.queries = {
    formatting.source.path = "/tmp/formatting.scm",
    injections.source.path = "/tmp/injections.scm",
  },
}
