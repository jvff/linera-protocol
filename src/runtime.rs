use {
    crate::{FlatLayout, GuestAllocation, GuestMemory, GuestPointer, InvalidLayoutError},
    frunk::{hlist, hlist_pat, HList},
    std::{
        borrow::Cow,
        marker::PhantomData,
        num::TryFromIntError,
        ops::{Deref, DerefMut},
        string::FromUtf8Error,
    },
    thiserror::Error,
};

pub trait Runtime: Sized {
    type DataRef<'data, Data>: Deref<Target = Data> + 'data
    where
        Data: 'data;

    type DataRefMut<'data, Data>: DerefMut<Target = Data> + 'data
    where
        Data: 'data;

    type Export;
    type Memory;
}

pub trait RuntimeInstance: Sized {
    type Data;
    type Family: Runtime;

    fn data(&self) -> <Self::Family as Runtime>::DataRef<'_, Self::Data>;
    fn data_mut(&mut self) -> <Self::Family as Runtime>::DataRefMut<'_, Self::Data>;

    fn load_export(&mut self, name: &str) -> Option<<Self::Family as Runtime>::Export>;
}

pub trait RuntimeInstanceWithFunctionScope<Parameters, Results>: RuntimeInstance
where
    Parameters: FlatLayout,
    Results: FlatLayout,
{
    type Function;

    fn function_from_export(
        &mut self,
        export: <Self::Family as Runtime>::Export,
    ) -> Result<Option<Self::Function>, RuntimeError>;

    fn call(
        &mut self,
        function: &Self::Function,
        parameters: Parameters,
    ) -> Result<Results, RuntimeError>;

    fn load_function(&mut self, name: &str) -> Result<Self::Function, RuntimeError> {
        let export = self
            .load_export(name)
            .ok_or_else(|| RuntimeError::FunctionNotFound(name.to_string()))?;

        self.function_from_export(export)?
            .ok_or_else(|| RuntimeError::NotAFunction(name.to_string()))
    }
}

pub trait CabiReallocScopeAlias:
    RuntimeInstanceWithFunctionScope<HList![i32, i32, i32, i32], HList![i32]>
{
}

impl<AnyRuntimeInstance> CabiReallocScopeAlias for AnyRuntimeInstance where
    AnyRuntimeInstance: RuntimeInstanceWithFunctionScope<HList![i32, i32, i32, i32], HList![i32]>
{
}

pub trait CabiFreeScopeAlias: RuntimeInstanceWithFunctionScope<HList![i32], HList![]> {}

impl<AnyRuntimeInstance> CabiFreeScopeAlias for AnyRuntimeInstance where
    AnyRuntimeInstance: RuntimeInstanceWithFunctionScope<HList![i32], HList![]>
{
}

pub trait RuntimeInstanceWithMemoryScope: CabiReallocScopeAlias + CabiFreeScopeAlias
where
    <Self::Family as Runtime>::Memory: RuntimeMemory<Self>,
{
    fn memory_from_export(
        &self,
        export: <Self::Family as Runtime>::Export,
    ) -> Result<Option<<Self::Family as Runtime>::Memory>, RuntimeError>;

    fn memory(&mut self) -> Result<Memory<'_, Self>, RuntimeError> {
        let export = self
            .load_export("memory")
            .ok_or_else(|| RuntimeError::MissingMemory)?;

        let memory = self
            .memory_from_export(export)?
            .ok_or_else(|| RuntimeError::NotMemory)?;

        Ok(Memory {
            runtime: self,
            memory,
            cabi_realloc: None,
            cabi_free: None,
        })
    }
}

pub trait RuntimeMemory<Runtime> {
    fn read<'runtime>(
        &self,
        runtime: &'runtime Runtime,
        location: GuestPointer,
        length: u32,
    ) -> Result<Cow<'runtime, [u8]>, RuntimeError>;

    fn write(
        &mut self,
        runtime: &mut Runtime,
        location: GuestPointer,
        bytes: &[u8],
    ) -> Result<(), RuntimeError>;
}

impl<AnyRuntimeInstance> RuntimeInstance for &'_ mut AnyRuntimeInstance
where
    AnyRuntimeInstance: RuntimeInstance,
{
    type Data = AnyRuntimeInstance::Data;
    type Family = AnyRuntimeInstance::Family;

    fn data(&self) -> <Self::Family as Runtime>::DataRef<'_, Self::Data> {
        AnyRuntimeInstance::data(&**self)
    }

    fn data_mut(&mut self) -> <Self::Family as Runtime>::DataRefMut<'_, Self::Data> {
        AnyRuntimeInstance::data_mut(*self)
    }

    fn load_export(&mut self, name: &str) -> Option<<Self::Family as Runtime>::Export> {
        AnyRuntimeInstance::load_export(*self, name)
    }
}

