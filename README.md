# haze-wayland-ssd

`haze-wayland-ssd` is the experimental Wayland server-side decoration preload
backend for Haze. It is built as a C-compatible Rust shared library and is
loaded into GNOME Shell/Mutter with a per-user preload environment.

The backend advertises `zxdg_decoration_manager_v1` when Mutter has not already
done so, creates decoration resources for clients that request them, and
responds with `MODE_SERVER_SIDE`.

## Supported Host Matrix

- Wayland: `1.22`, `1.23`, `1.24`
- GNOME Shell: `47`, `48`, `49`, `50`
- Mutter API: `15`, `16`, `17`, `18`

Unsupported hosts should not preload this library. Haze detects that state and
keeps the preferences page insensitive until the host is supported and the
backend has been installed.

## Build

```sh
cargo build --release
```

The resulting library is:

```text
target/release/libhaze_wayland_ssd.so
```

## Probe

```sh
tests/host_probe.sh
```
