use {
    crate::{GuestMemory, GuestPointer, Layout, WitLoad, WitStore, WitType},
    frunk::{hlist, hlist_pat, HList, HNil},
};

impl WitType for () {
    const SIZE: u32 = 0;

    type Layout = HNil;
}

impl WitLoad for () {
    fn load<Memory>(_memory: &Memory, _location: GuestPointer) -> Result<Self, Memory::Error>
    where
        Memory: GuestMemory,
    {
        Ok(())
    }

    fn lift_from<Memory>(
        HNil: <Self::Layout as Layout>::Flat,
        _memory: &Memory,
    ) -> Result<Self, Memory::Error>
    where
        Memory: GuestMemory,
    {
        Ok(())
    }
}

impl WitStore for () {
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

    fn lower<Memory>(
        &self,
        _memory: &mut Memory,
    ) -> Result<<Self::Layout as Layout>::Flat, Memory::Error>
    where
        Memory: GuestMemory,
    {
        Ok(HNil)
    }
}

impl WitType for bool {
    const SIZE: u32 = 1;

    type Layout = HList![i8];
}

impl WitLoad for bool {
    fn load<Memory>(memory: &Memory, location: GuestPointer) -> Result<Self, Memory::Error>
    where
        Memory: GuestMemory,
    {
        Ok(u8::load(memory, location)? != 0)
    }

    fn lift_from<Memory>(
        hlist_pat![value]: <Self::Layout as Layout>::Flat,
        _memory: &Memory,
    ) -> Result<Self, Memory::Error>
    where
        Memory: GuestMemory,
    {
        Ok(value != 0)
    }
}

impl WitStore for bool {
    fn store<Memory>(
        &self,
        memory: &mut Memory,
        location: GuestPointer,
    ) -> Result<(), Memory::Error>
    where
        Memory: GuestMemory,
    {
        let value: u8 = if *self { 1 } else { 0 };
        value.store(memory, location)
    }

    fn lower<Memory>(
        &self,
        _memory: &mut Memory,
    ) -> Result<<Self::Layout as Layout>::Flat, Memory::Error>
    where
        Memory: GuestMemory,
    {
        Ok(hlist![i32::from(*self)])
    }
}

impl<'t, T> WitType for &'t T
where
    T: WitType,
{
    const SIZE: u32 = T::SIZE;

    type Layout = T::Layout;
}

impl<'t, T> WitStore for &'t T
where
    T: WitStore,
{
    fn store<Memory>(
        &self,
        memory: &mut Memory,
        location: GuestPointer,
    ) -> Result<(), Memory::Error>
    where
        Memory: GuestMemory,
    {
        T::store(self, memory, location)
    }

    fn lower<Memory>(
        &self,
        memory: &mut Memory,
    ) -> Result<<Self::Layout as Layout>::Flat, Memory::Error>
    where
        Memory: GuestMemory,
    {
        T::lower(self, memory)
    }
}
