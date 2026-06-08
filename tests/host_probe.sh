#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0
# Copyright (C) 2026 Haze Project

set -euo pipefail

have() {
    command -v "$1" >/dev/null 2>&1
}

shell_major() {
    if have gnome-shell; then
        gnome-shell --version | sed -n 's/.* \([0-9][0-9]*\).*/\1/p' | head -n1
    fi
}

mutter_api() {
    for api in 18 17 16 15; do
        if pkg-config --exists "libmutter-${api}" 2>/dev/null; then
            printf '%s\n' "${api}"
            return 0
        fi
    done
    return 1
}

wayland_version() {
    pkg-config --modversion wayland-server 2>/dev/null || true
}

printf 'gnome-shell=%s\n' "$(shell_major || true)"
printf 'mutter-api=%s\n' "$(mutter_api || true)"
printf 'wayland-server=%s\n' "$(wayland_version)"

case "$(shell_major || true)" in
    47|48|49|50) ;;
    *) printf 'unsupported: GNOME Shell version is outside 47-50\n' >&2; exit 1 ;;
esac

case "$(mutter_api || true)" in
    15|16|17|18) ;;
    *) printf 'unsupported: Mutter API is outside 15-18\n' >&2; exit 1 ;;
esac

case "$(wayland_version)" in
    1.22*|1.23*|1.24*) ;;
    *) printf 'unsupported: wayland-server version is outside 1.22-1.24\n' >&2; exit 1 ;;
esac

printf 'supported\n'
