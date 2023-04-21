use {
    crate::{GuestMemory, GuestPointer, Layout, WitLoad, WitStore, WitType},
    frunk::{hlist, hlist_pat, HList},
};

impl WitType for String {
    const SIZE: u32 = 8;

    type Layout = HList![i32, i32];
}

impl WitLoad for String {
    fn load<Memory>(memory: &Memory, location: GuestPointer) -> Result<Self, Memory::Error>
    where
        Memory: GuestMemory,
    {
        let address = GuestPointer::load(memory, location)?;
        let length = u32::load(memory, location.after::<GuestPointer>())?;

        let bytes = memory.read(address, length)?.to_vec();

        Ok(String::from_utf8(bytes)?)
    }

    fn lift_from<Memory>(
        hlist_pat![address, length]: <Self::Layout as Layout>::Flat,
        memory: &Memory,
    ) -> Result<Self, Memory::Error>
    where
        Memory: GuestMemory,
    {
        let address = GuestPointer(address.try_into()?);
        let length = length as u32;

        let bytes = memory.read(address, length)?.to_vec();

        Ok(String::from_utf8(bytes)?)
    }
}

impl WitStore for String {
    fn store<Memory>(
        &self,
        memory: &mut Memory,
        location: GuestPointer,
    ) -> Result<(), Memory::Error>
    where
        Memory: GuestMemory,
    {
        let length = u32::try_from(self.len())?;
        let destination = memory.allocate(length)?;

        destination.store(memory, location)?;
        length.store(memory, location.after::<GuestPointer>())?;

        memory.write(*destination, self.as_bytes())
    }

    fn lower<Memory>(&self, memory: &mut Memory) -> Result<Self::Layout, Memory::Error>
    where
        Memory: GuestMemory,
    {
        let length = u32::try_from(self.len())?;
        let destination = memory.allocate(length)?;

        memory.write(*destination, self.as_bytes())?;

        Ok(destination.lower(memory)? + hlist![length as i32])
    }
}
