use {
    crate::{GuestMemory, GuestPointer, Layout, WitLoad, WitStore, WitType},
    frunk::{hlist_pat, HList},
    log::Level,
};

impl WitType for Level {
    const SIZE: u32 = 1;

    type Layout = HList![i8];
}

impl WitLoad for Level {
    fn load<Memory>(memory: &Memory, location: GuestPointer) -> Result<Self, Memory::Error>
    where
        Memory: GuestMemory,
    {
        match u8::load(memory, location)? {
            0 => Ok(Level::Error),
            1 => Ok(Level::Warn),
            2 => Ok(Level::Info),
            3 => Ok(Level::Debug),
            4 => Ok(Level::Trace),
            _ => unreachable!("Invalid log level"),
        }
    }

    fn lift_from<Memory>(
        hlist_pat![discriminant]: <Self::Layout as Layout>::Flat,
        _memory: &Memory,
    ) -> Result<Self, Memory::Error>
    where
        Memory: GuestMemory,
    {
        match discriminant {
            0 => Ok(Level::Error),
            1 => Ok(Level::Warn),
            2 => Ok(Level::Info),
            3 => Ok(Level::Debug),
            4 => Ok(Level::Trace),
            _ => unreachable!("Invalid log level"),
        }
    }
}

impl WitStore for Level {
    fn store<Memory>(
        &self,
        memory: &mut Memory,
        location: GuestPointer,
    ) -> Result<(), Memory::Error>
    where
        Memory: GuestMemory,
    {
        let discriminant: i8 = match self {
            Level::Error => 0,
            Level::Warn => 1,
            Level::Info => 2,
            Level::Debug => 3,
            Level::Trace => 4,
        };

        discriminant.store(memory, location)
    }

    fn lower<Memory>(
        &self,
        memory: &mut Memory,
    ) -> Result<<Self::Layout as Layout>::Flat, Memory::Error>
    where
        Memory: GuestMemory,
    {
        let discriminant: i8 = match self {
            Level::Error => 0,
            Level::Warn => 1,
            Level::Info => 2,
            Level::Debug => 3,
            Level::Trace => 4,
        };

        discriminant.lower(memory)
    }
}
