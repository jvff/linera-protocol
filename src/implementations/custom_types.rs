use {
    crate::{GuestMemory, GuestPointer, Layout, WitLoad, WitStore, WitType},
    frunk::{hlist, hlist_pat, HList},
};

impl WitType for GuestPointer {
    const SIZE: u32 = u32::SIZE;

    type Layout = HList![i32];
}

impl WitLoad for GuestPointer {
    fn load<Memory>(memory: &Memory, location: GuestPointer) -> Result<Self, Memory::Error>
    where
        Memory: GuestMemory,
    {
        Ok(GuestPointer(u32::load(memory, location)?))
    }

    fn lift_from<Memory>(
        hlist_pat![value]: <Self::Layout as Layout>::Flat,
        _memory: &Memory,
    ) -> Result<Self, Memory::Error>
    where
        Memory: GuestMemory,
    {
        Ok(GuestPointer(value.try_into()?))
    }
}

impl WitStore for GuestPointer {
    fn store<Memory>(
        &self,
        memory: &mut Memory,
        location: GuestPointer,
    ) -> Result<(), Memory::Error>
    where
        Memory: GuestMemory,
    {
        self.0.store(memory, location)
    }

    fn lower<Memory>(&self, _memory: &mut Memory) -> Result<Self::Layout, Memory::Error>
    where
        Memory: GuestMemory,
    {
        Ok(hlist![self.0 as i32])
    }
}
