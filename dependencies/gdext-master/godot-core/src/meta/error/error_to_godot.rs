/*
 * Copyright (c) godot-rust; Bromeon and contributors.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! The [`ErrorToGodot`] trait for mapping `Result<T, E>` to Godot return types.
//!
//! Built-in strategies are in the [`strat`][super::strat] module.

use crate::meta::ToGodot;

/// Defines how `Result<T, E>` returned by `#[func]` is mapped to Godot.
///
/// When implemented for a type `E`, this trait enables `Result<T, E>` return types in `#[func]` methods.
///
/// # Implementing the trait
/// The associated type [`Mapped`][Self::Mapped] determines what GDScript sees as the function's return type. This type
/// can depend on `T` -- the ok-value type of the `Result` -- because the trait is generic over `T`.
///
/// Users then override [`result_to_godot()`][Self::result_to_godot], returning a [`CallOutcome`]:
/// - [`CallOutcome::Return(mapped)`][CallOutcome::Return] -- the call succeeds; pass `mapped` back to GDScript.
/// - [`CallOutcome::CallFailed(msg)`][CallOutcome::CallFailed] -- an unexpected error occurred; log `msg` and fail the call.
///
/// # Built-in strategies
/// See the [`strat`][crate::meta::error::strat] module for all provided implementations, or for inspirations for custom error handling.
///
/// # Example: typed `Array<T>` with 0 or 1 elements
/// Since the trait is generic over `T`, custom implementations can require tighter bounds (such as [`Element`][crate::meta::Element]) and use
/// a typed `Array<T>` as the mapped type.
///
/// This example returns a 1-element array on success, or a 0-element one on error -- a poor man's `Option<T>` in GDScript.
///
/// ```no_run
/// # use godot::prelude::*;
/// use godot::builtin::Array;
/// use godot::meta::error::{CallOutcome, ErrorToGodot};
/// use godot::meta::{Element, owned_into_arg};
///
/// struct MyError(String);
///
/// impl<T: Element> ErrorToGodot<T> for MyError {
///     // GDScript sees Array[T] as the #[func]'s return type.
///     type Mapped = Array<T>;
///
///     fn result_to_godot(result: Result<T, Self>) -> CallOutcome<Array<T>> {
///         // Construct [elem] or [].
///         let array = match result {
///             Ok(elem) => array![owned_into_arg(elem)],
///             Err(_) => Array::new(),
///         };
///
///         // We always return a value, never fail the call -> only use CallOutcome::Return.
///         CallOutcome::Return(array)
///     }
/// }
/// ```
///
/// GDScript usage:
/// ```gdscript
/// var result := node.some_fn()  # typed Array[...]
/// if result.is_empty():
///     print("Operation failed")
/// else:
///     var value := result.front()  # typed!
/// ```
//
// Potential expansion: ToGodot for Result<T, E> for infallible strategies (e.g. (), global::Error).
// Backwards-compatible. By-ref needed for ToGodot taking &self. Rough sketch:
//
//     pub trait ErrorToGodotInfallible<T: ToGodot>: ErrorToGodot<T> {
//         fn result_to_godot_ref(r: Result<&T, &Self>) -> Self::Mapped;
//     }
//     impl<T, E> ToGodot for Result<T, E> where E: ErrorToGodotInfallible<T> { ... }
//
pub trait ErrorToGodot<T: ToGodot>: Sized {
    /// The type to which `Result<T, Self>` is mapped on Godot side.
    type Mapped: ToGodot;

    /// Map an owned `Result` to a Godot return value or an unexpected-error message.
    fn result_to_godot(result: Result<T, Self>) -> CallOutcome<Self::Mapped>;
}

/// Outcome of mapping a `Result<T, E>` for a `#[func]` return value.
///
/// Returned by [`ErrorToGodot::result_to_godot()`]. Decides how Godot handles the result of a user-defined `#[func]`.
pub enum CallOutcome<R> {
    /// Pass this value back to GDScript; the call succeeds.
    Return(R),

    /// The call encounters an unexpected error; log provided message and perform best-effort failure handling.
    ///
    /// This either stops the calling GDScript function or results in a default value of `R` on Godot side. Rust callers using
    /// `Object::try_call()` always receive `Err`. For detailed Godot-side semantics and an example, see
    /// [`strat::Unexpected`][crate::meta::error::strat::Unexpected].
    CallFailed(String),
}

// ----------------------------------------------------------------------------------------------------------------------------------------------
// Macro for immediately exiting function.

/// Return early from a `#[func]`, creating an error value from a format string (including string literals).
///
/// Same principle as [`eyre::bail!`](https://docs.rs/eyre/latest/eyre/macro.bail.html),
/// [`miette::bail!`](https://docs.rs/miette/latest/miette/macro.bail.html), and
/// [`anyhow::bail!`](https://docs.rs/anyhow/latest/anyhow/macro.bail.html).
///
/// This macro expands to `return Err(E::from(format!(...)))`, where `E` is inferred from the function's return type.
/// Accepts a string literal or a `format!`-style format string with arguments.
///
/// Works with any error type `E` that implements `From<String>`, e.g. [`strat::Unexpected`][crate::meta::error::strat::Unexpected].
#[macro_export]
macro_rules! func_bail {
    ($($arg:tt)*) => {
        return ::std::result::Result::Err(::std::convert::From::from(
            ::std::format!($($arg)*)
        ))
    };
}
