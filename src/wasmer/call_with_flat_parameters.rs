use {
    super::{Error, WasmerParameters, WasmerResults, WasmerRuntime},
    crate::flat_type::FlatType,
    frunk::{hlist_pat, HList},
    wasmer::{FromToNativeWasmType, TypedFunction},
};

pub trait CallWithFlatParameters<Runtime, Results>
where
    Results: WasmerResults,
{
    type FlatParameters: WasmerParameters;

    fn call_with_flat_parameters(
        &self,
        runtime: &mut Runtime,
        parameters: Self::FlatParameters,
    ) -> Result<Results::Flat, Error>;
}

impl<Runtime, Parameter, Results> CallWithFlatParameters<Runtime, Results>
    for TypedFunction<Parameter, Results::Results>
where
    Runtime: WasmerRuntime,
    Parameter: FlatType + FromToNativeWasmType,
    Results: WasmerResults,
{
    type FlatParameters = HList![Parameter];

    fn call_with_flat_parameters(
        &self,
        runtime: &mut Runtime,
        hlist_pat![parameter]: Self::FlatParameters,
    ) -> Result<Results::Flat, Error> {
        let result = self.call(runtime, parameter)?;

        Ok(Results::from_wasmer(result))
    }
}

macro_rules! impl_wrapped_call {
    (
        $first_name:ident: $first_type:ident,
        $second_name:ident: $second_type:ident,
        $( $names:ident: $types:ident ),* $(,)*
    ) => {
        impl_wrapped_call!(@generate);
        impl_wrapped_call!(
            $first_name: $first_type, $second_name: $second_type | $( $names: $types ),*
        );
    };

    ($( $names:ident: $types:ident ),* |) => {
        impl_wrapped_call!(@generate $( $names: $types ),*);
    };

    (
        $( $names:ident: $types:ident ),*
        | $next_name:ident: $next_type:ident
        $(, $queued_names:ident: $queued_types:ident )*
    ) => {
        impl_wrapped_call!(@generate $( $names: $types ),*);
        impl_wrapped_call!(
            $( $names: $types, )* $next_name: $next_type
            | $( $queued_names: $queued_types ),*
        );
    };

    (@generate $( $names:ident : $types:ident ),*) => {
        impl<Runtime, $( $types, )* Results> CallWithFlatParameters<Runtime, Results> for
            TypedFunction<( $( $types, )* ), Results::Results>
        where
            Runtime: WasmerRuntime,
            $( $types: FlatType + FromToNativeWasmType, )*
            Results: WasmerResults,
        {
            type FlatParameters = HList![ $( $types ),* ];

            fn call_with_flat_parameters(
                &self,
                runtime: &mut Runtime,
                hlist_pat![$( $names ),*]: Self::FlatParameters,
            ) -> Result<Results::Flat, Error> {
                let result = self.call(runtime, $( $names ),*)?;

                Ok(Results::from_wasmer(result))
            }
        }
    }
}

impl_wrapped_call!(
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
