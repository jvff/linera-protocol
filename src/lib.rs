mod export_function;
mod export_function_interface;
mod flat_type;
mod implementations;
mod import_function_interface;
mod join_flat_layouts;
mod join_flat_types;
mod layout;
mod layout_element;
mod maybe_flat_type;
mod results;
mod runtime;
mod simple_type;
mod util;

#[cfg(feature = "wasmer")]
pub mod wasmer;
#[cfg(feature = "wasmtime")]
pub mod wasmtime;

use {
    std::{
        borrow::Cow, marker::PhantomData, num::TryFromIntError, ops::Deref, string::FromUtf8Error,
    },
    thiserror::Error,
};

pub use {
    self::{
        export_function::ExportFunction,
        export_function_interface::ExportFunctionInterface,
        import_function_interface::ImportFunctionInterface,
        join_flat_layouts::{JoinFlatLayouts, SplitFlatLayouts},
        layout::{FlatLayout, Layout},
        runtime::{
            Caller, Runtime, RuntimeError, RuntimeInstance, RuntimeInstanceWithFunctionScope,
            RuntimeInstanceWithMemoryScope, RuntimeMemory,
        },
        util::{ConcatDisplay, Merge, RepeatToFill, Split, Unmerge},
    },
    frunk::{hlist, hlist_pat, HCons, HList, HNil},
};

#[cfg(feature = "wasmtime")]
pub use self::wasmer::Wasmer;
#[cfg(feature = "wasmtime")]
pub use {
    self::wasmtime::{Wasmtime, WasmtimeRuntime},
    anyhow,
};
#[cfg(feature = "macros")]
pub use {
    try_insert_ext::OptionInsertExt,
    witty_macros::{wit_export, wit_import, WitLoad, WitStore, WitType},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GuestPointer(u32);

impl GuestPointer {
    pub fn after<T: WitType>(&self) -> Self {
        GuestPointer(self.0 + T::SIZE)
    }

    pub fn after_padding_for<T: WitType>(&self) -> Self {
        let padding = (-(self.0 as i32) & (<T::Layout as Layout>::ALIGNMENT as i32 - 1)) as u32;

        GuestPointer(self.0 + padding)
    }

    pub fn index<T: WitType>(&self, index: u32) -> Self {
        let padding = (-(T::SIZE as i32) & (<T::Layout as Layout>::ALIGNMENT as i32 - 1)) as u32;

        GuestPointer(self.0 + index * (T::SIZE + padding))
    }
}

pub struct GuestAllocation<Memory> {
    address: Option<GuestPointer>,
    _memory: PhantomData<Memory>,
}

impl<Memory> Deref for GuestAllocation<Memory> {
    type Target = GuestPointer;

    fn deref(&self) -> &Self::Target {
        self.address.as_ref().expect("Usage of deallocated address")
    }
}

pub trait GuestMemory: Sized {
    type Error: From<InvalidLayoutError> + From<TryFromIntError> + From<FromUtf8Error>;

    fn read(&self, location: GuestPointer, length: u32) -> Result<Cow<'_, [u8]>, Self::Error>;
    fn write(&mut self, location: GuestPointer, bytes: &[u8]) -> Result<(), Self::Error>;
    fn allocate(&mut self, size: u32) -> Result<GuestAllocation<Self>, Self::Error>;
    fn deallocate(&mut self, allocation: GuestAllocation<Self>) -> Result<(), Self::Error>;
}

pub trait WitType: Sized {
    const SIZE: u32;

    type Layout: Layout;
}

pub trait WitLoad: WitType {
    fn load<Memory>(memory: &Memory, location: GuestPointer) -> Result<Self, Memory::Error>
    where
        Memory: GuestMemory;

    fn lift_from<Memory>(
        flat_layout: <Self::Layout as Layout>::Flat,
        memory: &Memory,
    ) -> Result<Self, Memory::Error>
    where
        Memory: GuestMemory;
}

pub trait WitStore: WitType {
    fn store<Memory>(
        &self,
        memory: &mut Memory,
        location: GuestPointer,
    ) -> Result<(), Memory::Error>
    where
        Memory: GuestMemory;

    fn lower<Memory>(
        &self,
        memory: &mut Memory,
    ) -> Result<<Self::Layout as Layout>::Flat, Memory::Error>
    where
        Memory: GuestMemory;
}

pub trait WitExport<Runtime> {
    fn export(runtime: &mut Runtime) -> Result<(), RuntimeError>;
}

#[derive(Debug, Error)]
#[error("Unexpected layout in memory")]
pub struct InvalidLayoutError;

// #[derive(Debug, Error)]
// pub enum RuntimeError {
// #[cfg(feature = "wasmer")]
// #[error(transparent)]
// Wasmer(#[from] wasmer::Error),

// #[cfg(feature = "wasmtime")]
// #[error(transparent)]
// Wasmtime(#[from] wasmtime::Error),
// }
