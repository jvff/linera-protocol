use {
    super::{
        functions, is_path_segment, is_reentrant_function, reentrancy_constraints,
        reentrant_functions, specialize_reentrant_function,
    },
    heck::ToKebabCase,
    proc_macro2::{Span, TokenStream},
    proc_macro_error::abort,
    quote::{format_ident, quote, quote_spanned, ToTokens},
    std::collections::HashSet,
    syn::{
        spanned::Spanned, AngleBracketedGenericArguments, FnArg, GenericArgument, Ident,
        ImplItemFn, ItemImpl, LitStr, PatType, Path, PathArguments, PathSegment, ReturnType,
        Signature, Token, TraitBound, Type, TypeParamBound, TypePath,
    },
};

pub fn generate(
    type_name: &Ident,
    implementation: &ItemImpl,
    reentrant_function_definitions: &[TokenStream],
    namespace: &LitStr,
) -> TokenStream {
    let reentrant_trait_name = format_ident!("{type_name}ReentrantWithWasmtime");
    let generic_type_parameter = Ident::new("Runtime", Span::call_site());

    let data_type = reentrant_functions(implementation)
        .next()
        .map(data_type_parameter);
    let data_type_constraint = data_type.map(|data_type| quote! { <Data = #data_type> });
    let generic_data_type_parameter = data_type.is_none().then(|| quote! { <Data> });
    let linker_data_type = data_type
        .map(ToTokens::to_token_stream)
        .unwrap_or_else(|| quote! { Data });

    let reentrancy_constraints_set: HashSet<_> = reentrant_functions(implementation)
        .flat_map(reentrancy_constraints)
        .collect();
    let reentrancy_constraints = reentrancy_constraints_set.into_iter();

    let exported_functions = functions(implementation).map(|function| {
        exported_function(
            function,
            namespace,
            &linker_data_type,
            &reentrant_trait_name,
        )
    });

    let reentrant_functions = reentrant_functions(implementation)
        .cloned()
        .map(|function| specialize_reentrant_function(function, &generic_type_parameter, false));

    quote! {
        impl #generic_data_type_parameter
            witty::WitExport<witty::wasmtime::Linker<#linker_data_type>> for #type_name
        {
            fn export(
                linker: &mut witty::wasmtime::Linker<#linker_data_type>,
            ) -> Result<(), witty::RuntimeError> {
                use witty::{
                    wasmtime::{
                        Caller, WasmtimeParameters, WasmtimeRuntime,
                        WasmtimeResults,
                    },
                    ExportFunction, ExportFunctionInterface, HList, Layout, WitLoad, WitStore,
                    WitType, hlist_pat,
                };

                #( #exported_functions )*

                Ok(())
            }
        }

        pub trait #reentrant_trait_name<Runtime> {
            #( #reentrant_function_definitions )*
        }

        impl<Runtime> #reentrant_trait_name<Runtime> for #type_name
        where
            Runtime: witty::wasmtime::WasmtimeRuntime #data_type_constraint
                + witty::RuntimeInstanceWithMemoryScope<Family = witty::Wasmtime>
                #( + #reentrancy_constraints )*,
        {
            #( #reentrant_functions )*
        }
    }
}

fn exported_function(
    function: &ImplItemFn,
    namespace: &LitStr,
    data_type: &TokenStream,
    reentrant_trait: &Ident,
) -> TokenStream {
    let function_name = &function.sig.ident;
    let function_wit_name = function_name.to_string().to_kebab_case();

    let is_reentrant = is_reentrant_function(&function);

    let self_for_call = match is_reentrant {
        true => quote! { <Self as #reentrant_trait<_>> },
        false => quote! { Self },
    };

    let caller_parameters_count = usize::from(is_reentrant);
    let maybe_caller_argument = is_reentrant.then(|| quote! { &mut caller, });

    let parameter_types_and_patterns = parameters(&function.sig).skip(caller_parameters_count);
    let parameters = parameter_types_and_patterns
        .clone()
        .map(|pattern_and_type| &pattern_and_type.pat);
    let parameter_types = parameter_types_and_patterns
        .clone()
        .map(|pattern_and_type| &pattern_and_type.ty);
    let arguments = parameters.clone();

    let (results, is_fallible) = return_type_of(function);
    let interface_type = quote_spanned! { function.sig.span() =>
        (HList![#( #parameter_types ),*], #results)
    };
    let output = quote_spanned! { function.sig.output.span() =>
        <<#interface_type as ExportFunctionInterface>::Output as WasmtimeResults>::Results
    };
    let call_early_return = is_fallible.then_some(Token![?](function.sig.output.span()));

    quote_spanned! { function.span() =>
        linker.export(
            #namespace,
            #function_wit_name,
            |mut caller: Caller<'_, #data_type>, wasmtime_input| -> witty::anyhow::Result<#output> {
                type Interface = #interface_type;

                let input = WasmtimeParameters::from_wasmtime(wasmtime_input);
                let (hlist_pat![#( #parameters ),*], result_storage) =
                    Interface::lift_from_input(input, &caller.memory()?)?;

                let results = #self_for_call::#function_name(
                    #maybe_caller_argument
                    #( #arguments ),*
                ) #call_early_return;
                let output =
                    Interface::lower_results(results, result_storage, &mut caller.memory()?)?;

                Ok(WasmtimeResults::into_wasmtime(output))
            }
        )?;
    }
}

fn parameters(function: &Signature) -> impl Iterator<Item = &PatType> + Clone {
    function.inputs.iter().map(|input| match input {
        FnArg::Typed(parameter) => parameter,
        FnArg::Receiver(receiver) => abort!(
            receiver.self_token,
            "Exported interfaces can not have `self` parameters"
        ),
    })
}

fn return_type_of(function: &ImplItemFn) -> (TokenStream, bool) {
    match &function.sig.output {
        ReturnType::Default => (quote_spanned! { function.sig.output.span() => () }, false),
        ReturnType::Type(_, return_type) => match ok_type_inside_result(&return_type) {
            Some(inner_type) => (inner_type.to_token_stream(), true),
            None => (return_type.to_token_stream(), false),
        },
    }
}

fn ok_type_inside_result(maybe_result_type: &Type) -> Option<&Type> {
    let Type::Path(TypePath { qself: None, path }) = maybe_result_type
        else { return None; };

    let (ok_type, error_type) = result_type_arguments(path)?;

    if let Type::Path(TypePath { qself: None, path }) = error_type {
        if !path.is_ident("RuntimeError") {
            return None;
        }
    } else {
        return None;
    }

    Some(ok_type)
}

fn result_type_arguments(result_path: &Path) -> Option<(&Type, &Type)> {
    let segment_count = result_path.segments.len();

    let is_result = if segment_count == 1 {
        result_path.leading_colon.is_none() && result_path.segments.first()?.ident == "Result"
    } else if result_path.segments.len() == 3 {
        let mut segments = result_path.segments.iter();

        is_path_segment(segments.next()?, "std", false)
            && is_path_segment(segments.next()?, "result", false)
            && is_path_segment(segments.next()?, "Result", true)
    } else {
        false
    };

    if is_result {
        let PathArguments::AngleBracketed(type_arguments) = &result_path.segments.last()?.arguments
            else { return None; };

        if type_arguments.args.len() == 2 {
            let mut arguments = type_arguments.args.iter();
            let GenericArgument::Type(ok_type) = arguments.next()? else { return None; };
            let GenericArgument::Type(error_type) = arguments.next()? else { return None; };

            Some((ok_type, error_type))
        } else {
            None
        }
    } else {
        None
    }
}

fn data_type_parameter(function: &ImplItemFn) -> &Type {
    let Some(FnArg::Typed(PatType { ty, .. })) = function.sig.inputs.first()
        else { unreachable!("Attempt to get data type parameter of a non-reentrant function") };
    let Type::ImplTrait(impl_trait) = &**ty
        else { unreachable!("Attempt to get data type parameter of a non-reentrant function") };
    let Some(TypeParamBound::Trait(TraitBound { path, .. })) = impl_trait.bounds.first()
        else { unreachable!("Attempt to get data type parameter of a non-reentrant function") };
    let Some(PathSegment { arguments: PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }), .. }) = path.segments.last()
        else { unreachable!("Attempt to get data type parameter of a non-reentrant function") };

    args.iter()
        .filter_map(|generic_argument| match generic_argument {
            GenericArgument::Type(generic_type) => Some(generic_type),
            _ => None,
        })
        .next()
        .expect("Missing generic data type parameter")
}
