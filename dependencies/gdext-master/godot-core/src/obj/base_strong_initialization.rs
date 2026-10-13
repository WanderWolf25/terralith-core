/*
 * Copyright (c) godot-rust; Bromeon and contributors.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::obj::base_init::InitState;
use crate::obj::{Base, Gd, GodotClass};

/// Tracks the initialization state of this `Base<T>`.
///
/// ZST for Godot >= 4.7, where tracking initialization state is no longer necressary.
#[derive(Clone)]
pub struct InitTracker;

impl<T: GodotClass> Base<T> {
    /// Since Godot 4.7 initialization layer receives fully-constructed base to work with – therefore it simply returns a clone of a given instance.
    #[doc(hidden)]
    pub(crate) fn to_init_gd_inner(&self) -> Gd<T> {
        self.__constructed_gd()
    }
}

impl InitTracker {
    pub fn new(_state: InitState) -> Self {
        Self
    }
    pub fn assert_constructed(&self) {}
    pub fn assert_script(&self) {}
}
