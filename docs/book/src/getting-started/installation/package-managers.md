# Package managers

Topiary has been packaged for some package managers. However, note that
packaged versions _may_ lag behind the current release.

## Cargo (Rust package manager)

The Topiary CLI, amongst other things, is available on
[crates.io](https://crates.io/crates/topiary-cli):

```sh
cargo install topiary-cli
```

## Docker

Prebuilt, lightweight Docker images of the Topiary CLI are available on
[GitHub](https://github.com/topiary/topiary/pkgs/container/topiary).

```sh
docker pull ghcr.io/topiary/topiary:latest
```

For example:

```sh
docker run --rm -i ghcr.io/topiary/topiary format -l json <<< '{"foo":123}'
```

You will need to orchestrate Docker bind mounts (`-v`/`--volume`) in
order to format files on disk.

<div class="warning">

Docker images have been constructed to contain all the supported
grammars defined in the shipped configuration. However, importantly,
they do not contain a C/C++ toolchain, so are not independently capable
of building other grammars.

</div>

## OPAM (OCaml Package Manager)

Topiary is available through [OPAM](https://opam.ocaml.org/packages/topiary)
for the purposes of formatting OCaml code:

```sh
opam install topiary
```

Development of this package can be found on [GitHub at
`tweag/topiary-opam`](https://github.com/topiary/topiary-opam).

## Nix (nixpkgs)

Topiary [exists within nixpkgs](https://search.nixos.org/packages?show=topiary)
and can therefore be installed in whichever way you prefer. For example:

### NixOS (`configuration.nix`)

```nix
environment.systemPackages = with pkgs; [
  topiary
];
```

### Home Manager (`home.nix`)

```nix
home.packages = with pkgs; [
  topiary
];
```

### Nix install

```sh
# Using flakes:
nix profile install nixpkgs#topiary

# Or, without flakes:
# (Note: Use nixos.topiary on NixOS)
nix-env -iA nixpkgs.topiary
```

### `nix-shell`

To temporarily add Topiary to your path, use:

```sh
# Using flakes:
nix shell nixpkgs#topiary

# Or, without flakes:
nix-shell -p topiary
```

## Arch Linux (AUR)

Topiary is available on the [Arch user repository](https://aur.archlinux.org/packages/topiary):

```sh
yay -S topiary
```

## Homebrew

Topiary is available on [Homebrew](https://formulae.brew.sh/formula/topiary):

```sh
brew install topiary
```
