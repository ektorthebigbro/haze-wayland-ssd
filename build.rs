// SPDX-License-Identifier: GPL-3.0
// Copyright (C) 2026 Haze Project

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const PROTOCOL_PATH: &str = "protocols/xdg-decoration-unstable-v1.xml";
const GENERATED_NAME: &str = "xdg_decoration_unstable_v1.rs";

fn validate_protocol(path: &Path) {
    let xml = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));

    for needle in [
        r#"<protocol name="xdg_decoration_unstable_v1">"#,
        r#"<interface name="zxdg_decoration_manager_v1" version="2">"#,
        r#"<interface name="zxdg_toplevel_decoration_v1" version="2">"#,
        r#"<event name="configure">"#,
    ] {
        assert!(
            xml.contains(needle),
            "protocol {} is not the expected xdg-decoration unstable v1 XML; missing {needle}",
            path.display()
        );
    }
}

fn main() {
    let manifest_dir =
        PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR"));
    let source = manifest_dir.join(PROTOCOL_PATH);
    let generated = out_dir.join(GENERATED_NAME);

    println!("cargo:rerun-if-changed={PROTOCOL_PATH}");
    validate_protocol(&source);

    fs::copy(&source, out_dir.join("xdg-decoration-unstable-v1.xml"))
        .unwrap_or_else(|error| panic!("failed to stage xdg-decoration XML: {error}"));

    fs::write(
        generated,
        format!(
            r#"pub mod __interfaces {{
    use wayland_server::protocol::__interfaces::*;
    wayland_scanner::generate_interfaces!("{PROTOCOL_PATH}");
}}

use self::__interfaces::*;
wayland_scanner::generate_server_code!("{PROTOCOL_PATH}");
"#
        ),
    )
    .unwrap_or_else(|error| panic!("failed to write generated protocol module: {error}"));

    let mutter_api = [18, 17, 16, 15]
        .into_iter()
        .find(|api| {
            pkg_config::Config::new()
                .probe(&format!("mutter-clutter-{api}"))
                .is_ok()
                && pkg_config::Config::new()
                    .probe(&format!("mutter-cogl-{api}"))
                    .is_ok()
        })
        .unwrap_or_else(|| panic!("failed to locate supported Mutter Clutter/Cogl pkg-config files; expected mutter-clutter-15..18 and mutter-cogl-15..18"));
    let clutter_name = format!("mutter-clutter-{mutter_api}");
    let cogl_name = format!("mutter-cogl-{mutter_api}");
    let mutter_name = format!("libmutter-{mutter_api}");
    let _clutter = pkg_config::Config::new()
        .probe(&clutter_name)
        .unwrap_or_else(|error| panic!("failed to locate {clutter_name} with pkg-config: {error}"));
    let _cogl = pkg_config::Config::new()
        .probe(&cogl_name)
        .unwrap_or_else(|error| panic!("failed to locate {cogl_name} with pkg-config: {error}"));
    let mutter = pkg_config::Config::new()
        .probe(&mutter_name)
        .unwrap_or_else(|error| panic!("failed to locate {mutter_name} with pkg-config: {error}"));

    for path in mutter.link_paths.iter() {
        if path.to_string_lossy().contains("mutter") {
            println!("cargo:rustc-link-arg=-Wl,-rpath,{}", path.display());
        }
    }
}
