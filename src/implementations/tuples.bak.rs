use {
    crate::{util::Split, GuestMemory, GuestPointer, Layout, WitLoad, WitStore, WitType},
    frunk::{hlist, hlist::Sculptor, hlist_pat, HList},
    std::ops::Add,
};

macro_rules! impl_wit_traits {
    ($head_name:ident : $head_type:ident, $( $tail_names:ident : $tail_types:ident ),*) => {
        impl_wit_traits!($head_name: $head_type | $( $tail_names: $tail_types ),*);
    };

    ($( $names:ident : $types:ident ),* |) => {
        impl_wit_traits!(@generate $( $names: $types, )*);
    };

    (
        $( $names_to_generate:ident : $types_to_generate:ident ),* |
        $next_name:ident : $next_type:ident $( , $queued_names:ident : $queued_types:ident )*
    ) => {
        impl_wit_traits!(@generate $( $names_to_generate: $types_to_generate, )*);
        impl_wit_traits!(
            $( $names_to_generate: $types_to_generate, )*
            $next_name: $next_type | $( $queued_names: $queued_types ),*);
    };

    (
        @generate
        $head_name:ident : $head_type:ident,
        $( $tail_names:ident : $tail_types:ident, )*
    ) => {
        // impl<$head_type, $( $tail_types ),*> WitType for ($head_type, $( $tail_types, )*)
        // where
            // $head_type: WitType,
            // $( $tail_types: WitType, )*
            // $head_type::Layout: Add<<($( $tail_types, )*) as WitType>::Layout>,
            // <$head_type::Layout as Add<<($( $tail_types, )*) as WitType>::Layout>>::Output: Layout,
        // {
            // const SIZE: u32 = $head_type::SIZE + <($( $tail_types, )*) as WitType>::SIZE;

            // type Layout =
                // <$head_type::Layout as Add<<($( $tail_types, )*) as WitType>::Layout>>::Output;
            // type WitExportName = String;

            // fn wit_export_name() -> Self::WitExportName {
                // todo!();
            // }
        // }

        // impl<$head_type, $( $tail_types ),*> WitLoad for ($head_type, $( $tail_types, )*)
        // where
            // $head_type: WitLoad,
            // $( $tail_types: WitLoad, )*
            // $head_type::Layout: Add<<($( $tail_types, )*) as WitType>::Layout>,
            // <$head_type::Layout as Add<<($( $tail_types, )*) as WitType>::Layout>>::Output: Layout,
        // {
            // fn load<Memory>(memory: &Memory, location: GuestPointer) -> Result<Self, Memory::Error>
            // where
                // Memory: GuestMemory,
            // {
                // let $head_name = $head_type::load(memory, location)?;
                // let location = location.after::<$head_type>();

                // $(
                    // let $tail_names =
                        // $tail_types::load(memory, location.after_padding_for::<$tail_types>())?;
                    // let location = location.after::<$tail_types>();
                // )*

                // Ok(( $head_name, $($tail_names,)* ))
            // }

            // fn lift_from<Memory>(
                // layout: <Self::Layout as Layout>::Flat,
                // memory: &Memory,
            // ) -> Result<Self, Memory::Error>
            // where
                // Memory: GuestMemory,
            // {
                // let (head_layout, tail_layout) = layout.sculpt();

                // let $head_name = $head_type::lift_from(head_layout, memory)?;
                // let ( $($tail_names,)* ) =
                    // <($( $tail_types, )*) as WitLoad>::lift_from(tail_layout, memory)?;

                // Ok(($head_name, $( $tail_names, )*))
            // }
        // }

        // impl<$head_type, $( $tail_types ),*> WitStore for ($head_type, $( $tail_types, )*)
        // where
            // $head_type: WitStore,
            // $( $tail_types: WitStore, )*
            // $head_type::Layout: Add<<($( $tail_types, )*) as WitType>::Layout>,
            // <$head_type::Layout as Add<<($( $tail_types, )*) as WitType>::Layout>>::Output: Layout,
        // {
            // fn store<Memory>(
                // &self,
                // memory: &mut Memory,
                // location: GuestPointer,
            // ) -> Result<(), Memory::Error>
            // where
                // Memory: GuestMemory,
            // {
                // let ($head_name, $( $tail_names, )*) = self;

                // $head_name.store(memory, location)?;
                // let location = location.after::<$head_type>();

                // $(
                    // location.after_padding_for::<$tail_types>();
                    // memory.write(location, &self.to_le_bytes())?;
                    // let location = location.after::<$tail_types>();
                // )*

                // Ok(())
            // }

            // fn lower<Memory>(
                // &self,
                // memory: &mut Memory,
            // ) -> Result<<Self::Layout as Layout>::Flat, Memory::Error>
            // where
                // Memory: GuestMemory,
            // {
                // let ($head_name, $( $tail_names, )*) = self;
                // let tail = ($( $tail_names, )*);

                // Ok($head_name.lower(memory)? + tail.lower(memory)?)
            // }
        // }
    };
}

