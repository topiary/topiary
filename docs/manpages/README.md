# `man topiary`

manpages for Topiary are generated from a subset of the [Topiary
Book](../book); specifically the Topiary CLI chapters. The same
mechanism is used to create manpages as the Topiary Book: [mdBook],
using the [`mdbook-man`][mdbook-man] renderer and a pre- and
post-processor written by the Topiary team.

## Getting started

> [!IMPORTANT]
> - You must use the v0.4 series of [mdBook (`mdbook`)[mdbook]. Newer
>   versions have introduced breaking changes.
>
> - You must use the latest `HEAD` of [`mdbook-man`][mdbook-man]; the
>   v0.1 release is insufficient:
>
>   ```console
>   $ cargo install --git https://github.com/vv9k/mdbook-man.git
>   ```
>
> - [`mdbook-manmunge`][mdbook-manmunge] is designed to work with the
>   above dependencies.
>
> These three binaries must be in your `$PATH` to build.

To build the manpages and install, run:

```console
$ make
$ make install
```

By default the manpages will be installed to
`~/.local/share/man/man1/topiary.1.gz`. The `~/.local/share/man` prefix
can be overridden by setting the `MAN_DIR` variable. For example:

```console
$ make
$ sudo make install MAN_DIR=/usr/share/man
```

### Using Nix

The Topiary Nix devshell provides correct versions of  mdBook,
`mdbook-man` and `mdbook-manmunge`, comprising the whole toolchain for
building the manpages.

Alternatively, a package exists that will build the manpages as its
derivation output, handling all the dependencies for you. For example:

```console
$ nix build .#topiary-manpages
```

## `mdbook-manmunge`

The pre- and post-processor has been designed to mould certain aspects
of the output of `mdbook-man` into a more palatable form. This is
arguably the wrong approach; a better solution would have been to fork
`mdbook-man` and make the necessary changes there. This may happen in
the future, but this solution will do for now.

Please see the [`mdbook-manmunge` repository][mdbook-manmunge-repo] for
more details.

### Installation

```console
$ cargo install mdbook-manmunge
```

> [!TIP]
> You will need to include Cargo's binary directory (e.g.,
> `~/.cargo/bin`) in your `$PATH`.

<!-- Links -->
[mdbook]: https://rust-lang.github.io/mdBook
[mdbook-man]: https://github.com/vv9k/mdbook-man
[mdbook-manmunge]: #mdbook-manmunge
[mdbook-manmunge:repo]: https://github.com/topiary/mdbook-manmunge
