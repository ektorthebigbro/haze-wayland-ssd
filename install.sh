#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0
# Copyright (C) 2026 Haze Project

set -euo pipefail

repo="ektorthebigbro/haze-wayland-ssd"
target_dir="${HAZE_EXTENSION_DIR:-"${HOME}/.local/share/gnome-shell/extensions/haze@ektorthebigbro.github.io/native"}"
skip_deps=0

usage() {
    cat <<'EOF'
Usage: install.sh [OPTIONS]

Build and install haze-wayland-ssd from the latest GitHub release source.

Options:
  --target-dir DIR  Native library install directory.
  --skip-deps       Do not install build dependencies automatically.
  -h, --help        Show this help.
EOF
}

while (($#)); do
    case "$1" in
        --target-dir)
            [[ $# -ge 2 ]] || { printf 'error: --target-dir requires a value\n' >&2; exit 2; }
            target_dir="$2"
            shift
            ;;
        --skip-deps)
            skip_deps=1
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            printf 'error: unknown option: %s\n\n' "$1" >&2
            usage >&2
            exit 2
            ;;
    esac
    shift
done

have() {
    command -v "$1" >/dev/null 2>&1
}

fail() {
    printf 'error: %s\n' "$*" >&2
    exit 1
}

run_root() {
    if [[ "$(id -u)" == 0 ]]; then
        "$@"
    elif have sudo; then
        sudo "$@"
    elif have pkexec; then
        pkexec "$@"
    else
        fail "need sudo or pkexec to install missing build dependencies"
    fi
}

install_deps() {
    [[ -f /etc/os-release ]] || return 0
    local id id_like
    id="$(. /etc/os-release && printf '%s' "${ID:-}")"
    id_like="$(. /etc/os-release && printf '%s' "${ID_LIKE:-}")"
    case " ${id} ${id_like} " in
        *fedora*|*rhel*|*centos*|*rocky*|*almalinux*|*nobara*)
            run_root dnf install -y git cargo rustc pkgconf-pkg-config wayland-devel glib2-devel
            ;;
        *arch*|*manjaro*|*endeavouros*)
            run_root pacman -S --needed --noconfirm git cargo rust pkgconf wayland glib2
            ;;
        *debian*|*ubuntu*|*linuxmint*|*pop*)
            run_root apt-get update
            run_root apt-get install -y git cargo rustc pkg-config libwayland-dev libglib2.0-dev
            ;;
        *suse*)
            run_root zypper --non-interactive install -y git cargo rust pkgconf-pkg-config wayland-devel glib2-devel
            ;;
        *)
            printf 'warning: unrecognised distro; install git, cargo, rustc, pkg-config, wayland, and glib development packages manually\n' >&2
            ;;
    esac
}

shell_major() {
    gnome-shell --version | sed -n 's/.* \([0-9][0-9]*\).*/\1/p' | head -n1
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

verify_host() {
    local shell mutter wayland
    shell="$(shell_major || true)"
    mutter="$(mutter_api || true)"
    wayland="$(pkg-config --modversion wayland-server 2>/dev/null || true)"
    case "${shell}" in 47|48|49|50) ;; *) fail "unsupported GNOME Shell version: ${shell:-unknown}" ;; esac
    case "${mutter}" in 15|16|17|18) ;; *) fail "unsupported Mutter API: ${mutter:-unknown}" ;; esac
    case "${wayland}" in 1.22*|1.23*|1.24*) ;; *) fail "unsupported wayland-server version: ${wayland:-unknown}" ;; esac
}

configure_preload() {
    local lib="${target_dir}/libhaze_wayland_ssd.so"
    local env_dir="${HOME}/.config/environment.d"
    install -d -m 0755 "${env_dir}"
    printf 'LD_PRELOAD=%s\n' "${lib}" > "${env_dir}/90-haze-wayland-ssd.conf"
    if have systemctl; then
        systemctl --user set-environment "LD_PRELOAD=${lib}" || true
    fi
    if have dbus-update-activation-environment; then
        LD_PRELOAD="${lib}" dbus-update-activation-environment --systemd LD_PRELOAD || true
    fi
}

[[ "${skip_deps}" == 1 ]] || install_deps
for cmd in curl tar cargo rustc pkg-config; do
    have "${cmd}" || fail "missing required command: ${cmd}"
done
for pkg in wayland-server wayland-client glib-2.0 gobject-2.0; do
    pkg-config --exists "${pkg}" || fail "missing pkg-config dependency: ${pkg}"
done
verify_host

tmp="$(mktemp -d)"
trap 'rm -rf "${tmp}"' EXIT
tag="$(
    curl -fsSL "https://api.github.com/repos/${repo}/releases/latest" |
        sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' |
        head -n1
)"
[[ -n "${tag}" ]] || fail "could not resolve latest ${repo} release tag"
curl -fL --retry 3 --connect-timeout 15 \
    "https://github.com/${repo}/archive/refs/tags/${tag}.tar.gz" |
    tar -xz -C "${tmp}" --strip-components=1

cargo build --release --manifest-path "${tmp}/Cargo.toml"
install -d -m 0755 "${target_dir}"
install -m 0644 "${tmp}/target/release/libhaze_wayland_ssd.so" "${target_dir}/libhaze_wayland_ssd.so"
configure_preload

cat <<EOF
haze-wayland-ssd installed to ${target_dir}/libhaze_wayland_ssd.so.
Restart GNOME Shell or log out and back in before using Window Decorations.
EOF
