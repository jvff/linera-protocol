// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Helper Wasm module for reentrancy tests with a mixed functions on the host side, one using a
//! caller parameter and one without.

#![cfg_attr(target_arch = "wasm32", no_main)]

wit_bindgen::generate!("reentrant-mixed-functions");

export_reentrant_mixed_functions!(Implementation);

use self::{
    exports::witty_macros::test_modules::{
        entrypoint::Entrypoint, simple_function::SimpleFunction,
    },
    witty_macros::test_modules::mixed_functions::*,
};

struct Implementation;

impl Entrypoint for Implementation {
    fn entrypoint() {
        with_caller();
        assert_eq!(without_caller(), 100);
    }
}

impl SimpleFunction for Implementation {
    fn simple() {}
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {}
