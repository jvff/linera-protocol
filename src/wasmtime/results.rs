use {
    crate::{flat_type::FlatType, Layout},
    frunk::{hlist, hlist_pat, HList},
    wasmtime::{WasmResults, WasmTy},
};

pub trait WasmtimeResults: Layout {
    type Results: WasmResults;

    fn from_wasmtime(results: Self::Results) -> Self::Flat;
    fn into_wasmtime(self) -> Self::Results;
}

impl WasmtimeResults for HList![] {
    type Results = ();

    fn from_wasmtime((): Self::Results) -> Self::Flat {
        hlist![]
    }

    fn into_wasmtime(self) -> Self::Results {}
}

impl<T> WasmtimeResults for HList![T]
where
    T: FlatType + WasmTy,
{
    type Results = T;

    fn from_wasmtime(value: Self::Results) -> Self {
        hlist![value]
    }

    fn into_wasmtime(self) -> Self::Results {
        let hlist_pat![value] = self;
        value
    }
}
