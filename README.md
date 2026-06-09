# haze-wayland-ssd

`haze-wayland-ssd` is the experimental Wayland server-side decoration preload
backend for Haze. It is built as a C-compatible Rust shared library and is
loaded into GNOME Shell/Mutter with a per-user preload environment.

The backend advertises `zxdg_decoration_manager_v1` when Mutter has not already
done so, creates decoration resources for clients that request them, and
defaults undecided toplevels to server-side mode. Explicit client-side requests
are honored so applications with embedded controls keep their own buttons;
clients that request server-side mode, or unset their preference, can be handed
to Haze's Shell-side replacement frame path.

Xwayland windows do not speak `zxdg-decoration`, so they cannot receive a
Wayland decoration configure event. For those clients the library exposes a
C-compatible decoration policy bridge. Haze applies that policy through
Mutter's `MetaWindowActor` path so Xwayland and Wayland windows share the same
replacement frame behavior without stripping embedded app controls.

## Supported Host Matrix

- Wayland: `1.22`, `1.23`, `1.24`
- GNOME Shell: `47`, `48`, `49`, `50`
- Mutter API: `15`, `16`, `17`, `18`

Unsupported hosts should not preload this library. Haze detects that state and
keeps the preferences page insensitive until the host is supported and the
backend has been installed.

## Build

Install from the latest GitHub release:

```sh
curl -fsSL https://github.com/ektorthebigbro/haze-wayland-ssd/releases/latest/download/install.sh | bash
```

Restart GNOME Shell or log out and back in after installing or replacing the
preload library.

Build from a source checkout:

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
