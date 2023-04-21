use {
    super::{Entrypoint, ExportedSimpleFunction},
    witty::{
        wasmer::{Imports, StoreRuntime},
        WitExport,
    },
};

#[test]
fn simple_function() {
    let store = load_test_module("simple-function", |imports| {
        ExportedSimpleFunction::export(imports)
            .expect("Failed to export simple function WIT interface")
    });

    Entrypoint::new(store)
        .entrypoint()
        .expect("Failed to execute test of imported simple-function");
}

// #[test]
// fn getters() {
// let store = load_test_module("getters");

// let mut getters = Getters::new(store);

// assert_eq!(
// getters
// .get_true()
// .expect("Failed to run guest's `get-true` function"),
// true
// );
// assert_eq!(
// getters
// .get_false()
// .expect("Failed to run guest's `get-false` function"),
// false
// );
// assert_eq!(
// getters
// .get_s8()
// .expect("Failed to run guest's `get-s8` function"),
// -125
// );
// assert_eq!(
// getters
// .get_u8()
// .expect("Failed to run guest's `get-u8` function"),
// 200
// );
// assert_eq!(
// getters
// .get_s16()
// .expect("Failed to run guest's `get-s16` function"),
// -410
// );
// assert_eq!(
// getters
// .get_u16()
// .expect("Failed to run guest's `get-u16` function"),
// 60_000
// );
// assert_eq!(
// getters
// .get_s32()
// .expect("Failed to run guest's `get-s32` function"),
// -100_000
// );
// assert_eq!(
// getters
// .get_u32()
// .expect("Failed to run guest's `get-u32` function"),
// 3_000_111
// );
// assert_eq!(
// getters
// .get_float32()
// .expect("Failed to run guest's `get-f32` function"),
// -0.125
// );
// assert_eq!(
// getters
// .get_float64()
// .expect("Failed to run guest's `get-f64` function"),
// 128.25
// );
// }

// #[test]
// fn setters() {
// let store = load_test_module("setters");

// let mut setters = Setters::new(store);

// setters
// .set_bool(false)
// .expect("Failed to run guest's `set-bool` function");
// setters
// .set_s8(-100)
// .expect("Failed to run guest's `set-s8` function");
// setters
// .set_u8(201)
// .expect("Failed to run guest's `set-u8` function");
// setters
// .set_s16(-20_000)
// .expect("Failed to run guest's `set-s16` function");
// setters
// .set_u16(50_000)
// .expect("Failed to run guest's `set-u16` function");
// setters
// .set_s32(-2_000_000)
// .expect("Failed to run guest's `set-s32` function");
// setters
// .set_u32(4_000_000)
// .expect("Failed to run guest's `set-u32` function");
// setters
// .set_float32(10.5)
// .expect("Failed to run guest's `set-f32` function");
// setters
// .set_float64(-0.000_08)
// .expect("Failed to run guest's `set-f64` function");
// }

// #[test]
// fn operations() {
// let store = load_test_module("operations");

// let mut operations = Operations::new(store);

// assert_eq!(
// operations
// .and_bool(false, true)
// .expect("Failed to run guest's `and-bool` function"),
// false
// );
// assert_eq!(
// operations
// .and_bool(true, true)
// .expect("Failed to run guest's `and-bool` function"),
// true
// );
// assert_eq!(
// operations
// .add_s8(-126, 1)
// .expect("Failed to run guest's `add-s8` function"),
// -125
// );
// assert_eq!(
// operations
// .add_u8(189, 11)
// .expect("Failed to run guest's `add-u8` function"),
// 200
// );
// assert_eq!(
// operations
// .add_s16(-400, -10)
// .expect("Failed to run guest's `add-s16` function"),
// -410
// );
// assert_eq!(
// operations
// .add_u16(32_000, 28_000)
// .expect("Failed to run guest's `add-u16` function"),
// 60_000
// );
// assert_eq!(
// operations
// .add_s32(-2_000_000, 1_900_000)
// .expect("Failed to run guest's `add-s32` function"),
// -100_000
// );
// assert_eq!(
// operations
// .add_u32(3_000_000, 111)
// .expect("Failed to run guest's `add-u32` function"),
// 3_000_111
// );
// assert_eq!(
// operations
// .add_float32(0.0, -0.125)
// .expect("Failed to run guest's `add-f32` function"),
// -0.125
// );
// assert_eq!(
// operations
// .add_float64(128.0, 0.25)
// .expect("Failed to run guest's `add-f64` function"),
// 128.25
// );
// }

fn load_test_module(name: &str, imports_setup: impl FnOnce(&mut Imports<()>)) -> StoreRuntime<()> {
    let engine = wasmer::Engine::default();
    let module = wasmer::Module::from_file(
        &engine,
        format!("test-modules/target/wasm32-unknown-unknown/debug/reentrant-{name}.wasm"),
    )
    .expect("Failed to load module");

    let mut imports = Imports::new(engine, ());

    imports_setup(&mut imports);

    imports
        .instantiate(&module)
        .expect("Failed to instantiate module")
}
