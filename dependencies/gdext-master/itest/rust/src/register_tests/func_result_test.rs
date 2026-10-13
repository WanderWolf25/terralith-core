/*
 * Copyright (c) godot-rust; Bromeon and contributors.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use godot::builtin::VariantType;
use godot::global::Error;
use godot::meta::error::func_bail;
use godot::prelude::*;

use crate::framework::itest;

#[derive(GodotClass)]
#[class(init, base=RefCounted)]
struct FuncResulter;

#[godot_api]
impl FuncResulter {
    // ------------------------------------------------------------------------------------------------------------------------------------------
    // Result<T, global::Error> -- recoverable enum

    #[func]
    fn error_ok(&self) -> Result<(), Error> {
        Ok(())
    }

    #[func]
    fn error_failed(&self) -> Result<(), Error> {
        Err(Error::FAILED)
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------
    // Result<T, strat::Unexpected> -- fatal with ? ergonomics

    #[func]
    fn ok_int(&self) -> Result<i64, strat::Unexpected> {
        // Simulates ? on std::num::ParseIntError.
        let value = "42".parse::<i64>()?;
        Ok(value)
    }

    #[func]
    fn err_io(&self) -> Result<GString, strat::Unexpected> {
        // std::io::Error auto-converts via From.
        let _data = std::fs::read_to_string("/nonexistent/path")?;
        unreachable!()
    }

    #[func]
    fn err_bail(&self) -> Result<i64, strat::Unexpected> {
        func_bail!("custom message");
    }

    #[func]
    fn ok_unit(&self) -> Result<(), strat::Unexpected> {
        Ok(())
    }

    #[func]
    fn err_string(&self) -> Result<i64, strat::Unexpected> {
        // String also converts via From (through Into<Box<dyn Error>>).
        let err: String = "string error".into();
        Err(err.into())
    }

    #[func]
    fn err_parse(&self) -> Result<i64, strat::Unexpected> {
        // ? converts ParseIntError into strat::Unexpected automatically.
        let score = "not_a_number".parse::<i64>()?;
        Ok(score)
    }
}

// ----------------------------------------------------------------------------------------------------------------------------------------------
// Tests

#[itest]
fn func_result_ok_returns_value() {
    let mut obj = FuncResulter::new_gd();

    let result = obj.call("ok_int", &[]);
    assert_eq!(result.get_type(), VariantType::INT);
    assert_eq!(i64::from_variant(&result), 42);
}

#[itest]
fn func_result_ok_unit_returns_nil() {
    let mut obj = FuncResulter::new_gd();

    let result = obj.call("ok_unit", &[]);
    assert_eq!(result.get_type(), VariantType::NIL);
}

#[itest]
fn func_result_try_call_ok_returns_value() {
    let mut obj = FuncResulter::new_gd();

    let result = obj.try_call("ok_int", &[]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().to::<i64>(), 42);
}

#[itest]
fn func_result_err_fails_call() {
    let mut obj = FuncResulter::new_gd();

    assert!(obj.try_call("err_io", &[]).is_err());
    assert!(obj.try_call("err_bail", &[]).is_err());
    assert!(obj.try_call("err_string", &[]).is_err());
    assert!(obj.try_call("err_parse", &[]).is_err());
}

// ----------------------------------------------------------------------------------------------------------------------------------------------
// Other strategies (1 test per strategy)

#[itest]
fn global_error_returns_enum_value() {
    let mut obj = FuncResulter::new_gd();

    let ok = obj.call("error_ok", &[]);
    assert_eq!(ok.get_type(), VariantType::INT);
    assert_eq!(ok.to::<Error>(), Error::OK);

    let err = obj.call("error_failed", &[]);
    assert_eq!(err.get_type(), VariantType::INT);
    assert_eq!(err.to::<Error>(), Error::FAILED);
}
