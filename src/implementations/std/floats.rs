use {
    crate::{GuestMemory, GuestPointer, Layout, WitLoad, WitStore, WitType},
    frunk::{hlist, hlist_pat, HList},
    std::ops::Deref,
};

macro_rules! impl_wit_traits {
    ($float:ty, $size:expr) => {
        impl WitType for $float {
            const SIZE: u32 = $size;

            type Layout = HList![$float];
        }

        impl WitLoad for $float {
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
                hlist_pat![value]: <Self::Layout as Layout>::Flat,
                _memory: &Memory,
            ) -> Result<Self, Memory::Error>
            where
                Memory: GuestMemory,
            {
                Ok(value)
            }
        }

        impl WitStore for $float {
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
                &self,
                _memory: &mut Memory,
            ) -> Result<<Self::Layout as Layout>::Flat, Memory::Error>
            where
                Memory: GuestMemory,
            {
                Ok(hlist![*self])
            }
        }
    };
}

impl_wit_traits!(f32, 4);
impl_wit_traits!(f64, 8);
