use {
    crate::{flat_type::FlatType, Layout},
    frunk::{hlist, hlist_pat, HCons, HList},
    wasmtime::{WasmParams, WasmTy},
};

pub trait WasmtimeParameters {
    type Parameters: WasmParams;

    fn into_wasmtime(self) -> Self::Parameters;
    fn from_wasmtime(parameters: Self::Parameters) -> Self;
}

macro_rules! parameters {
    ($( $names:ident : $types:ident ),*) => {
        parameters!(| $( $names: $types ),*);
    };

    ($( $names:ident : $types:ident ),* |) => {
        parameters!(@generate $( $names: $types ),*);
    };

    (
        $( $names_to_generate:ident : $types_to_generate:ident ),* |
        $next_name:ident : $next_type:ident $( , $queued_names:ident : $queued_types:ident )*
    ) => {
        parameters!(@generate $( $names_to_generate: $types_to_generate ),*);
        parameters!(
            $( $names_to_generate: $types_to_generate, )*
            $next_name: $next_type | $( $queued_names: $queued_types ),*);
    };

    (@generate $( $names:ident : $types:ident ),*) => {
        impl<$( $types ),*> WasmtimeParameters for HList![$( $types ),*]
        where
            $( $types: FlatType + WasmTy, )*
        {
            type Parameters = ($( $types, )*);

            #[allow(clippy::unused_unit)]
            fn into_wasmtime(self) -> Self::Parameters {
                let hlist_pat![$( $names ),*] = self;

                ($( $names, )*)
            }

            fn from_wasmtime(($( $names, )*): Self::Parameters) -> Self {
                hlist![$( $names ),*]
            }
        }
    };
}

parameters!(
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
    p: P
);

impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, Rest> WasmtimeParameters
    for HCons<
        A,
        HCons<
            B,
            HCons<
                C,
                HCons<
                    D,
                    HCons<
                        E,
                        HCons<
                            F,
                            HCons<
                                G,
                                HCons<
                                    H,
                                    HCons<
                                        I,
                                        HCons<
                                            J,
                                            HCons<
                                                K,
                                                HCons<
                                                    L,
                                                    HCons<
                                                        M,
                                                        HCons<
                                                            N,
                                                            HCons<O, HCons<P, HCons<Q, Rest>>>,
                                                        >,
                                                    >,
                                                >,
                                            >,
                                        >,
                                    >,
                                >,
                            >,
                        >,
                    >,
                >,
            >,
        >,
    >
where
    A: FlatType,
    B: FlatType,
    C: FlatType,
    D: FlatType,
    E: FlatType,
    F: FlatType,
    G: FlatType,
    H: FlatType,
    I: FlatType,
    J: FlatType,
    K: FlatType,
    L: FlatType,
    M: FlatType,
    N: FlatType,
    O: FlatType,
    P: FlatType,
    Q: FlatType,
    Rest: Layout,
{
    type Parameters = (i32,);

    fn into_wasmtime(self) -> Self::Parameters {
        unreachable!("Attempt to convert a list of flat parameters larger than the maximum limit");
    }

    fn from_wasmtime(_: Self::Parameters) -> Self {
        unreachable!(
            "Attempt to convert into a list of flat parameters larger than the maximum limit"
        );
    }
}
