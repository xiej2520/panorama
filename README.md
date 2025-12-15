# Rust Flake

## Building

```sh
cargo run --release

# with nix
nix build
nix run

# with submodules
nix build '.?submodules=1'# 
nix run '.?submodules=1'# 
```

Use direnv or `nix shell` to set up dev environment.

## Panorama

Minecraft screenshot code uses 90 FoV, 4096x4096 (but distributed as 1024x1024?)
- `panorama_0`: (y_start, 0)
- `panorama_1`: ((y_start + 90) % 360, 0)
- `panorama_2`: ((y_start + 180) % 360, 0)
- `panorama_3`: ((y_start - 90) % 360, 0)
- `panorama_4`: (y_start % 360, -90)
- `panorama_5`: (y_start, 90)
