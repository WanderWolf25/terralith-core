/*
 * Copyright (c) godot-rust; Bromeon and contributors.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Commands related to parsing user-provided JSON and extension headers.

// At first re-using mapping from godot-codegen json.rs might seem more desirable but there are few issues to consider:
// * Overall JSON file structure might change slightly from version to version, while header should stay consistent (otherwise it defeats the purpose of having any header at all).
// Having two parsers – minimal one inherent to api-custom-json feature and codegen one – makes handling all the edge cases easier.
// * `godot-codegen` depends on `godot-bindings` thus simple re-using types from former in side the latter is not possible (cyclic dependency).
// Moving said types to `godot-bindings` would increase the cognitive overhead (since domain mapping is responsibility of `godot-codegen`, while godot-bindings is responsible for providing required resources & emitting the version).
// In the future we might experiment with splitting said types into separate crates.

use std::borrow::Cow;
use std::path::Path;
use std::sync::Once;
use std::{fs, panic};

use nanoserde::DeJson;

use crate::{GodotVersion, LATEST_API_VERSION, StopWatch, env_var_or_deprecated};

/// A minimal version of deserialized JsonExtensionApi that includes only the header.
#[derive(DeJson)]
struct JsonExtensionApi {
    pub header: JsonHeader,
}

/// Deserialized "header" key in given `extension_api.json`.
#[derive(DeJson)]
struct JsonHeader {
    pub version_major: u8,
    pub version_minor: u8,
    pub version_patch: u8,
    pub version_status: String,
    pub version_build: String,
    pub version_full_name: String,
}

impl JsonHeader {
    fn into_godot_version(self) -> GodotVersion {
        GodotVersion {
            full_string: self.version_full_name,
            major: self.version_major,
            minor: self.version_minor,
            patch: self.version_patch,
            status: self.version_status,
            custom_rev: Some(self.version_build),
        }
    }
}

pub fn load_gdextension_interface_json(watch: &mut StopWatch) -> Cow<'static, str> {
    println!("cargo:rerun-if-env-changed=GDRUST_GODOT_INTERFACE_JSON");
    watch.record("load_interface_json");

    if let Ok(path) = std::env::var("GDRUST_GODOT_INTERFACE_JSON")
        && let Ok(contents) = fs::read_to_string(&path)
    {
        Cow::Owned(contents)
    } else {
        gdextension_api::load_gdextension_interface_json()
    }
}

pub fn load_custom_extension_api_json() -> String {
    static WARN_ONCE: Once = Once::new();
    let env_var = env_var_or_deprecated(
        &WARN_ONCE,
        "GDRUST_GODOT_API_JSON",
        "GODOT4_GDEXTENSION_JSON",
    );

    println!("cargo:rerun-if-env-changed=GDRUST_GODOT_API_JSON");
    println!("cargo:rerun-if-env-changed=GODOT4_GDEXTENSION_JSON");

    let path = env_var.expect(
        "godot-rust with `api-custom-json` feature requires GDRUST_GODOT_API_JSON \
        environment variable (with the path to the said json).",
    );
    let json_path = Path::new(&path);

    fs::read_to_string(json_path).unwrap_or_else(|_| {
        panic!(
            "failed to open file with custom GDExtension JSON {}.",
            json_path.display()
        )
    })
}

/// Returns the Godot version specified in `extension_api.json`, or the version of the used header if newer.
pub(crate) fn read_godot_version() -> GodotVersion {
    let extension_api: JsonExtensionApi =
        DeJson::deserialize_json(&load_custom_extension_api_json())
            .expect("failed to deserialize JSON");

    let json_header_version = extension_api
        .header
        .into_godot_version()
        .validate_or_panic();

    if json_header_version.is_newer_than_latest()
        && std::env::var("GDRUST_GODOT_INTERFACE_JSON").is_err()
    {
        let (major, minor, patch) = LATEST_API_VERSION;

        // Note: this warning will be shown only with the extra verbose setting (`-vv`), on compilation error, or when compiling
        // this very workspace (i.e. when it is a local dependency).
        println!(
            "cargo::warning=Using Godot version API {h_version} specified in `GDRUST_GODOT_API_JSON` with \
            prebuilt Godot headers {major}.{minor}.{patch}.\
            Consider providing custom `gdextension_interface.json` with `GDRUST_GODOT_INTERFACE_JSON` env variable instead.",
            h_version = json_header_version.full_string,
        );
    }

    json_header_version
}
