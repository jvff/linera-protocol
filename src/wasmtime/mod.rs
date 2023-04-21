mod export_function;
mod parameters;
mod results;

pub use {
    self::{parameters::WasmtimeParameters, results::WasmtimeResults},
    wasmtime::{Caller, Error, Linker},
};
use {
    crate::{
        import_function_interface::{FlatParameters, FlatResults},
        runtime::{
            Runtime, RuntimeError, RuntimeInstance, RuntimeInstanceWithFunctionScope,
            RuntimeInstanceWithMemoryScope, RuntimeMemory,
        },
        FlatLayout, GuestAllocation, GuestMemory, GuestPointer, Layout,
    },
    anyhow::{anyhow, bail},
    std::{borrow::Cow, marker::PhantomData},
    wasmtime::{
        AsContext, AsContextMut, Extern, Instance, Store, StoreContext, StoreContextMut, TypedFunc,
        WasmParams, WasmResults,
    },
};

pub struct Wasmtime;

impl Runtime for Wasmtime {
    type DataRef<'data, Data> = &'data Data where Data: 'data;
    type DataRefMut<'data, Data> = &'data mut Data where Data: 'data;

    type Export = Extern;
    type Memory = wasmtime::Memory;
}

pub type Function<Parameters, Results> = TypedFunc<
    <<<Parameters as Layout>::Flat as FlatParameters>::Input as WasmtimeParameters>::Parameters,
    <<<Results as Layout>::Flat as FlatResults>::Output as WasmtimeResults>::Results,
>;

pub trait WasmtimeRuntime: AsContextMut + Sized {
    fn data(&self) -> &Self::Data;
    fn data_mut(&mut self) -> &mut Self::Data;

    fn load_export(&mut self, name: &str) -> Option<Extern>;

    fn load_function<Parameters, Results>(
        &mut self,
        name: &str,
    ) -> Result<TypedFunc<Parameters, Results>, Error>
    where
        Parameters: WasmParams,
        Results: WasmResults,
    {
        let export = self.load_export(name).ok_or_else(|| {
            anyhow!("Function `{name}` could not be found in the module's exports")
        })?;

        match export {
            Extern::Func(function) => function.typed(self.as_context()),
            _ => bail!("Export `{name}` is not a function"),
        }
    }

    fn memory(&mut self) -> Result<Memory<'_, Self>, Error> {
        let memory = match self.load_export("memory") {
            Some(Extern::Memory(memory)) => memory,
            Some(_) => bail!("`memory` export is not WebAssembly memory"),
            None => bail!("Missing `memory` export in guest WebAssembly module"),
        };

        Ok(Memory {
            runtime: self,
            memory,
            cabi_realloc: None,
            cabi_free: None,
        })
    }
}

impl<Data> WasmtimeRuntime for Caller<'_, Data> {
    fn data(&self) -> &Self::Data {
        Caller::data(self)
    }

    fn data_mut(&mut self) -> &mut Self::Data {
        Caller::data_mut(self)
    }

    fn load_export(&mut self, name: &str) -> Option<Extern> {
        Caller::get_export(self, name)
    }
}

impl<Data> RuntimeInstance for Caller<'_, Data> {
    type Data = Data;
    type Family = Wasmtime;

    fn data(&self) -> &Self::Data {
        Caller::data(self)
    }

    fn data_mut(&mut self) -> &mut Self::Data {
        Caller::data_mut(self)
    }

    fn load_export(&mut self, name: &str) -> Option<Extern> {
        Caller::get_export(self, name)
    }
}

impl<Data, Parameters, Results> RuntimeInstanceWithFunctionScope<Parameters, Results>
    for Caller<'_, Data>
