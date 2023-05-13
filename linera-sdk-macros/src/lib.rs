use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse::Parser, parse_quote, punctuated::Punctuated, Ident, Path, Token};

/// Procedural macro for application state types.
///
/// Requires a comma separated list of options, for example: `#[application_state(view, graphql)]`.
/// Two options are supported: `graphql` and `view`. At least one of them must be enabled.
///
/// The `view` option derives
/// [`linera_views::RootView`](https://docs.rs/linera-views/latest/linera_views/views/trait.RootView.html)
/// using the
/// [`linera_sdk::views::ViewStorageContext`](https://docs.rs/linera-sdk/latest/linera_sdk/views/type.ViewStorageContext.html)
/// as the only supported context.
///
/// The `graphql` option derives
/// [`linera_views::GraphQLView`](https://docs.rs/linera-views/latest/linera_views/views/derive.GraphQLView.html)
/// using the
/// [`linera_sdk::views::ViewStorageContext`](https://docs.rs/linera-sdk/latest/linera_sdk/views/type.ViewStorageContext.html)
/// as the only supported context.
///
/// # Examples
///
/// Using a custom service handler:
///
/// ```
/// use linera_sdk::{application_state, views::RegisterView};
///
/// #[application_state(view)]
/// pub struct MyApplicationState {
///     data: RegisterView<u8>,
/// }
/// ```
///
/// Using a GraphQL service handler:
///
/// ```
/// use linera_sdk::{application_state, views::RegisterView};
///
/// #[application_state(view, graphql)]
/// pub struct MyApplicationState {
///     data: RegisterView<u8>,
/// }
/// ```
#[proc_macro_attribute]
pub fn application_state(attribute: TokenStream, input: TokenStream) -> TokenStream {
    let parser = Punctuated::<Ident, Token![,]>::parse_terminated;
    let options = parser
        .parse(attribute)
        .expect("Invalid options. Supported options syntax `OPTION [, OPTION]* [,]?`");
    let input = TokenStream2::from(input);

    let derives: Punctuated<Path, Token![,]> = options
        .iter()
        .map(|option| -> Path {
            match option.to_string().as_str() {
                "view" => parse_quote! { linera_sdk::views::RootView },
                "graphql" => parse_quote! { linera_sdk::views::GraphQLView },
                invalid => panic!("Unknown option {invalid}. Supported options:`graphql`, `view`."),
            }
        })
        .collect();

    quote! {
        #[derive(#derives)]
        #[specific_context = "linera_sdk::views::ViewStorageContext"]
        #[crate_path = "linera_sdk::views"]
        #input
    }
    .into()
}
