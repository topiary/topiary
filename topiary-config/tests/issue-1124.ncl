# https://github.com/topiary/topiary/issues/1124
{
  languages.ocaml = {
    extensions = ["ml"],
    grammar.source.library_path.path = "/some/path",
  },
}
