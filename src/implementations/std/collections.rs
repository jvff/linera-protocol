use {
    crate::{GuestMemory, GuestPointer, Layout, WitLoad, WitStore, WitType},
    frunk::{hlist, hlist_pat, HList},
};

impl<T> WitType for Vec<T>
where
    T: WitType,
{
    const SIZE: u32 = 8;

    type Layout = HList![i32, i32];
}

impl<T> WitLoad for Vec<T>
where
    T: WitLoad,
{
    fn load<Memory>(memory: &Memory, location: GuestPointer) -> Result<Self, Memory::Error>
    where
        Memory: GuestMemory,
    {
        let address = GuestPointer::load(memory, location)?;
        let length = u32::load(memory, location.after::<GuestPointer>())?;

        (0..length)
            .map(|index| T::load(memory, address.index::<T>(index)))
            .collect()
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

        (0..length)
            .map(|index| T::load(memory, address.index::<T>(index)))
            .collect()
    }
}

impl<T> WitStore for Vec<T>
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
        let length = u32::try_from(self.len())?;
        let size = length * T::SIZE;

        let destination = memory.allocate(size)?;

        destination.store(memory, location)?;
        length.store(memory, location.after::<GuestPointer>())?;

        self.iter()
            .zip(0..)
            .try_for_each(|(element, index)| element.store(memory, destination.index::<T>(index)))
    }

    fn lower<Memory>(&self, memory: &mut Memory) -> Result<Self::Layout, Memory::Error>
    where
        Memory: GuestMemory,
    {
        let length = u32::try_from(self.len())?;
        let size = length * T::SIZE;

        let destination = memory.allocate(size)?;

        self.iter().zip(0..).try_for_each(|(element, index)| {
            element.store(memory, destination.index::<T>(index))
        })?;

        Ok(destination.lower(memory)? + hlist![length as i32])
    }
}
