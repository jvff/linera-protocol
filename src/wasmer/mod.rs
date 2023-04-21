mod call_with_flat_parameters;
mod export_function;
mod function;
mod parameters;
mod results;

pub use {
    self::{
        call_with_flat_parameters::CallWithFlatParameters, parameters::WasmerParameters,
        results::WasmerResults,
    },
    std::{
        marker::PhantomData,
        sync::{Arc, Mutex, MutexGuard},
    },
    wasmer::FunctionEnvMut,
};
use {
    crate::{
        import_function_interface::{FlatParameters, FlatResults},
        runtime::{
            Runtime, RuntimeError, RuntimeInstance, RuntimeInstanceWithMemoryScope, RuntimeMemory,
        },
        GuestAllocation, GuestMemory, GuestPointer, InvalidLayoutError, Layout,
    },
    std::{borrow::Cow, num::TryFromIntError, string::FromUtf8Error},
    thiserror::Error,
    wasmer::{
        AsStoreMut, AsStoreRef, Engine, Extern, FunctionEnv, Instance, InstantiationError, Module,
        Store, StoreMut, StoreRef, TypedFunction, WasmTypeList,
    },
    wasmer_vm::StoreObjects,
};

pub struct Wasmer;

impl Runtime for Wasmer {
    type DataRef<'data, Data> = MutexGuard<'data, Data> where Data: 'data;
    type DataRefMut<'data, Data> = MutexGuard<'data, Data> where Data: 'data;

    type Export = Extern;
    type Memory = wasmer::Memory;
}

pub type Function<Parameters, Results> = TypedFunction<
    <<<Parameters as Layout>::Flat as FlatParameters>::Input as WasmerParameters>::ImportParameters,
    <<<Results as Layout>::Flat as FlatResults>::Output as WasmerResults>::Results,
>;

pub trait WasmerRuntime: AsStoreMut + Sized {
    type Data;

    fn data(&self) -> MutexGuard<'_, Self::Data>;
    fn data_mut(&mut self) -> MutexGuard<'_, Self::Data>;
    fn load_export(&mut self, name: &str) -> Option<Extern>;

    fn load_function<Parameters, Results>(
        &mut self,
        name: &str,
    ) -> Result<TypedFunction<Parameters, Results>, Error>
    where
        Parameters: WasmTypeList,
        Results: WasmTypeList,
    {
        let export = self
            .load_export(name)
            .ok_or_else(|| Error::FunctionNotFound(name.to_string()))?;

        match export {
            Extern::Function(function) => Ok(function.typed(&self.as_store_ref())?),
            _ => Err(Error::NotAFunction(name.to_string())),
        }
    }

    fn memory(&mut self) -> Result<Memory<'_, Self>, Error> {
        let memory = match self.load_export("memory") {
            Some(Extern::Memory(memory)) => memory,
            Some(_) => return Err(Error::NotMemory),
            None => return Err(Error::MissingMemory),
        };

        Ok(Memory {
            runtime: self,
            memory,
            cabi_realloc: None,
            cabi_free: None,
        })
    }
}

impl<Data> WasmerRuntime for FunctionEnvMut<'_, InstanceAndData<Data>>
where
    Data: Send + 'static,
{
    type Data = Data;

    fn data(&self) -> MutexGuard<'_, Self::Data> {
        wasmer::FunctionEnvMut::data(self).data()
    }

    fn data_mut(&mut self) -> MutexGuard<'_, Self::Data> {
        wasmer::FunctionEnvMut::data(self).data()
    }

    fn load_export(&mut self, name: &str) -> Option<Extern> {
        wasmer::FunctionEnvMut::data_mut(self).load_export(name)
    }
}

