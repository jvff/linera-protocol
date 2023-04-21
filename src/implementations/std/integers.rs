use {
    crate::{GuestMemory, GuestPointer, Layout, WitLoad, WitStore, WitType},
    frunk::{hlist, hlist_pat, HList},
    std::ops::Deref,
};

macro_rules! impl_wit_traits {
    ($integer:ty, 1) => {
        impl WitType for $integer {
            const SIZE: u32 = 1;

            type Layout = HList![$integer];
        }

        impl WitLoad for $integer {
            fn load<Memory>(memory: &Memory, location: GuestPointer) -> Result<Self, Memory::Error>
            where
                Memory: GuestMemory,
            {
                let slice = memory.read(location, 1)?;
                Ok(slice[0] as $integer)
            }

            fn lift_from<Memory>(
                hlist_pat![value]: <Self::Layout as Layout>::Flat,
                _memory: &Memory,
            ) -> Result<Self, Memory::Error>
            where
                Memory: GuestMemory,
            {
                Ok(value as $integer)
            }
        }

        impl WitStore for $integer {
            fn store<Memory>(
                &self,
                memory: &mut Memory,
                location: GuestPointer,
            ) -> Result<(), Memory::Error>
            where
                Memory: GuestMemory,
            {
                memory.write(location, &[*self as u8])
            }

            fn lower<Memory>(
                &self,
                _memory: &mut Memory,
            ) -> Result<<Self::Layout as Layout>::Flat, Memory::Error>
            where
                Memory: GuestMemory,
            {
                Ok(hlist![*self as i32])
            }
        }
    };

    ($integer:ty, $size:expr, $flat_type:ty) => {
        impl_wit_traits!(
            $integer,
            $size,
            ($integer),
            ($flat_type),
            self -> hlist![*self as $flat_type],
            hlist_pat![value] => value as Self
        );
    };

    (
        $integer:ty,
        $size:expr,
        ($( $simple_types:ty ),*),
        ($( $flat_types:ty ),*),
        $this:ident -> $lower:expr,
        $lift_pattern:pat => $lift:expr
    ) => {
        impl WitType for $integer {
            const SIZE: u32 = $size;

            type Layout = HList![$( $simple_types ),*];
        }

        impl WitLoad for $integer {
            fn load<Memory>(memory: &Memory, location: GuestPointer) -> Result<Self, Memory::Error>
            where
                Memory: GuestMemory,
            {
                let bytes = memory
                    .read(location, Self::SIZE)?
                    .deref()
                    .try_into()
                    .expect("Incorrect number of bytes read");

                Ok(Self::from_le_bytes(bytes))
            }

            fn lift_from<Memory>(
                $lift_pattern: <Self::Layout as Layout>::Flat,
                _memory: &Memory,
            ) -> Result<Self, Memory::Error>
            where
                Memory: GuestMemory,
            {
                Ok($lift)
            }
        }

        impl WitStore for $integer {
            fn store<Memory>(
                &self,
                memory: &mut Memory,
                location: GuestPointer,
            ) -> Result<(), Memory::Error>
            where
                Memory: GuestMemory,
            {
                memory.write(location, &self.to_le_bytes())
            }

            fn lower<Memory>(
                &$this,
                _memory: &mut Memory,
            ) -> Result<<Self::Layout as Layout>::Flat, Memory::Error>
            where
                Memory: GuestMemory,
            {
                Ok($lower)
            }
        }
    };
}

impl_wit_traits!(u8, 1);
impl_wit_traits!(i8, 1);
impl_wit_traits!(u16, 2, i32);
impl_wit_traits!(i16, 2, i32);
impl_wit_traits!(u32, 4, i32);
impl_wit_traits!(i32, 4, i32);
impl_wit_traits!(u64, 8, i64);
impl_wit_traits!(i64, 8, i64);

macro_rules! x128_lower {
    ($this:ident) => {
        hlist![
            ($this & ((1 << 64) - 1)) as i64,
            (($this >> 64) & ((1 << 64) - 1)) as i64,
        ]
    };
}

impl_wit_traits!(
    u128,
    16,
    (u64, u64),
    (i64, i64),
    self -> x128_lower!(self),
    hlist_pat![least_significant_bytes, most_significant_bytes] => {
        ((most_significant_bytes as Self) << 64)
        | (least_significant_bytes as Self & ((1 << 64) - 1))
    }
);

impl_wit_traits!(
    i128,
    16,
    (i64, i64),
    (i64, i64),
    self -> x128_lower!(self),
    hlist_pat![least_significant_bytes, most_significant_bytes] => {
        ((most_significant_bytes as Self) << 64)
        | (least_significant_bytes as Self & ((1 << 64) - 1))
    }
);
