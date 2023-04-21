use {
    crate::{
        GuestMemory, GuestPointer, JoinFlatLayouts, Layout, Merge, SplitFlatLayouts, WitLoad,
        WitStore, WitType,
    },
    frunk::{hlist, hlist_pat, HCons},
};

impl<T, E> WitType for Result<T, E>
where
    T: WitType,
    E: WitType,
    T::Layout: Merge<E::Layout>,
    <T::Layout as Merge<E::Layout>>::Output: Layout,
{
    const SIZE: u32 = {
        if T::SIZE > E::SIZE {
            1 + T::SIZE
        } else {
            1 + E::SIZE
        }
    };

    type Layout = HCons<i8, <T::Layout as Merge<E::Layout>>::Output>;
}

impl<T, E> WitLoad for Result<T, E>
where
    T: WitLoad,
    E: WitLoad,
    T::Layout: Merge<E::Layout>,
    <T::Layout as Merge<E::Layout>>::Output: Layout,
    <<T::Layout as Merge<E::Layout>>::Output as Layout>::Flat: SplitFlatLayouts<<T::Layout as Layout>::Flat>
        + SplitFlatLayouts<<E::Layout as Layout>::Flat>,
{
    fn load<Memory>(memory: &Memory, location: GuestPointer) -> Result<Self, Memory::Error>
    where
        Memory: GuestMemory,
    {
        let is_err = bool::load(memory, location)?;

        match is_err {
            true => Ok(Err(E::load(
                memory,
                location.after::<bool>().after_padding_for::<E>(),
            )?)),
            false => Ok(Ok(T::load(
                memory,
                location.after::<bool>().after_padding_for::<T>(),
            )?)),
        }
    }

    fn lift_from<Memory>(
        hlist_pat![is_err, ...value_layout]: <Self::Layout as Layout>::Flat,
        memory: &Memory,
    ) -> Result<Self, Memory::Error>
    where
        Memory: GuestMemory,
    {
        let is_err = bool::lift_from(hlist![is_err], memory)?;

        match is_err {
            false => Ok(Ok(T::lift_from(value_layout.split(), memory)?)),
            true => Ok(Err(E::lift_from(value_layout.split(), memory)?)),
        }
    }
}

impl<T, E> WitStore for Result<T, E>
where
    T: WitStore,
    E: WitStore,
    T::Layout: Merge<E::Layout>,
    <T::Layout as Merge<E::Layout>>::Output: Layout,
    <T::Layout as Layout>::Flat:
        JoinFlatLayouts<<<T::Layout as Merge<E::Layout>>::Output as Layout>::Flat>,
    <E::Layout as Layout>::Flat:
        JoinFlatLayouts<<<T::Layout as Merge<E::Layout>>::Output as Layout>::Flat>,
{
    fn store<Memory>(
        &self,
        memory: &mut Memory,
        location: GuestPointer,
    ) -> Result<(), Memory::Error>
    where
        Memory: GuestMemory,
    {
        match self {
            Ok(value) => {
                false.store(memory, location)?;
                value.store(memory, location.after::<bool>().after_padding_for::<T>())
            }
            Err(error) => {
                true.store(memory, location)?;
                error.store(memory, location.after::<bool>().after_padding_for::<E>())
            }
        }
    }

    fn lower<Memory>(
        &self,
        memory: &mut Memory,
    ) -> Result<<Self::Layout as Layout>::Flat, Memory::Error>
    where
        Memory: GuestMemory,
    {
        match self {
            Ok(value) => Ok(false.lower(memory)? + value.lower(memory)?.join()),
            Err(error) => Ok(true.lower(memory)? + error.lower(memory)?.join()),
        }
    }
}