impl_wit_traits!(
    a: A,
    b: B,
    c: C,
    d: D,
    e: E,
    f: F,
    g: G,
    h: H,
    i: I,
    j: J,
    k: K,
    l: L,
    m: M,
    n: N,
    o: O,
    p: P,
    q: Q,
    r: R,
    s: S,
    t: T,
    u: U,
    v: V,
    w: W,
    x: X,
    y: Y,
    z: Z
);

impl<A> WitType for (A,)
where
    A: WitType,
    // A::Layout: Add<<() as WitType>::Layout>,
    // A::Layout: Add<frunk::HNil>,
    // <A::Layout as Add<<() as WitType>::Layout>>::Output: Layout,
{
    // const SIZE: u32 = A::SIZE + <() as WitType>::SIZE;

    // type Layout = <A::Layout as Add<<() as WitType>::Layout>>::Output;
    const SIZE: u32 = A::SIZE;

    type Layout = A::Layout;
    type WitExportName = String;

    fn wit_export_name() -> Self::WitExportName {
        todo!();
    }
}

impl<A> WitLoad for (A,)
where
    A: WitLoad,
{
    fn load<Memory>(memory: &Memory, location: GuestPointer) -> Result<Self, Memory::Error>
    where
        Memory: GuestMemory,
    {
        let a = A::load(memory, location)?;

        Ok((a,))
    }

    fn lift_from<Memory>(
        layout: <Self::Layout as Layout>::Flat,
        memory: &Memory,
    ) -> Result<Self, Memory::Error>
    where
        Memory: GuestMemory,
    {
        let a = A::lift_from(layout, memory)?;

        Ok((a,))
    }
}

impl<A> WitStore for (A,)
where
    A: WitStore,
    // <A::Layout as Add<<() as WitType>::Layout>>::Output: Layout,
{
    fn store<Memory>(
        &self,
        memory: &mut Memory,
        location: GuestPointer,
    ) -> Result<(), Memory::Error>
    where
        Memory: GuestMemory,
    {
        let (a,) = self;

        a.store(memory, location)?;

        Ok(())
    }

    fn lower<Memory>(
        &self,
        memory: &mut Memory,
    ) -> Result<<Self::Layout as Layout>::Flat, Memory::Error>
    where
        Memory: GuestMemory,
    {
        let (a,) = self;

        a.lower(memory)
    }
}

impl<A, B> WitType for (A, B)
where
    A: WitType,
    B: WitType,
    A::Layout: Add<B::Layout>,
    <A::Layout as Add<B::Layout>>::Output: Layout,
{
    const SIZE: u32 = A::SIZE + B::SIZE;

    type Layout = <A::Layout as Add<B::Layout>>::Output;
    type WitExportName = String;

    fn wit_export_name() -> Self::WitExportName {
        todo!();
    }
}

impl<A, B> WitLoad for (A, B)
where
    A: WitLoad,
    B: WitLoad,
    A::Layout: Add<B::Layout>,
    <A::Layout as Add<B::Layout>>::Output: Layout + Split<A::Layout>,
    <<A::Layout as Add<B::Layout>>::Output as Layout>::Flat:
        Split<<A::Layout as Layout>::Flat, Remainder = <B::Layout as Layout>::Flat>,
{
    fn load<Memory>(memory: &Memory, location: GuestPointer) -> Result<Self, Memory::Error>
    where
        Memory: GuestMemory,
    {
        let a = A::load(memory, location)?;
        let b = B::load(memory, location.after::<A>().after_padding_for::<B>())?;

        Ok((a, b))
    }

    fn lift_from<Memory>(
        layout: <Self::Layout as Layout>::Flat,
        memory: &Memory,
    ) -> Result<Self, Memory::Error>
    where
        Memory: GuestMemory,
    {
        let (a_layout, b_layout) = layout.split();
        let a = A::lift_from(a_layout, memory)?;
        let b = B::lift_from(b_layout, memory)?;

        Ok((a, b))
    }
}

