use {
    super::{InstanceAndData, StoreRuntime, WasmerParameters, WasmerResults},
    crate::{
        flat_type::FlatType,
        runtime::{Runtime, RuntimeError, RuntimeInstanceWithFunctionScope},
        FlatLayout,
    },
    frunk::{hlist_pat, HList},
    wasmer::{AsStoreRef, Extern, FromToNativeWasmType, FunctionEnvMut, TypedFunction},
};

macro_rules! function_scope {
    ($( $names:ident : $types:ident ),*) => {
        function_scope!(| $( $names: $types ),*);
    };

    ($( $names:ident : $types:ident ),* |) => {
        function_scope!(@generate $( $names: $types ),*);
    };

    (
        $( $names_to_generate:ident : $types_to_generate:ident ),* |
        $next_name:ident : $next_type:ident $( , $queued_names:ident : $queued_types:ident )*
    ) => {
        function_scope!(@generate $( $names_to_generate: $types_to_generate ),*);
        function_scope!(
            $( $names_to_generate: $types_to_generate, )*
            $next_name: $next_type | $( $queued_names: $queued_types ),*
        );
    };

    (@generate $( $names:ident : $types:ident ),*) => {
        function_scope!(@generate::<Data>(StoreRuntime<Data>) $( $names: $types ),*);
        function_scope!(
            @generate::<Data>(FunctionEnvMut<'_, InstanceAndData<Data>>) $( $names: $types ),*
        );
        function_scope!(
            @generate::<Data>(&mut FunctionEnvMut<'_, InstanceAndData<Data>>) $( $names: $types ),*
        );
    };

    (@generate::<$data:ident>($runtime:ty) $( $names:ident : $types:ident ),*) => {
        impl<$data, $( $types, )* Results>
            RuntimeInstanceWithFunctionScope<HList![$( $types ),*], Results>
            for $runtime
        where
            $data: Send + 'static,
            $( $types: FlatType + FromToNativeWasmType, )*
            Results: FlatLayout + WasmerResults,
        {
            type Function = TypedFunction<
                <HList![$( $types ),*] as WasmerParameters>::ImportParameters,
                <Results as WasmerResults>::Results,
            >;

            fn function_from_export(
                &mut self,
                export: <Self::Family as Runtime>::Export,
            ) -> Result<Option<Self::Function>, RuntimeError> {
                Ok(match export {
                    Extern::Function(function) => Some(function.typed(&self.as_store_ref())?),
                    _ => None,
                })
            }

            fn call(
                &mut self,
                function: &Self::Function,
                hlist_pat![$( $names ),*]: HList![$( $types ),*],
            ) -> Result<Results, RuntimeError> {
                let results = function.call(&mut *self, $( $names ),*)?;

                Ok(Results::from_wasmer(results))
            }
        }
    };
}

function_scope!(
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