impl<Data> RuntimeInstance for FunctionEnvMut<'_, InstanceAndData<Data>>
where
    Data: Send + 'static,
{
    type Data = Data;
    type Family = Wasmer;

    fn data(&self) -> MutexGuard<'_, Data> {
        wasmer::FunctionEnvMut::data(self).data()
    }

    fn data_mut(&mut self) -> MutexGuard<'_, Data> {
        wasmer::FunctionEnvMut::data(self).data()
    }

    fn load_export(&mut self, name: &str) -> Option<Extern> {
        wasmer::FunctionEnvMut::data_mut(self).load_export(name)
    }
}

impl<Data> RuntimeInstanceWithMemoryScope for FunctionEnvMut<'_, InstanceAndData<Data>>
where
    Data: Send + 'static,
{
    fn memory_from_export(
        &self,
        export: <Self::Family as Runtime>::Export,
    ) -> Result<Option<<Self::Family as Runtime>::Memory>, RuntimeError> {
        Ok(match export {
            Extern::Memory(memory) => Some(memory),
            _ => None,
        })
    }
}

impl<Data> RuntimeInstanceWithMemoryScope for &mut FunctionEnvMut<'_, InstanceAndData<Data>>
where
    Data: Send + 'static,
{
    fn memory_from_export(
        &self,
        export: <Self::Family as Runtime>::Export,
    ) -> Result<Option<<Self::Family as Runtime>::Memory>, RuntimeError> {
        Ok(match export {
            Extern::Memory(memory) => Some(memory),
            _ => None,
        })
    }
}

impl<AllWasmerRuntimes> WasmerRuntime for &mut AllWasmerRuntimes
where
    AllWasmerRuntimes: WasmerRuntime,
{
    type Data = AllWasmerRuntimes::Data;

    fn data(&self) -> MutexGuard<'_, Self::Data> {
        WasmerRuntime::data(&**self)
    }

    fn data_mut(&mut self) -> MutexGuard<'_, Self::Data> {
        WasmerRuntime::data(&**self)
    }

    fn load_export(&mut self, name: &str) -> Option<Extern> {
        WasmerRuntime::load_export(&mut **self, name)
    }
}

pub struct StoreRuntime<Data> {
    store: Store,
    instance_and_data: InstanceAndData<Data>,
}

impl<Data> StoreRuntime<Data> {
    pub fn new(store: Store, instance_and_data: InstanceAndData<Data>) -> Self {
        StoreRuntime {
            store,
            instance_and_data,
        }
    }
}

impl<Data> AsStoreRef for StoreRuntime<Data> {
    fn as_store_ref(&self) -> StoreRef<'_> {
        self.store.as_store_ref()
    }
}

impl<Data> AsStoreMut for StoreRuntime<Data> {
    fn as_store_mut(&mut self) -> StoreMut<'_> {
        self.store.as_store_mut()
    }

    fn objects_mut(&mut self) -> &mut StoreObjects {
        self.store.objects_mut()
    }
}

impl<Data> WasmerRuntime for StoreRuntime<Data> {
    type Data = Data;

    fn data(&self) -> MutexGuard<'_, Self::Data> {
        self.instance_and_data.data()
    }

    fn data_mut(&mut self) -> MutexGuard<'_, Self::Data> {
        self.instance_and_data.data()
    }

    fn load_export(&mut self, name: &str) -> Option<Extern> {
        self.instance_and_data.load_export(name)
    }
}

impl<Data> RuntimeInstance for StoreRuntime<Data>
where
    Data: Send + 'static,
{
    type Data = Data;
    type Family = Wasmer;

    fn data(&self) -> MutexGuard<'_, Data> {
        self.instance_and_data.data()
    }

    fn data_mut(&mut self) -> MutexGuard<'_, Data> {
        self.instance_and_data.data()
    }

    fn load_export(&mut self, name: &str) -> Option<Extern> {
        self.instance_and_data.load_export(name)
    }
}

