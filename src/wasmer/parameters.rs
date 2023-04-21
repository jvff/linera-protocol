use {
    crate::{flat_type::FlatType, Layout},
    frunk::{hlist, hlist_pat, HCons, HList},
    wasmer::FromToNativeWasmType,
};

pub trait WasmerParameters {
    type ImportParameters;
    type ExportParameters;

    fn into_wasmer(self) -> Self::ImportParameters;
    fn from_wasmer(parameters: Self::ExportParameters) -> Self;
}

impl<Parameter> WasmerParameters for HList![Parameter]
where
    Parameter: FlatType + FromToNativeWasmType,
{
    type ImportParameters = Parameter;
    type ExportParameters = (Parameter,);

    #[allow(clippy::unused_unit)]
    fn into_wasmer(self) -> Self::ImportParameters {
        let hlist_pat![parameter] = self;

        parameter
    }

    fn from_wasmer((parameter,): Self::ExportParameters) -> Self {
        hlist![parameter]
    }
}

macro_rules! parameters {
    (
        $first_name:ident: $first_type:ident,
        $second_name:ident: $second_type:ident,
        $( $names:ident : $types:ident ),*
    ) => {
        parameters!(@generate);
        parameters!($first_name: $first_type, $second_name: $second_type | $( $names: $types ),*);
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
        impl<$( $types ),*> WasmerParameters for HList![$( $types ),*]
        where
            $( $types: FlatType + FromToNativeWasmType, )*
        {
            type ImportParameters = ($( $types, )*);
            type ExportParameters = ($( $types, )*);

            #[allow(clippy::unused_unit)]
            fn into_wasmer(self) -> Self::ImportParameters {
                let hlist_pat![$( $names ),*] = self;

                ($( $names, )*)
            }

            fn from_wasmer(($( $names, )*): Self::ExportParameters) -> Self {
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

impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, Rest> WasmerParameters
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
    type ImportParameters = (i32,);
    type ExportParameters = (i32,);

    fn into_wasmer(self) -> Self::ImportParameters {
        unreachable!("Attempt to convert a list of flat parameters larger than the maximum limit");
    }

    fn from_wasmer(_: Self::ExportParameters) -> Self {
        unreachable!(
            "Attempt to convert into a list of flat parameters larger than the maximum limit"
        );
    }
}