where
    Parameters: FlatLayout + WasmtimeParameters,
    Results: FlatLayout + WasmtimeResults,
{
    type Function = TypedFunc<
        <Parameters as WasmtimeParameters>::Parameters,
        <Results as WasmtimeResults>::Results,
    >;

    fn function_from_export(
        &mut self,
        export: <Self::Family as Runtime>::Export,
    ) -> Result<Option<Self::Function>, RuntimeError> {
        Ok(match export {
            Extern::Func(function) => Some(function.typed(self.as_context())?),
            _ => None,
        })
    }

    fn call(
        &mut self,
        function: &Self::Function,
        parameters: Parameters,
    ) -> Result<Results, RuntimeError> {
        let results = function.call(self.as_context_mut(), parameters.into_wasmtime())?;

        Ok(Results::from_wasmtime(results))
    }
}

impl<Data, Parameters, Results> RuntimeInstanceWithFunctionScope<Parameters, Results>
    for &mut Caller<'_, Data>
where
    Parameters: FlatLayout + WasmtimeParameters,
    Results: FlatLayout + WasmtimeResults,
{
    type Function = TypedFunc<
        <Parameters as WasmtimeParameters>::Parameters,
        <Results as WasmtimeResults>::Results,
    >;

    fn function_from_export(
        &mut self,
        export: <Self::Family as Runtime>::Export,
    ) -> Result<Option<Self::Function>, RuntimeError> {
        Ok(match export {
            Extern::Func(function) => Some(function.typed(self.as_context())?),
            _ => None,
        })
    }

    fn call(
        &mut self,
        function: &Self::Function,
        parameters: Parameters,
    ) -> Result<Results, RuntimeError> {
        let results = function.call(self.as_context_mut(), parameters.into_wasmtime())?;

        Ok(Results::from_wasmtime(results))
    }
}

impl<Data> RuntimeInstanceWithMemoryScope for Caller<'_, Data> {
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

impl<Data> RuntimeInstanceWithMemoryScope for &mut Caller<'_, Data> {
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

impl<AllWasmtimeRuntimes> WasmtimeRuntime for &mut AllWasmtimeRuntimes
where
    AllWasmtimeRuntimes: WasmtimeRuntime,
{
    fn data(&self) -> &Self::Data {
        WasmtimeRuntime::data(&**self)
    }

    fn data_mut(&mut self) -> &mut Self::Data {
        WasmtimeRuntime::data_mut(&mut **self)
    }

    fn load_export(&mut self, name: &str) -> Option<Extern> {
        WasmtimeRuntime::load_export(&mut **self, name)
    }
}

pub struct InstanceWithStore<T> {
    instance: Instance,
    store: Store<T>,
}

impl<T> InstanceWithStore<T> {
    pub fn new(instance: Instance, store: Store<T>) -> Self {
        InstanceWithStore { instance, store }
    }
}

impl<T> AsContext for InstanceWithStore<T> {
    type Data = T;

    fn as_context(&self) -> StoreContext<T> {
        self.store.as_context()
    }
}

impl<T> AsContextMut for InstanceWithStore<T> {
    fn as_context_mut(&mut self) -> StoreContextMut<T> {
        self.store.as_context_mut()
    }
}

impl<T> WasmtimeRuntime for InstanceWithStore<T> {
    fn data(&self) -> &Self::Data {
        self.store.data()
    }

    fn data_mut(&mut self) -> &mut Self::Data {
        self.store.data_mut()
    }

    fn load_export(&mut self, name: &str) -> Option<Extern> {
        self.instance.get_export(&mut self.store, name)
    }
}

impl<Data> RuntimeInstance for InstanceWithStore<Data> {
    type Data = Data;
    type Family = Wasmtime;

    fn data(&self) -> &Self::Data {
        self.store.data()
    }

    fn data_mut(&mut self) -> &mut Self::Data {
        self.store.data_mut()
    }

    fn load_export(&mut self, name: &str) -> Option<Extern> {
        self.instance.get_export(&mut self.store, name)
    }
}

impl<Data, Parameters, Results> RuntimeInstanceWithFunctionScope<Parameters, Results>
    for InstanceWithStore<Data>
