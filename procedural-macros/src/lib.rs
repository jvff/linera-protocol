mod util;
mod wit_export;
mod wit_import;
mod wit_load;
mod wit_store;
mod wit_type;

use {
    heck::ToKebabCase,
    proc_macro::TokenStream,
    proc_macro2::Span,
    proc_macro_error::{abort, proc_macro_error},
    quote::{quote, ToTokens},
    syn::{
        parse::{self, Parse, ParseStream},
        parse_macro_input,
        punctuated::Punctuated,
        Data, DeriveInput, Expr, ExprLit, Ident, ItemImpl, ItemTrait, Lit, LitBool, LitStr,
        MetaNameValue, Token,
    },
};

#[proc_macro_error]
#[proc_macro_derive(WitType)]
pub fn derive_wit_type(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let body = match &input.data {
        Data::Struct(struct_item) => wit_type::derive_for_struct(&struct_item.fields),
        Data::Enum(enum_item) => wit_type::derive_for_enum(&input.ident, enum_item.variants.iter()),
        Data::Union(_union_item) => {
            abort!(input.ident, "Can't derive `WitType` for `union`s")
        }
    };

    derive_trait(input, body, Ident::new("WitType", Span::call_site()))
}

#[proc_macro_error]
#[proc_macro_derive(WitLoad)]
pub fn derive_wit_load(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let body = match &input.data {
        Data::Struct(struct_item) => wit_load::derive_for_struct(&struct_item.fields),
        Data::Enum(enum_item) => wit_load::derive_for_enum(&input.ident, enum_item.variants.iter()),
        Data::Union(_union_item) => {
            abort!(input.ident, "Can't derive `WitLoad` for `union`s")
        }
    };

    derive_trait(input, body, Ident::new("WitLoad", Span::call_site()))
}

#[proc_macro_error]
#[proc_macro_derive(WitStore)]
pub fn derive_wit_store(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let body = match &input.data {
        Data::Struct(struct_item) => wit_store::derive_for_struct(&struct_item.fields),
        Data::Enum(enum_item) => {
            wit_store::derive_for_enum(&input.ident, enum_item.variants.iter())
        }
        Data::Union(_union_item) => {
            abort!(input.ident, "Can't derive `WitStore` for `union`s")
        }
    };

    derive_trait(input, body, Ident::new("WitStore", Span::call_site()))
}

fn derive_trait(input: DeriveInput, body: impl ToTokens, trait_name: Ident) -> TokenStream {
    let (generic_params, type_generics, where_clause) = input.generics.split_for_impl();
    let type_name = &input.ident;

    quote! {
        impl #generic_params #trait_name for #type_name #type_generics #where_clause {
            #body
        }
    }
    .into()
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn wit_import(attribute: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemTrait);
    let namespace = prepare_namespace(attribute, &input.ident);

    wit_import::generate(input, &namespace).into()
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn wit_export(attribute: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemImpl);
    let namespace = prepare_namespace(attribute, wit_export::type_name(&input));

    wit_export::generate(input, &namespace).into()
}

struct WitAttribute {
    metadata: Punctuated<MetaNameValue, Token![,]>,
}

impl Parse for WitAttribute {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        Ok(WitAttribute {
            metadata: Punctuated::parse_terminated(input)?,
        })
    }
}

fn prepare_namespace(attribute: TokenStream, type_name: &Ident) -> LitStr {
    let span = Span::call_site();
    let input = syn::parse::<WitAttribute>(attribute)
        .unwrap_or_else(|_| {
            abort!(
                span,
                r#"Failed to parse attribute parameters, expected either `root = true` \
                or `package = "namespace:package""#
            )
        })
        .metadata;

    if input.len() == 1 && matches!(input.first(), Some(pair) if pair.path.is_ident("root")) {
        let is_properly_specified = matches!(
            input.first(),
            Some(MetaNameValue {
                value: Expr::Lit(ExprLit {
                    lit: Lit::Bool(LitBool { value: true, .. }),
                    ..
                }),
                ..
            })
        );

        if !is_properly_specified {
            abort!(span, "Expected `root = true`");
        }

        return LitStr::new("$root", Span::call_site());
    }

    let package_name = input
        .iter()
        .find(|pair| pair.path.is_ident("package"))
        .map(|pair| extract_string_literal(&pair.value))
        .unwrap_or_else(|| {
            abort!(
                span,
                r#"Missing package name specifier in attribute parameters \
                (package = "namespace:package")"#
            )
        });

    let interface_name = input
        .iter()
        .find(|pair| pair.path.is_ident("interface"))
        .map(|pair| extract_string_literal(&pair.value))
        .unwrap_or_else(|| type_name.to_string().to_kebab_case());

    LitStr::new(&format!("{package_name}/{interface_name}"), span)
}

fn extract_string_literal(expression: &Expr) -> String {
    let Expr::Lit(ExprLit {
        lit: Lit::Str(lit_str),
        ..
    }) = expression
        else { abort!(expression, "Expected a string literal"); };

    lit_str.value()
}
