// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use linera_base::{
    data_types::{Amount, Timestamp},
    identifiers::{ApplicationId, BytecodeId, ChainId, MessageId},
};
use linera_views::batch::WriteOperation;
use std::any::Any;
use wasmtime::{Extern, Func, Linker};
use wit_bindgen_host_wasmtime_rust::{
    rt::{get_memory, RawMem},
    Endian,
};
use witty::{wit_export, wit_import, Caller, RuntimeError, WitExport};

#[wit_import]
pub trait MockSystemApi {
    fn mocked_chain_id() -> ChainId;
    fn mocked_application_id() -> ApplicationId;
    fn mocked_application_parameters() -> Vec<u8>;
    fn mocked_read_system_balance() -> Amount;
    fn mocked_read_system_timestamp() -> Timestamp;
    fn mocked_log(message: String, level: log::Level);
    fn mocked_load() -> Vec<u8>;
    fn mocked_load_and_lock() -> Option<Vec<u8>>;
    fn mocked_store_and_unlock() -> bool;
    fn mocked_lock() -> bool;
    fn mocked_unlock() -> bool;
    fn mocked_read_key_bytes(key: Vec<u8>) -> Option<Vec<u8>>;
    fn mocked_find_keys(prefix: Vec<u8>) -> Vec<Vec<u8>>;
    fn mocked_find_key_values(prefix: Vec<u8>) -> Vec<(Vec<u8>, Vec<u8>)>;
    fn mocked_write_batch(operations: Vec<WriteOperation>);
    fn mocked_try_query_application(
        application: ApplicationId,
        query: Vec<u8>,
    ) -> Result<Vec<u8>, String>;
}

pub struct SystemApiProxy;

#[wit_export]
impl SystemApiProxy {
    pub fn chain_id(caller: impl Caller<()>) -> Result<ChainId, RuntimeError> {
        MockSystemApi::new(caller).mocked_chain_id()
    }

    pub fn application_id(caller: impl Caller<()>) -> Result<ApplicationId, RuntimeError> {
        MockSystemApi::new(caller).mocked_application_id()
    }

    pub fn application_parameters(caller: impl Caller<()>) -> Result<Vec<u8>, RuntimeError> {
        MockSystemApi::new(caller).mocked_application_parameters()
    }

    pub fn read_system_balance(caller: impl Caller<()>) -> Result<Amount, RuntimeError> {
        MockSystemApi::new(caller).mocked_read_sytem_balance()
    }

    pub fn read_system_timestamp(caller: impl Caller<()>) -> Result<Timestamp, RuntimeError> {
        MockSystemApi::new(caller).mocked_read_sytem_timestamp()
    }

    pub fn log(
        caller: impl Caller<()>,
        message: String,
        level: log::Level,
    ) -> Result<(), RuntimeError> {
        MockSystemApi::new(caller).mocked_log(message, level)
    }

    pub fn load(caller: impl Caller<()>) -> Result<Vec<u8>, RuntimeError> {
        MockSystemApi::new(caller).mocked_load()
    }

    pub fn load_and_lock(caller: impl Caller<()>) -> Result<Option<Vec<u8>>, RuntimeError> {
        MockSystemApi::new(caller).mocked_load_and_lock()
    }

    pub fn store_and_unlock(caller: impl Caller<()>) -> Result<bool, RuntimeError> {
        MockSystemApi::new(caller).mocked_store_and_unlock()
    }

    pub fn lock_new(caller: impl Caller<()>) -> Result<i32, RuntimeError> {
        MockSystemApi::new(caller).mocked_store_and_unlock()
    }
}

/// A map of resources allocated on the host side.
#[derive(Default)]
pub struct Resources(Vec<Box<dyn Any + Send + 'static>>);

impl Resources {
    /// Adds a resource to the map, returning its handle.
    pub fn insert(&mut self, value: impl Any + Send + 'static) -> i32 {
        let handle = self.0.len().try_into().expect("Resources map overflow");

        self.0.push(Box::new(value));

        handle
    }

    /// Returns an immutable reference to a resource referenced by the provided `handle`.
    pub fn get<T: 'static>(&self, handle: i32) -> &T {
        self.0[usize::try_from(handle).expect("Invalid handle")]
            .downcast_ref()
            .expect("Incorrect handle type")
    }
}

/// A resource representing a query.
#[derive(Clone)]
struct Query {
    application_id: ApplicationId,
    query: Vec<u8>,
}

/// Adds the mock system APIs to the linker, so that they are available to guest WebAsembly
/// modules.
///
/// The system APIs are proxied back to the guest module, to be handled by the functions exported
/// from `linera_sdk::test::unit`.
pub fn add_to_linker(linker: &mut Linker<Resources>) -> Result<()> {
    Ok(SystemApiProxy::export(linker)?)
}
