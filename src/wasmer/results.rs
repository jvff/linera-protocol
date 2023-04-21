use {
    crate::{flat_type::FlatType, Layout},
    frunk::{hlist, hlist_pat, HList},
    wasmer::{FromToNativeWasmType, WasmTypeList},
};

pub trait WasmerResults: Layout {
    type Results: WasmTypeList;

    fn from_wasmer(results: Self::Results) -> Self::Flat;
    fn into_wasmer(self) -> Self::Results;
}

impl WasmerResults for HList![] {
    type Results = ();

    fn from_wasmer((): Self::Results) -> Self::Flat {
        hlist![]
    }

    fn into_wasmer(self) -> Self::Results {}
}

impl<T> WasmerResults for HList![T]
where
    T: FlatType + FromToNativeWasmType,
{
    type Results = T;

    fn from_wasmer(value: Self::Results) -> Self {
        hlist![value]
    }

    fn into_wasmer(self) -> Self::Results {
        let hlist_pat![value] = self;
        value
    }
}