where
    Parameters: FlatLayout + WasmtimeParameters,
    Results: FlatLayout + WasmtimeResults,
{
    type Function = TypedFunc<
        <Parameters as WasmtimeParameters>::Parameters,
        <Results as WasmtimeResults>::Results,
    >;

    fn function_from_export(
        &mut self,
        export: <Self::Family as Runtime>::Export,
    ) -> Result<Option<Self::Function>, RuntimeError> {
        Ok(match export {
            Extern::Func(function) => Some(function.typed(self.as_context())?),
            _ => None,
        })
    }

    fn call(
        &mut self,
        function: &Self::Function,
        parameters: Parameters,
    ) -> Result<Results, RuntimeError> {
        let results = function.call(self.as_context_mut(), parameters.into_wasmtime())?;

        Ok(Results::from_wasmtime(results))
    }
}

impl<Data> RuntimeInstanceWithMemoryScope for InstanceWithStore<Data> {
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

impl<Runtime> RuntimeMemory<Runtime> for wasmtime::Memory
where
    Runtime: WasmtimeRuntime,
{
    fn read<'runtime>(
        &self,
        runtime: &'runtime Runtime,
        location: GuestPointer,
        length: u32,
    ) -> Result<Cow<'runtime, [u8]>, RuntimeError> {
        let start = location.0 as usize;
        let end = start + length as usize;

        Ok(Cow::Borrowed(&self.data(runtime)[start..end]))
    }

    fn write(
        &mut self,
        runtime: &mut Runtime,
        location: GuestPointer,
        bytes: &[u8],
    ) -> Result<(), RuntimeError> {
        let start = location.0 as usize;
        let end = start + bytes.len();

        self.data_mut(runtime)[start..end].copy_from_slice(bytes);

        Ok(())
    }
}

pub struct Memory<'runtime, Runtime> {
    runtime: &'runtime mut Runtime,
    memory: wasmtime::Memory,
    cabi_realloc: Option<TypedFunc<(i32, i32, i32, i32), i32>>,
    cabi_free: Option<TypedFunc<(i32,), ()>>,
}

impl<Runtime> GuestMemory for Memory<'_, Runtime>
where
    Runtime: WasmtimeRuntime,
{
    type Error = anyhow::Error;

    fn read(&self, location: GuestPointer, length: u32) -> Result<Cow<'_, [u8]>, Self::Error> {
        let start = location.0 as usize;
        let end = start + length as usize;

        Ok(Cow::Borrowed(&self.memory.data(&*self.runtime)[start..end]))
    }

    fn write(&mut self, location: GuestPointer, bytes: &[u8]) -> Result<(), Self::Error> {
        let start = location.0 as usize;
        let end = start + bytes.len();

        self.memory.data_mut(&mut *self.runtime)[start..end].copy_from_slice(bytes);

        Ok(())
    }

    fn allocate(&mut self, size: u32) -> Result<GuestAllocation<Self>, Self::Error> {
        if self.cabi_realloc.is_none() {
            self.cabi_realloc = Some(self.runtime.load_function("cabi_realloc")?);
        }

        let size =
            i32::try_from(size).map_err(|_| anyhow!("Requested allocation size is too large"))?;

        let allocation_address = self
            .cabi_realloc
            .as_mut()
            .expect("`cabi_realloc` function was not loaded before it was called")
            .call(&mut *self.runtime, (0, 0, 1, size))?;

        Ok(GuestAllocation {
            address: Some(GuestPointer(
                allocation_address
                    .try_into()
                    .map_err(|_| anyhow!("Memory allocation failed"))?,
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
            .ok_or_else(|| anyhow!("Attempt to deallocate an already deallocated address"))?
            .0
            .try_into()
            .map_err(|_| anyhow!("Attempt to deallocate an invalid address"))?;

        self.cabi_free
            .as_mut()
            .expect("`cabi_realloc` function was not loaded before it was called")
            .call(&mut *self.runtime, (address,))?;

        Ok(())
    }
}
