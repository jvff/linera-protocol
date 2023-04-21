#[path = "wit_import/wasmer.rs"]
mod wasmer;
#[path = "wit_import/wasmtime.rs"]
mod wasmtime;

use witty_macros::wit_import;

#[wit_import(package = "witty-macros:test-modules")]
trait SimpleFunction {
    fn simple();
}

#[wit_import(package = "witty-macros:test-modules")]
trait Getters {
    fn get_true() -> bool;
    fn get_false() -> bool;
    fn get_s8() -> i8;
    fn get_u8() -> u8;
    fn get_s16() -> i16;
    fn get_u16() -> u16;
    fn get_s32() -> i32;
    fn get_u32() -> u32;
    fn get_float32() -> f32;
    fn get_float64() -> f64;
}

#[wit_import(package = "witty-macros:test-modules")]
trait Setters {
    fn set_bool(value: bool);
    fn set_s8(value: i8);
    fn set_u8(value: u8);
    fn set_s16(value: i16);
    fn set_u16(value: u16);
    fn set_s32(value: i32);
    fn set_u32(value: u32);
    fn set_float32(value: f32);
    fn set_float64(value: f64);
}

#[wit_import(package = "witty-macros:test-modules")]
trait Operations {
    fn and_bool(first: bool, second: bool) -> bool;
    fn add_s8(first: i8, second: i8) -> i8;
    fn add_u8(first: u8, second: u8) -> u8;
    fn add_s16(first: i16, second: i16) -> i16;
    fn add_u16(first: u16, second: u16) -> u16;
    fn add_s32(first: i32, second: i32) -> i32;
    fn add_u32(first: u32, second: u32) -> u32;
    fn add_float32(first: f32, second: f32) -> f32;
    fn add_float64(first: f64, second: f64) -> f64;
}
