use {
    super::{Entrypoint, Getters, Operations, Setters, SimpleFunction},
    witty::{
        wasmer::{Imports, StoreRuntime},
        WitExport,
    },
};

#[test]
fn simple_function() {
    let store = load_test_module("simple-function", |imports| {
        SimpleFunction::export(imports).expect("Failed to export simple function WIT interface")
    });

    Entrypoint::new(store)
        .entrypoint()
        .expect("Failed to execute test of imported simple-function");
}

#[test]
fn getters() {
    let store = load_test_module("getters", |imports| {
        Getters::export(imports).expect("Failed to export getters WIT interface")
    });

    Entrypoint::new(store)
        .entrypoint()
        .expect("Failed to execute test of imported getters");
}

#[test]
fn setters() {
    let store = load_test_module("setters", |imports| {
        Setters::export(imports).expect("Failed to export setters WIT interface")
    });

    Entrypoint::new(store)
        .entrypoint()
        .expect("Failed to execute test of imported setters");
}

#[test]
fn operations() {
    let store = load_test_module("operations", |imports| {
        Operations::export(imports).expect("Failed to export operations WIT interface")
    });

    Entrypoint::new(store)
        .entrypoint()
        .expect("Failed to execute test of imported operations");
}

fn load_test_module(name: &str, imports_setup: impl FnOnce(&mut Imports<()>)) -> StoreRuntime<()> {
    let engine = wasmer::Engine::default();
    let module = wasmer::Module::from_file(
        &engine,
        format!("test-modules/target/wasm32-unknown-unknown/debug/import-{name}.wasm"),
    )
    .expect("Failed to load module");

    let mut imports = Imports::new(engine, ());

    imports_setup(&mut imports);

    imports
        .instantiate(&module)
        .expect("Failed to instantiate module")
}
