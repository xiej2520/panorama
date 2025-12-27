# Rust Flake

## Building

```sh
cargo run --release -- packs/monument.zip

# with nix
nix build
nix run

# stitch binary
# expects panorama_<0-5>.png in cwd
cargo run --release --bin stitch
```

```sh
Usage: panorama [<path>] [--fps <fps>]

panorama

Positional Arguments:
  path              path to panorama_<x>.png image files or resource pack

Options:
  --fps             fps target
  --help, help      display usage information
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

## Controls

- `Right Click + Drag` to move camera
- `Scroll` to zoom in or out
- `O` to toggle panorama overlay
- `Space` to toggle rotation
- `Left Arrow` to decrease rotation speed (-0.5 degrees/second)
- `Right Arrow` to increase rotation speed (0.5 degrees/second)
- `Backspace` to reset camera
- `Escape` to close
- `F11` for fullscreen
- `R` to start recording (requires ffmpeg to be available in `PATH`). **Overwrites `output.mp4`**.

## TODO

- Safe(r) wrappers for OpenGL function calls (use `glium`?)
- WASM and display on web page
- Blur effect
- Better error handling, allow missing panorama images, drag and drop images
