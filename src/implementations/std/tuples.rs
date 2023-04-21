use {
    crate::{GuestMemory, GuestPointer, Layout, WitLoad, WitStore, WitType},
    frunk::{hlist, hlist_pat, HList},
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

    (@generate $( $names:ident : $types:ident, )*) => {
        impl<$( $types ),*> WitType for ($( $types, )*)
        where
            $( $types: WitType, )*
            HList![$( $types ),*]: WitType,
        {
            const SIZE: u32 = <HList![$( $types ),*] as WitType>::SIZE;

            type Layout = <HList![$( $types ),*] as WitType>::Layout;
        }

        impl<$( $types ),*> WitLoad for ($( $types, )*)
        where
            $( $types: WitLoad, )*
            HList![$( $types ),*]: WitLoad,
        {
            fn load<Memory>(memory: &Memory, location: GuestPointer) -> Result<Self, Memory::Error>
            where
                Memory: GuestMemory,
            {
                let hlist_pat![$( $names, )*] =
                    <HList![$( $types, )*] as WitLoad>::load(memory, location)?;

                Ok(($( $names, )*))
            }

            fn lift_from<Memory>(
                layout: <Self::Layout as Layout>::Flat,
                memory: &Memory,
            ) -> Result<Self, Memory::Error>
            where
                Memory: GuestMemory,
            {
                let hlist_pat![$( $names, )*] =
                    <HList![$( $types, )*] as WitLoad>::lift_from(layout, memory)?;

                Ok(($( $names, )*))
            }
        }

        impl<$( $types ),*> WitStore for ($( $types, )*)
        where
            $( $types: WitStore, )*
            HList![$( $types ),*]: WitStore,
            for<'a> HList![$( &'a $types ),*]:
                WitType<Layout = <HList![$( $types ),*] as WitType>::Layout> + WitStore,
        {
            fn store<Memory>(
                &self,
                memory: &mut Memory,
                location: GuestPointer,
            ) -> Result<(), Memory::Error>
            where
                Memory: GuestMemory,
            {
                let ($( $names, )*) = self;

                hlist![$( $names ),*].store(memory, location)?;

                Ok(())
            }

            fn lower<Memory>(
                &self,
                memory: &mut Memory,
            ) -> Result<<Self::Layout as Layout>::Flat, Memory::Error>
            where
                Memory: GuestMemory,
            {
                let ($( $names, )*) = self;

                hlist![$( $names ),*].lower(memory)
            }
        }
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
