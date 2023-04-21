#[cfg(feature = "wasmer")]
mod wasmer;
#[cfg(feature = "wasmtime")]
mod wasmtime;

use {
    proc_macro2::{Span, TokenStream},
    proc_macro_error::abort,
    quote::{quote, ToTokens},
    syn::{
        parse_quote, spanned::Spanned, FnArg, Ident, ImplItem, ImplItemFn, ItemImpl, LitStr, Pat,
        PatIdent, PatType, Path, PathArguments, PathSegment, Token, TraitBoundModifier,
        TraitItemFn, Type, TypeParamBound, TypePath, Visibility,
    },
};

#[cfg_attr(not(feature = "wasmtime"), allow(unused_variables))]
pub fn generate(mut implementation: ItemImpl, namespace: &LitStr) -> TokenStream {
    let type_name = type_name(&implementation);
    let reentrant_trait_functions = generate_reentrant_trait_functions(&implementation);

    let wasmer = {
        #[cfg(feature = "wasmer")]
        {
            wasmer::generate(
                &type_name,
                &implementation,
                &reentrant_trait_functions,
                namespace,
            )
        }
        #[cfg(not(feature = "wasmer"))]
        {
            None::<Ident>
        }
    };
    let wasmtime = {
        #[cfg(feature = "wasmtime")]
        {
            wasmtime::generate(
                &type_name,
                &implementation,
                &reentrant_trait_functions,
                namespace,
            )
        }
        #[cfg(not(feature = "wasmtime"))]
        {
            None::<Ident>
        }
    };

    implementation.items.retain(|item| {
        if let ImplItem::Fn(function) = item {
            !is_reentrant_function(&function)
        } else {
            true
        }
    });

    quote! {
        #implementation
        #wasmer
        #wasmtime
    }
}

pub fn type_name(implementation: &ItemImpl) -> &Ident {
    let Type::Path(TypePath { qself: None, path: path_name }) = &*implementation.self_ty
        else {
            abort!(
                implementation.self_ty,
                "`#[wit_export]` must be used on `impl` blocks of non-generic types",
            );
        };

    path_name.get_ident().unwrap_or_else(|| {
        abort!(
            implementation.self_ty,
            "`#[wit_export]` must be used on `impl` blocks of non-generic types",
        );
    })
}

fn generate_reentrant_trait_functions(implementation: &ItemImpl) -> Vec<TokenStream> {
    let generic_type_parameter = Ident::new("Runtime", Span::call_site());

    reentrant_functions(implementation)
        .cloned()
        .map(|function| specialize_reentrant_function(function, &generic_type_parameter, true))
        .collect()
}

fn functions(implementation: &ItemImpl) -> impl Iterator<Item = &ImplItemFn> + Clone {
    implementation.items.iter().map(|item| match item {
        ImplItem::Fn(function) => function,
        ImplItem::Const(const_item) => abort!(
            const_item.ident,
            "Const items are not supported in exported types"
        ),
        ImplItem::Type(type_item) => abort!(
            type_item.ident,
            "Type items are not supported in exported types"
        ),
        ImplItem::Macro(macro_item) => abort!(
            macro_item.mac.path,
            "Macro items are not supported in exported types"
        ),
        ImplItem::Verbatim(_) | _ => {
            abort!(item, "Only function items are supported in exported types")
        }
    })
}

fn reentrant_functions(implementation: &ItemImpl) -> impl Iterator<Item = &ImplItemFn> + Clone {
    functions(implementation).filter(is_reentrant_function)
}

fn is_reentrant_function(function: &&ImplItemFn) -> bool {
    function
        .sig
        .inputs
        .first()
        .map(|first_input| match first_input {
            FnArg::Receiver(_) => false,
            FnArg::Typed(PatType { ty, .. }) => match &**ty {
                Type::ImplTrait(impl_trait) => {
                    impl_trait.bounds.len() >= 1
                        && is_caller_impl_trait(
                            impl_trait
                                .bounds
                                .first()
                                .expect("Missing element from list of size 1"),
                        )
                }
                _ => false,
            },
        })
        .unwrap_or(false)
}

fn is_caller_impl_trait(bound: &TypeParamBound) -> bool {
    let TypeParamBound::Trait(trait_bound) = bound
        else { return false; };

    trait_bound.paren_token.is_none()
        && matches!(trait_bound.modifier, TraitBoundModifier::None)
        && trait_bound.lifetimes.is_none()
        && is_caller_path(&trait_bound.path)
}

fn is_caller_path(path: &Path) -> bool {
    let mut segments = path.segments.iter();

    let path_is_correct = if path.segments.len() == 1 {
        is_path_segment(
            segments.next().expect("Missing path segment"),
            "Caller",
            true,
        )
    } else if path.segments.len() == 2 {
        is_path_segment(
            segments.next().expect("Missing path segment"),
            "witty",
            false,
        ) && is_path_segment(
            segments.next().expect("Missing path segment"),
            "Caller",
            true,
        )
    } else {
        false
    };

    path_is_correct && path.leading_colon.is_none()
}

fn is_path_segment(
    segment: &PathSegment,
    expected_identifier: &str,
    with_type_parameters: bool,
) -> bool {
    let arguments_are_correct = if with_type_parameters {
        matches!(segment.arguments, PathArguments::AngleBracketed(_))
    } else {
        matches!(segment.arguments, PathArguments::None)
    };

    segment.ident == expected_identifier && arguments_are_correct
}

fn specialize_reentrant_function(
    mut function: ImplItemFn,
    new_caller_type: impl ToTokens,
    for_trait: bool,
) -> TokenStream {
    let Some(FnArg::Typed(PatType { pat, ty, .. })) = function.sig.inputs.first_mut()
        else { unreachable!("Attempt to specialize a non-reentrant function") };

    *ty = parse_quote!(#new_caller_type);
    function.vis = Visibility::Inherited;

    if for_trait {
        let Pat::Ident(PatIdent { mutability, .. }) = &mut **pat
            else { abort!(pat.span(), "Caller parameter must be a name") };

        *mutability = None;

        let span = function.sig.span();

        TraitItemFn {
            attrs: vec![],
            sig: function.sig,
            default: None,
            semi_token: Some(Token![;](span)),
        }
        .to_token_stream()
    } else {
        function.to_token_stream()
    }
}

fn reentrancy_constraints(function: &ImplItemFn) -> impl Iterator<Item = &TypeParamBound> + '_ {
    let Some(FnArg::Typed(PatType { ty, .. })) = function.sig.inputs.first()
        else { unreachable!("Attempt to get data type parameter of a non-reentrant function") };
    let Type::ImplTrait(impl_trait) = &**ty
        else { unreachable!("Attempt to get data type parameter of a non-reentrant function") };

    impl_trait.bounds.iter()
}