impl<A, B> WitStore for (A, B)
where
    A: WitStore,
    B: WitStore,
    A::Layout: Add<B::Layout>,
    <A::Layout as Add<B::Layout>>::Output: Layout + Split<A::Layout>,
    <A::Layout as Layout>::Flat:
        Add<<B::Layout as Layout>::Flat, Output = <Self::Layout as Layout>::Flat>,
{
    fn store<Memory>(
        &self,
        memory: &mut Memory,
        location: GuestPointer,
    ) -> Result<(), Memory::Error>
    where
        Memory: GuestMemory,
    {
        let (a, b) = self;

        a.store(memory, location)?;
        let location = location.after::<A>();
        let location = location.after_padding_for::<B>();
        b.store(memory, location)?;
        // let location = location.after::<B>();
        // let location = location.after_padding_for::<C>();
        // c.store(memory, location)?;

        Ok(())
    }

    fn lower<Memory>(
        &self,
        memory: &mut Memory,
    ) -> Result<<Self::Layout as Layout>::Flat, Memory::Error>
    where
        Memory: GuestMemory,
    {
        let (a, b) = self;

        Ok(a.lower(memory)? + b.lower(memory)?)
    }
}

impl<A, B, C> WitType for (A, B, C)
where
    A: WitType,
    (B, C): WitType,
    A::Layout: Add<<(B, C) as WitType>::Layout>,
    <A::Layout as Add<<(B, C) as WitType>::Layout>>::Output: Layout,
{
    const SIZE: u32 = A::SIZE + <(B, C)>::SIZE;

    type Layout = <A::Layout as Add<<(B, C) as WitType>::Layout>>::Output;
    type WitExportName = String;

    fn wit_export_name() -> Self::WitExportName {
        todo!();
    }
}

impl<A, B, C> WitLoad for (A, B, C)
where
    A: WitLoad,
    (B, C): WitLoad,
    A::Layout: Add<<(B, C) as WitType>::Layout>,
    <A::Layout as Add<<(B, C) as WitType>::Layout>>::Output: Layout,
    <<A::Layout as Add<<(B, C) as WitType>::Layout>>::Output as Layout>::Flat: Split<
        <A::Layout as Layout>::Flat,
        Remainder = <<(B, C) as WitType>::Layout as Layout>::Flat,
    >,
{
    fn load<Memory>(memory: &Memory, location: GuestPointer) -> Result<Self, Memory::Error>
    where
        Memory: GuestMemory,
    {
        let a = A::load(memory, location)?;
        let (b, c) =
            <(B, C) as WitLoad>::load(memory, location.after::<A>().after_padding_for::<(B, C)>())?;

        Ok((a, b, c))
    }

    fn lift_from<Memory>(
        layout: <Self::Layout as Layout>::Flat,
        memory: &Memory,
    ) -> Result<Self, Memory::Error>
    where
        Memory: GuestMemory,
    {
        let (a_layout, tail_layout) = layout.split();
        let a = A::lift_from(a_layout, memory)?;
        let (b, c) = <(B, C) as WitLoad>::lift_from(tail_layout, memory)?;

        Ok((a, b, c))
    }
}

impl<A, B, C> WitStore for (A, B, C)
where
    A: WitStore,
    (B, C): WitStore,
    A::Layout: Add<<(B, C) as WitType>::Layout>,
    <A::Layout as Add<<(B, C) as WitType>::Layout>>::Output: Layout + Split<A::Layout>,
    <A::Layout as Layout>::Flat:
        Add<<<(B, C) as WitType>::Layout as Layout>::Flat, Output = <Self::Layout as Layout>::Flat>,
{
    fn store<Memory>(
        &self,
        memory: &mut Memory,
        location: GuestPointer,
    ) -> Result<(), Memory::Error>
    where
        Memory: GuestMemory,
    {
        let (a, b, c) = self;

        a.store(memory, location)?;
        (*b, *c).store(memory, location.after::<A>().after_padding_for::<(B, C)>())?;

        Ok(())
    }

    fn lower<Memory>(
        &self,
        memory: &mut Memory,
    ) -> Result<<Self::Layout as Layout>::Flat, Memory::Error>
    where
        Memory: GuestMemory,
    {
        let (a, b, c) = self;

        Ok(a.lower(memory)? + (*b, *c).lower(memory)?)
    }
}
