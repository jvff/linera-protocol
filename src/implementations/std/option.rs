use {
    crate::{
        GuestMemory, GuestPointer, JoinFlatLayouts, Layout, Merge, SplitFlatLayouts, WitLoad,
        WitStore, WitType,
    },
    frunk::{hlist, hlist_pat, HCons, HNil},
};

impl<T> WitType for Option<T>
where
    T: WitType,
    HNil: Merge<T::Layout>,
    <HNil as Merge<T::Layout>>::Output: Layout,
{
    const SIZE: u32 = 1 + T::SIZE;

    type Layout = HCons<i8, <HNil as Layout>::Merge<T::Layout>>;
}

impl<T> WitLoad for Option<T>
where
    T: WitLoad,
    HNil: Merge<T::Layout>,
    <HNil as Merge<T::Layout>>::Output: Layout,
    <<HNil as Merge<T::Layout>>::Output as Layout>::Flat:
        SplitFlatLayouts<<T::Layout as Layout>::Flat>,
{
    fn load<Memory>(memory: &Memory, location: GuestPointer) -> Result<Self, Memory::Error>
    where
        Memory: GuestMemory,
    {
        let is_some = bool::load(memory, location)?;

        match is_some {
            true => Ok(Some(T::load(
                memory,
                location.after::<bool>().after_padding_for::<T>(),
            )?)),
            false => Ok(None),
        }
    }

    fn lift_from<Memory>(
        hlist_pat![is_some, ...value_layout]: <Self::Layout as Layout>::Flat,
        memory: &Memory,
    ) -> Result<Self, Memory::Error>
    where
        Memory: GuestMemory,
    {
        let is_some = bool::lift_from(hlist![is_some], memory)?;

        if is_some {
            Ok(Some(T::lift_from(value_layout.split(), memory)?))
        } else {
            Ok(None)
        }
    }
}

impl<T> WitStore for Option<T>
where
    T: WitStore,
    HNil: Merge<T::Layout>,
    <HNil as Merge<T::Layout>>::Output: Layout,
    <T::Layout as Layout>::Flat:
        JoinFlatLayouts<<<HNil as Merge<T::Layout>>::Output as Layout>::Flat>,
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
            Some(value) => {
                true.store(memory, location)?;
                value.store(memory, location.after::<bool>().after_padding_for::<T>())
            }
            None => false.store(memory, location),
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
            Some(value) => Ok(true.lower(memory)? + value.lower(memory)?.join()),
            None => Ok(false.lower(memory)? + Default::default()),
        }
    }
}
