# Rust Flake

## Building

```sh
nix build
nix run

# with submodules
nix build '.?submodules=1'# 
nix run '.?submodules=1'# 
```

Use direnv or `nix shell` to set up dev environment.
