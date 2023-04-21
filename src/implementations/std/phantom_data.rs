use {
    crate::{GuestMemory, GuestPointer, Layout, WitLoad, WitStore, WitType},
    frunk::{hlist, HList},
    std::marker::PhantomData,
};

impl<T> WitType for PhantomData<T> {
    const SIZE: u32 = 0;

    type Layout = HList![];
}

impl<T> WitLoad for PhantomData<T> {
    fn load<Memory>(_memory: &Memory, _location: GuestPointer) -> Result<Self, Memory::Error>
    where
        Memory: GuestMemory,
    {
        Ok(PhantomData)
    }

    fn lift_from<Memory>(
        _flat_layout: <Self::Layout as Layout>::Flat,
        _memory: &Memory,
    ) -> Result<Self, Memory::Error>
    where
        Memory: GuestMemory,
    {
        Ok(PhantomData)
    }
}

impl<T> WitStore for PhantomData<T> {
    fn store<Memory>(
        &self,
        _memory: &mut Memory,
        _location: GuestPointer,
    ) -> Result<(), Memory::Error>
    where
        Memory: GuestMemory,
    {
        Ok(())
    }

    fn lower<Memory>(&self, _memory: &mut Memory) -> Result<Self::Layout, Memory::Error>
    where
        Memory: GuestMemory,
    {
        Ok(hlist![])
    }
}