impl<Data> RuntimeInstanceWithMemoryScope for StoreRuntime<Data>
where
    Data: Send + 'static,
{
    fn memory_from_export(
        &self,
        export: <Self::Family as Runtime>::Export,
    ) -> Result<Option<<Self::Family as Runtime>::Memory>, RuntimeError> {
        Ok(match export {
            Extern::Memory(memory) => Some(memory),
            _ => None,
        })
    }
}

pub struct Imports<Data> {
    store: Store,
    imports: wasmer::Imports,
    environment: InstanceAndData<Data>,
}

impl<Data> Imports<Data>
where
    Data: Send + 'static,
{
    pub fn new(engine: Engine, data: Data) -> Self {
        Imports {
            store: Store::new(engine),
            imports: wasmer::Imports::default(),
            environment: InstanceAndData::new(None, data),
        }
    }

    pub fn store(&self) -> &Store {
        &self.store
    }

    pub fn environment(&mut self) -> FunctionEnv<InstanceAndData<Data>> {
        FunctionEnv::new(&mut self.store, self.environment.clone())
    }

    pub fn define(&mut self, namespace: &str, name: &str, value: impl Into<Extern>) {
        self.imports.define(namespace, name, value);
    }

    #[allow(clippy::result_large_err)]
    pub fn instantiate(
        mut self,
        module: &Module,
    ) -> Result<StoreRuntime<Data>, InstantiationError> {
        let instance = Instance::new(&mut self.store, module, &self.imports)?;

        *self
            .environment
            .instance
            .try_lock()
            .expect("Unexpected usage of instance before it was initialized") = Some(instance);

        Ok(StoreRuntime::new(self.store, self.environment))
    }
}

impl<Data> AsStoreRef for Imports<Data> {
    fn as_store_ref(&self) -> StoreRef<'_> {
        self.store.as_store_ref()
    }
}

impl<Data> AsStoreMut for Imports<Data> {
    fn as_store_mut(&mut self) -> StoreMut<'_> {
        self.store.as_store_mut()
    }

    fn objects_mut(&mut self) -> &mut StoreObjects {
        self.store.objects_mut()
    }
}

pub struct InstanceAndData<Data> {
    instance: Arc<Mutex<Option<Instance>>>,
    data: Arc<Mutex<Data>>,
}

impl<Data> InstanceAndData<Data> {
    fn new(instance: impl Into<Option<Instance>>, data: Data) -> Self {
        InstanceAndData {
            instance: Arc::new(Mutex::new(instance.into())),
            data: Arc::new(Mutex::new(data)),
        }
    }

    fn data(&self) -> MutexGuard<'_, Data> {
        self.data
            .try_lock()
            .expect("Unexpected reentrant access to data")
    }

    fn load_export(&mut self, name: &str) -> Option<Extern> {
        self.instance
            .try_lock()
            .expect("Unexpected reentrant access to data")
            .as_mut()
            .expect("Unexpected attempt to load an export before instance is created")
            .exports
            .get_extern(name)
            .cloned()
    }
}

impl<Data> Clone for InstanceAndData<Data> {
    fn clone(&self) -> Self {
        InstanceAndData {
            instance: self.instance.clone(),
            data: self.data.clone(),
        }
    }
}

impl<Runtime> RuntimeMemory<Runtime> for wasmer::Memory
where
    Runtime: WasmerRuntime,
{
    fn read<'runtime>(
        &self,
        runtime: &'runtime Runtime,
        location: GuestPointer,
        length: u32,
    ) -> Result<Cow<'runtime, [u8]>, RuntimeError> {
        let mut buffer = vec![0u8; length as usize];
        let start = location.0 as u64;

        self.view(runtime)
            .read(start, &mut buffer)
            .map_err(Error::from)?;

        Ok(Cow::Owned(buffer))
    }

    fn write(
        &mut self,
        runtime: &mut Runtime,
        location: GuestPointer,
        bytes: &[u8],
    ) -> Result<(), RuntimeError> {
        let start = location.0 as u64;

        self.view(&*runtime)
            .write(start, bytes)
            .map_err(Error::from)?;

        Ok(())
    }
}

