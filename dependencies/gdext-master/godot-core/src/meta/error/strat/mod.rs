/*
 * Copyright (c) godot-rust; Bromeon and contributors.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Built-in [`ErrorToGodot`] strategies for mapping `Result<T, E>` return types of `#[func]` methods.
//!
//! This module is intended to be used qualified with `strat::` module: `strat::Unexpected` etc.
//! It is re-exported in the prelude to enable this.
//!
//! # Overview
//! Each type in this module implements [`ErrorToGodot`] with a different mapping strategy. Some strategies are not provided out-of-the-box
//! but could serve as inspiration to build your own. If you find any of those useful, let us know, and we may consider adding them.
//!
//! | Strategy                                       | `Mapped` type                       | Ok path            | Err path                  | GDScript ergonomics                     |
//! |------------------------------------------------|-------------------------------------|--------------------|---------------------------|-----------------------------------------|
//! | [`strat::Unexpected`]<br>Unexpected errors     | `T`                                 | `val`              | Default or<br>failed call | Sees `T`; `?` works with any `Error`    |
//! | `()`<br>Nil on error                           | `Variant`                           | `val.to_variant()` | `null`                    | `if val == null`                        |
//! | [`global::Error`]<br>Godot error enum          | `global::Error`                     | `OK` constant      | `ERR_*` constant          | `val == OK`                             |
//! | _(not provided)_<br>`Variant`                  | `Variant`                           | `val.to_variant()` | `err.to_variant()`        | Must check type/value                   |
//! | _(not provided)_<br>Dictionary `ok`/`err`      | `Dictionary`<br>`<GString,Variant>` | `{"ok" => val}`    | `{"err" => msg}`          | `d.has("ok")`<br>`d["ok"]`              |
//! | _(not provided)_<br>Array 0/1 elems            | `Array<T>`                          | `[val]`            | `[]`                      | `a.is_empty()`<br>`a.front()` -- typed! |
//! | _(not provided)_<br>Custom class               | `Gd<RustResult>`                    | wrap in class      | wrap in class             | `r.is_ok()`<br>`r.unwrap()`             |

mod unexpected;

pub use unexpected::*;

use super::{CallOutcome, ErrorToGodot};
use crate::builtin::Variant;
use crate::global;
use crate::meta::ToGodot;
#[expect(unused)] // for docs.
use crate::meta::error::strat;

// ----------------------------------------------------------------------------------------------------------------------------------------------
// () impl: GDScript sees Variant -- nil on error, val.to_variant() on success.

/// Error strategy that returns `null` on error, instead of making the call fail.
///
/// Use this when an absent value is a normal outcome that GDScript should handle, for example a missing save file
/// for a new player. GDScript receives a `Variant` containing either the value or `null`.
///
/// Since `()` discards all error information, use `.map_err(|_| ())?` to propagate any error into it.
///
/// # Example
/// ```no_run
/// use godot::prelude::*;
/// # #[derive(GodotClass)] #[class(init, base=Node)] struct PlayerData;
///
/// #[godot_api]
/// impl PlayerData {
///     // Returns the high score from a save file, or null if absent or unreadable.
///     // A missing file is normal for new players -- GDScript handles null gracefully.
///     #[func]
///     fn load_high_score(&self, save_path: String) -> Result<i64, ()> {
///         let text = std::fs::read_to_string(save_path).map_err(|_| ())?;
///         text.trim().parse::<i64>().map_err(|_| ())
///     }
/// }
/// ```
///
/// GDScript usage:
/// ```gdscript
/// var score = player.load_high_score("user://highscore.dat")
/// if score == null:
///     # New player, no save file yet.
/// ```
impl<T: ToGodot> ErrorToGodot<T> for () {
    type Mapped = Variant;

    fn result_to_godot(result: Result<T, Self>) -> CallOutcome<Variant> {
        match result {
            Ok(val) => CallOutcome::Return(val.to_variant()),
            Err(()) => CallOutcome::Return(Variant::nil()),
        }
    }
}

// ----------------------------------------------------------------------------------------------------------------------------------------------
// global::Error impl: GDScript sees the Error enum.
//
// Note: ok_to_mapped discards the Ok value and returns Error::OK. The typical use case is Result<(), global::Error>.

impl<T: ToGodot> ErrorToGodot<T> for global::Error {
    type Mapped = global::Error;

    fn result_to_godot(result: Result<T, Self>) -> CallOutcome<global::Error> {
        match result {
            Ok(_) => CallOutcome::Return(global::Error::OK),
            Err(e) => CallOutcome::Return(e),
        }
    }
}
