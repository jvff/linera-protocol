#[path = "wit_export/wasmer.rs"]
mod wasmer;
#[path = "wit_export/wasmtime.rs"]
mod wasmtime;

use {
    witty::WitExport,
    witty_macros::{wit_export, wit_import},
};

#[wit_import(package = "witty-macros:test-modules")]
pub trait Entrypoint {
    fn entrypoint();
}

pub struct SimpleFunction;

#[wit_export(package = "witty-macros:test-modules")]
impl SimpleFunction {
    fn simple() {
        println!("In simple");
    }
}

pub struct Getters;

#[wit_export(package = "witty-macros:test-modules")]
impl Getters {
    fn get_true() -> bool {
        true
    }

    fn get_false() -> bool {
        false
    }

    fn get_s8() -> i8 {
        -125
    }

    fn get_u8() -> u8 {
        200
    }

    fn get_s16() -> i16 {
        -410
    }

    fn get_u16() -> u16 {
        60_000
    }

    fn get_s32() -> i32 {
        -100_000
    }

    fn get_u32() -> u32 {
        3_000_111
    }

    fn get_float32() -> f32 {
        -0.125
    }

    fn get_float64() -> f64 {
        128.25
    }
}

pub struct Setters;

#[wit_export(package = "witty-macros:test-modules")]
impl Setters {
    fn set_bool(value: bool) {
        assert_eq!(value, false);
    }

    fn set_s8(value: i8) {
        assert_eq!(value, -100);
    }

    fn set_u8(value: u8) {
        assert_eq!(value, 201);
    }

    fn set_s16(value: i16) {
        assert_eq!(value, -20_000);
    }

    fn set_u16(value: u16) {
        assert_eq!(value, 50_000);
    }

    fn set_s32(value: i32) {
        assert_eq!(value, -2_000_000);
    }

    fn set_u32(value: u32) {
        assert_eq!(value, 4_000_000);
    }

    fn set_float32(value: f32) {
        assert_eq!(value, 10.4);
    }

    fn set_float64(value: f64) {
        assert_eq!(value, -0.000_08);
    }
}

pub struct Operations;

#[wit_export(package = "witty-macros:test-modules")]
impl Operations {
    fn and_bool(first: bool, second: bool) -> bool {
        first && second
    }

    fn add_s8(first: i8, second: i8) -> i8 {
        first + second
    }

    fn add_u8(first: u8, second: u8) -> u8 {
        first + second
    }

    fn add_s16(first: i16, second: i16) -> i16 {
        first + second
    }

    fn add_u16(first: u16, second: u16) -> u16 {
        first + second
    }

    fn add_s32(first: i32, second: i32) -> i32 {
        first + second
    }

    fn add_u32(first: u32, second: u32) -> u32 {
        first + second
    }

    fn add_s64(first: i64, second: i64) -> i64 {
        first + second
    }

    fn add_u64(first: u64, second: u64) -> u64 {
        first + second
    }

    fn add_float32(first: f32, second: f32) -> f32 {
        first + second
    }

    fn add_float64(first: f64, second: f64) -> f64 {
        first + second
    }
}