pub struct Memory<'runtime, Runtime> {
    runtime: &'runtime mut Runtime,
    memory: wasmer::Memory,
    cabi_realloc: Option<TypedFunction<(i32, i32, i32, i32), i32>>,
    cabi_free: Option<TypedFunction<i32, ()>>,
}

impl<Runtime> GuestMemory for Memory<'_, Runtime>
where
    Runtime: WasmerRuntime,
{
    type Error = Error;

    fn read(&self, location: GuestPointer, length: u32) -> Result<Cow<'_, [u8]>, Self::Error> {
        let mut buffer = vec![0u8; length as usize];
        let start = location.0 as u64;

        self.memory.view(&*self.runtime).read(start, &mut buffer)?;

        Ok(Cow::Owned(buffer))
    }

    fn write(&mut self, location: GuestPointer, bytes: &[u8]) -> Result<(), Self::Error> {
        let start = location.0 as u64;

        self.memory.view(&*self.runtime).write(start, bytes)?;

        Ok(())
    }

    fn allocate(&mut self, size: u32) -> Result<GuestAllocation<Self>, Self::Error> {
        if self.cabi_realloc.is_none() {
            self.cabi_realloc = Some(self.runtime.load_function("cabi_realloc")?);
        }

        let size = i32::try_from(size).map_err(|_| Error::AllocationTooLarge)?;

        let allocation_address = self
            .cabi_realloc
            .as_mut()
            .expect("`cabi_realloc` function was not loaded before it was called")
            .call(&mut *self.runtime, 0, 0, 1, size)?;

        Ok(GuestAllocation {
            address: Some(GuestPointer(
                allocation_address
                    .try_into()
                    .map_err(|_| Error::AllocationTooLarge)?,
            )),
            _memory: PhantomData,
        })
    }

    fn deallocate(&mut self, mut allocation: GuestAllocation<Self>) -> Result<(), Self::Error> {
        if self.cabi_free.is_none() {
            self.cabi_free = Some(self.runtime.load_function("cabi_free")?);
        }

        let address = allocation
            .address
            .take()
            .ok_or_else(|| Error::AlreadyDeallocated)?
            .0
            .try_into()
            .map_err(|_| Error::DeallocateInvalidAddress)?;

        self.cabi_free
            .as_mut()
            .expect("`cabi_realloc` function was not loaded before it was called")
            .call(&mut *self.runtime, address)?;

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Function `{_0}` could not be found in the module's exports")]
    FunctionNotFound(String),

    #[error("Export `{_0}` is not a function")]
    NotAFunction(String),

    #[error("Missing `memory` export in guest WebAssembly module")]
    MissingMemory,

    #[error("`memory` export is not WebAssembly memory")]
    NotMemory,

    #[error("Requested allocation size is too large")]
    AllocationTooLarge,

    #[error("Memory allocation failed")]
    AllocationFailed,

    #[error("Attempt to deallocate an already deallocated address")]
    AlreadyDeallocated,

    #[error("Attempt to deallocate an invalid address")]
    DeallocateInvalidAddress,

    #[error("Reentrant call to guest failed")]
    FromReentrantCall(#[from] Box<crate::RuntimeError>),

    #[error(transparent)]
    Memory(#[from] wasmer::MemoryError),

    #[error(transparent)]
    MemoryAccess(#[from] wasmer::MemoryAccessError),

    #[error(transparent)]
    Runtime(#[from] wasmer::RuntimeError),

    #[error(transparent)]
    InvalidString(#[from] FromUtf8Error),

    #[error(transparent)]
    InvalidNumber(#[from] TryFromIntError),

    #[error(transparent)]
    InvalidLayout(#[from] InvalidLayoutError),
}

impl From<crate::RuntimeError> for Error {
    fn from(reentrant_error: crate::RuntimeError) -> Self {
        Error::FromReentrantCall(Box::new(reentrant_error))
    }
}