// impl<AnyRuntimeInstance, Parameters, Results> RuntimeInstanceWithFunctionScope<Parameters, Results>
// for &'_ mut AnyRuntimeInstance
// where
// AnyRuntimeInstance: RuntimeInstanceWithFunctionScope<Parameters, Results>,
// {
// type Function = AnyRuntimeInstance::Function;

// fn function_from_export(
// &mut self,
// export: <Self::Family as Runtime>::Export,
// ) -> Result<Self::Function, RuntimeError> {
// AnyRuntimeInstance::function_from_export(*self, export)
// }

// fn call(
// &mut self,
// function: Self::Function,
// parameters: Parameters,
// ) -> Result<Results, RuntimeError> {
// AnyRuntimeInstance::call(*self, function, parameters)
// }
// }

pub trait Caller<Data>: RuntimeInstance<Data = Data> + RuntimeInstanceWithMemoryScope
where
    <Self::Family as Runtime>::Memory: RuntimeMemory<Self>,
{
}

impl<Data, AnyRuntime> Caller<Data> for AnyRuntime
where
    AnyRuntime: RuntimeInstance<Data = Data> + RuntimeInstanceWithMemoryScope,
    <AnyRuntime::Family as Runtime>::Memory: RuntimeMemory<AnyRuntime>,
{
}

pub struct Memory<'runtime, R>
where
    R: CabiReallocScopeAlias + CabiFreeScopeAlias,
{
    runtime: &'runtime mut R,
    memory: <R::Family as Runtime>::Memory,
    cabi_realloc: Option<
        <R as RuntimeInstanceWithFunctionScope<HList![i32, i32, i32, i32], HList![i32]>>::Function,
    >,
    cabi_free: Option<<R as RuntimeInstanceWithFunctionScope<HList![i32], HList![]>>::Function>,
}

impl<R> GuestMemory for Memory<'_, R>
where
    R: CabiReallocScopeAlias + CabiFreeScopeAlias,
    <R::Family as Runtime>::Memory: RuntimeMemory<R>,
{
    type Error = RuntimeError;

    fn read(&self, location: GuestPointer, length: u32) -> Result<Cow<'_, [u8]>, Self::Error> {
        self.memory.read(&*self.runtime, location, length)
    }

    fn write(&mut self, location: GuestPointer, bytes: &[u8]) -> Result<(), Self::Error> {
        self.memory.write(&mut *self.runtime, location, bytes)
    }

    fn allocate(&mut self, size: u32) -> Result<GuestAllocation<Self>, Self::Error> {
        if self.cabi_realloc.is_none() {
            self.cabi_realloc = Some(<R as RuntimeInstanceWithFunctionScope<
                HList![i32, i32, i32, i32],
                HList![i32],
            >>::load_function(self.runtime, "cabi_realloc")?);
        }

        let size = i32::try_from(size).map_err(|_| RuntimeError::AllocationTooLarge)?;

        let cabi_realloc = self
            .cabi_realloc
            .as_ref()
            .expect("`cabi_realloc` function was not loaded before it was called");

        let hlist_pat![allocation_address] =
            self.runtime.call(cabi_realloc, hlist![0, 0, 1, size])?;

        Ok(GuestAllocation {
            address: Some(GuestPointer(
                allocation_address
                    .try_into()
                    .map_err(|_| RuntimeError::AllocationFailed)?,
            )),
            _memory: PhantomData,
        })
    }

    fn deallocate(&mut self, mut allocation: GuestAllocation<Self>) -> Result<(), Self::Error> {
        if self.cabi_free.is_none() {
            self.cabi_free = Some(<R as RuntimeInstanceWithFunctionScope<
                HList![i32],
                HList![],
            >>::load_function(self.runtime, "cabi_free")?);
        }

        let address = allocation
            .address
            .take()
            .ok_or_else(|| RuntimeError::AlreadyDeallocated)?
            .0
            .try_into()
            .map_err(|_| RuntimeError::DeallocateInvalidAddress)?;

        let cabi_free = self
            .cabi_free
            .as_ref()
            .expect("`cabi_free` function was not loaded before it was called");

        self.runtime.call(cabi_free, hlist![address])?;

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[cfg(feature = "wasmer")]
    #[error(transparent)]
    Wasmer(#[from] crate::wasmer::Error),

    #[cfg(feature = "wasmtime")]
    #[error(transparent)]
    Wasmtime(#[from] crate::wasmtime::Error),

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

    #[error(transparent)]
    InvalidString(#[from] FromUtf8Error),

    #[error(transparent)]
    InvalidNumber(#[from] TryFromIntError),

    #[error(transparent)]
    InvalidLayout(#[from] InvalidLayoutError),
}

#[cfg(feature = "wasmer")]
impl From<wasmer::RuntimeError> for RuntimeError {
    fn from(error: wasmer::RuntimeError) -> Self {
        RuntimeError::Wasmer(crate::wasmer::Error::from(error))
    }
}
