# Using Nix

Topiary provides a flake for easy (re)use via Nix. It is also possible to use
Topiary's Nix code without going through flakes, although some inputs are harder
to provide than others -- have a look at the `nix/` directory.

The main package is `topiary-cli`. However, because Topiary features dynamic
loading of grammars, which requires network access the first time, it might not
play well with sandboxed environments such as those used to build Nix
derivations.

To tackle this, we provide a Nix library capable of pre-fetching grammars into
derivations. The main functions of interest are `lib.prefetchLanguages`, which
turns a Topiary configuration into one with pre-fetched grammars, and
`lib.wrapWithConfig`, a convenience to pass such configurations to Topiary. See
the Nix API reference for more details.

We provide building blocks rather than all-made solutions because we are unsure
at this point of how people will use Topiary in their Nix code. More specific
helpers might be written in the future to cover specific use cases.

One exception is our integration with [`git-hooks.nix`], where we provide a
ready-to-use helper, `lib.gitHook`. For instance, you can add Topiary as a
formatter in your project's pre-commit hooks with:

``` nix
pre-commit-check = git-hooks-nix.run {
  hooks = {
    nixfmt.enable = true; ## keep your normal hooks
    ...
    ## Add the following:
    topiary-latest = topiary.lib.${system}.gitHook // { enable = true; };
  };
};
```

[`git-hooks.nix`]: https://github.com/cachix/git-hooks.nix
