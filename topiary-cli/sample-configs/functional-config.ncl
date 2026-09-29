# Example of functional configuration
let formatting_only =
  ["bash", "css", "json", "nickel"]
  |> std.array.map (fun lang => { field = lang, value = { queries.formatting.source.path = "../../topiary-queries/queries/%{lang}/formatting.scm" } })
  |> std.record.from_array
# Merged with explicit configuration
in {
  languages =
    {
      markdown.queries = {
        formatting.source.path = "../../topiary-queries/queries/markdown/formatting.scm",
        injections.source.path = "../../topiary-queries/queries/markdown/injections.scm"
      },
      rust.queries = {
        formatting.source.path = "../../topiary-queries/queries/rust/formatting.scm",
        injections.source.path = "../../topiary-queries/queries/rust/injections.scm"
      },
    }
    & formatting_only
}
