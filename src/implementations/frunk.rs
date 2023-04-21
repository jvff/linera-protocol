use {
    crate::{util::Split, GuestMemory, GuestPointer, Layout, WitLoad, WitStore, WitType},
    frunk::{HCons, HNil},
    std::ops::Add,
};

impl WitType for HNil {
    const SIZE: u32 = 0;

    type Layout = HNil;
}

impl WitLoad for HNil {
    fn load<Memory>(_memory: &Memory, _location: GuestPointer) -> Result<Self, Memory::Error>
    where
        Memory: GuestMemory,
    {
        Ok(HNil)
    }

    fn lift_from<Memory>(
        HNil: <Self::Layout as Layout>::Flat,
        _memory: &Memory,
    ) -> Result<Self, Memory::Error>
    where
        Memory: GuestMemory,
    {
        Ok(HNil)
    }
}

impl WitStore for HNil {
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

impl<Head, Tail> WitType for HCons<Head, Tail>
where
    Head: WitType,
    Tail: WitType,
    Head::Layout: Add<Tail::Layout>,
    <Head::Layout as Add<Tail::Layout>>::Output: Layout,
{
    const SIZE: u32 = Head::SIZE + Tail::SIZE;

    type Layout = <Head::Layout as Add<Tail::Layout>>::Output;
}

impl<Head, Tail> WitLoad for HCons<Head, Tail>
where
    Head: WitLoad,
    Tail: WitLoad,
    Head::Layout: Add<Tail::Layout>,
    <Head::Layout as Add<Tail::Layout>>::Output: Layout,
    <Self::Layout as Layout>::Flat:
        Split<<Head::Layout as Layout>::Flat, Remainder = <Tail::Layout as Layout>::Flat>,
{
    fn load<Memory>(memory: &Memory, location: GuestPointer) -> Result<Self, Memory::Error>
    where
        Memory: GuestMemory,
    {
        Ok(HCons {
            head: Head::load(memory, location)?,
            tail: Tail::load(memory, location.after::<Head>())?,
        })
    }

    fn lift_from<Memory>(
        layout: <Self::Layout as Layout>::Flat,
        memory: &Memory,
    ) -> Result<Self, Memory::Error>
    where
        Memory: GuestMemory,
    {
        let (head_layout, tail_layout) = layout.split();

        Ok(HCons {
            head: Head::lift_from(head_layout, memory)?,
            tail: Tail::lift_from(tail_layout, memory)?,
        })
    }
}

impl<Head, Tail> WitStore for HCons<Head, Tail>
where
    Head: WitStore,
    Tail: WitStore,
    Head::Layout: Add<Tail::Layout>,
    <Head::Layout as Add<Tail::Layout>>::Output: Layout,
    <Head::Layout as Layout>::Flat: Add<<Tail::Layout as Layout>::Flat>,
    Self::Layout: Layout<
        Flat = <<Head::Layout as Layout>::Flat as Add<<Tail::Layout as Layout>::Flat>>::Output,
    >,
{
    fn store<Memory>(
        &self,
        memory: &mut Memory,
        location: GuestPointer,
    ) -> Result<(), Memory::Error>
    where
        Memory: GuestMemory,
    {
        self.head.store(memory, location)?;
        self.tail.store(memory, location.after::<Head>())?;

        Ok(())
    }

    fn lower<Memory>(
        &self,
        memory: &mut Memory,
    ) -> Result<<Self::Layout as Layout>::Flat, Memory::Error>
    where
        Memory: GuestMemory,
    {
        let head_layout = self.head.lower(memory)?;
        let tail_layout = self.tail.lower(memory)?;

        Ok(head_layout + tail_layout)
    }
}
