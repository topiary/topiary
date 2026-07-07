# Suggested query development workflow

In order to work productively on query files, the following is one
suggested way to work:

1. If you're working on a new language, first follow the steps in [the
   previous chapter](adding-a-new-language.md).

2. Say you are working on formatting `mylanguage` code, then from the
   previous step (or otherwise), there should be two files that drive
   the test suite:

   - `topiary-cli/tests/samples/input/mylanguage/mylanguage.code`
   - `topiary-cli/tests/samples/expected/mylanguage/mylanguage.code`

   These respectively define the input, which will be formatted by
   Topiary when under test, and the expected output, which will be
   compared against the formatted input. (Note that the `.code`
   extension here is arbitrary, for illustrative purposes, but an
   appropriate extension is required.)

   Add a snippet of code to each of these files that exhibit the
   formatting you wish to implement as Tree-sitter queries. These
   snippets can be identical, but it would be a better test if the input
   version was intentionally misformatted.

   <div class="warning">
   If code already exists in these files, please ensure that the new
   snippet is both syntactically valid, in the context of the other
   code, and inserted at the same relative position in both files.
   </div>

3. Run:

   ```sh
   cargo test \
     --no-default-features \
     -F mylanguage \
     -p topiary-cli \
     input_output_tester \
     -- --nocapture
   ```

   Provided it works, it should output a _lot_ of log messages. Copy
   that output to a text editor. You are particularly interested in the
   CST output that starts with a line like this: `CST node: {Node
   compilation_unit (0, 0) - (5942, 0)} - Named: true`.

> **Note**\
> As an alternative to using the debugging output, the [`visualise`
> subcommand](../cli/usage/visualise.md) exists to output the
> Tree-sitter syntax tree in a variety of formats.

4. The test run will output all the differences between the actual
   output and the expected output, e.g. missing spaces between tokens.
   Pick a difference you would like to fix, and find the line number and
   column in the input file.

5. In the CST debug or visualisation output, find the nodes in this
   region, such as the following:

   ```
   [DEBUG atom_collection] CST node:   {Node constructed_type (39, 15) - (39, 42)} - Named: true
   [DEBUG atom_collection] CST node:     {Node type_constructor_path (39, 15) - (39, 35)} - Named: true
   [DEBUG atom_collection] CST node:       {Node type_constructor (39, 15) - (39, 35)} - Named: true
   [DEBUG atom_collection] CST node:     {Node type_constructor_path (39, 36) - (39, 42)} - Named: true
   [DEBUG atom_collection] CST node:       {Node type_constructor (39, 36) - (39, 42)} - Named: true
   ```

6. This may indicate that you would like spaces after all
   `type_constructor_path` nodes:

   ```scheme
   (type_constructor_path) @append_space
   ```

   Or, more likely, you just want spaces between pairs of them:

   ```scheme
   (
     (type_constructor_path) @append_space
     .
     (type_constructor_path)
   )
   ```

   Or maybe you want spaces between all children of `constructed_type`:

   ```scheme
   (constructed_type
     (_) @append_space
     .
     (_)
   )
   ```

> **Note**\
> Using the [`#query_name!`](../reference/capture-names/general.md#query_name)
> predicate will help identify which query does what.

7. Run `cargo test` again, to see if the output has improved, then
   return to step 4.
