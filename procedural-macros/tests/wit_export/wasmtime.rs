use {
    super::{Entrypoint, Getters, Operations, Setters, SimpleFunction},
    witty::{wasmtime::InstanceWithStore, WitExport},
};

#[test]
fn simple_function_wasmtime() {
    let store = load_test_module("simple-function", |linker| {
        SimpleFunction::export(linker).expect("Failed to export simple function WIT interface")
    });

    Entrypoint::new(store)
        .entrypoint()
        .expect("Failed to execute test of imported simple-function");
}

#[test]
fn getters_wasmtime() {
    let store = load_test_module("getters", |linker| {
        Getters::export(linker).expect("Failed to export getters WIT interface")
    });

    Entrypoint::new(store)
        .entrypoint()
        .expect("Failed to execute test of imported getters");
}

#[test]
fn setters_wasmtime() {
    let store = load_test_module("setters", |linker| {
        Setters::export(linker).expect("Failed to export setters WIT interface")
    });

    Entrypoint::new(store)
        .entrypoint()
        .expect("Failed to execute test of imported setters");
}

#[test]
fn operations_wasmtime() {
    let store = load_test_module("operations", |linker| {
        Operations::export(linker).expect("Failed to export operations WIT interface")
    });

    Entrypoint::new(store)
        .entrypoint()
        .expect("Failed to execute test of imported operations");
}

fn load_test_module(
    name: &str,
    linker_setup: impl FnOnce(&mut wasmtime::Linker<()>),
) -> InstanceWithStore<()> {
    let engine = wasmtime::Engine::default();
    let mut linker = wasmtime::Linker::new(&engine);

    linker_setup(&mut linker);

    let mut store = wasmtime::Store::new(&engine, ());
    let module = wasmtime::Module::from_file(
        &engine,
        format!("test-modules/target/wasm32-unknown-unknown/debug/import-{name}.wasm"),
    )
    .expect("Failed to load module");

    let instance = linker
        .instantiate(&mut store, &module)
        .expect("Failed to instantiate module");

    InstanceWithStore::new(instance, store)
}
