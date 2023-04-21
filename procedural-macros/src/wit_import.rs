use {
    crate::util::TokensSetItem,
    heck::ToKebabCase,
    proc_macro2::{Span, TokenStream},
    proc_macro_error::abort,
    quote::{quote, quote_spanned, ToTokens},
    std::collections::HashSet,
    syn::{spanned::Spanned, FnArg, Ident, ItemTrait, LitStr, ReturnType, TraitItem, TraitItemFn},
};

pub fn generate(trait_definition: ItemTrait, namespace: &LitStr) -> TokenStream {
    WitImportGenerator::new(&trait_definition, namespace).generate()
}

pub struct WitImportGenerator<'input> {
    trait_name: &'input Ident,
    namespace: &'input LitStr,
    functions: Vec<FunctionInformation<'input>>,
}

struct FunctionInformation<'input> {
    function: &'input TraitItemFn,
    parameter_definitions: TokenStream,
    parameter_bindings: TokenStream,
    return_type: TokenStream,
    interface: TokenStream,
    runtime_constraint: TokenStream,
}

impl<'input> WitImportGenerator<'input> {
    fn new(trait_definition: &'input ItemTrait, namespace: &'input LitStr) -> Self {
        let functions: Vec<_> = trait_definition
            .items
            .iter()
            .map(FunctionInformation::from)
            .collect();

        WitImportGenerator {
            trait_name: &trait_definition.ident,
            namespace,
            functions,
        }
    }

    fn generate(self) -> TokenStream {
        let function_slots = self.function_slots();
        let slot_initializations = self.slot_initializations();
        let imported_functions = self.imported_functions();
        let runtime_constraints = self.runtime_constraints();

        let trait_name = self.trait_name;

        quote! {
            pub struct #trait_name<Runtime>
            #runtime_constraints
            {
                runtime: Runtime,
                #( #function_slots ),*
            }

            impl<Runtime> #trait_name<Runtime>
            #runtime_constraints
            {
                pub fn new(runtime: Runtime) -> Self {
                    #trait_name {
                        runtime,
                        #( #slot_initializations ),*
                    }
                }
            }

            impl<Runtime> #trait_name<Runtime>
            #runtime_constraints
            {
                #( #imported_functions )*
            }
        }
    }

    fn function_slots(&self) -> impl Iterator<Item = TokenStream> + '_ {
        self.functions.iter().map(|function| {
            let function_name = function.name();
            let runtime_constraint = &function.runtime_constraint;

            quote_spanned! { function.span() =>
                #function_name: Option<<Runtime as #runtime_constraint>::Function>
            }
        })
    }

    fn slot_initializations(&self) -> impl Iterator<Item = TokenStream> + '_ {
        self.functions.iter().map(|function| {
            let function_name = function.name();

            quote_spanned! { function.span() =>
                #function_name: None
            }
        })
    }

    fn imported_functions(&self) -> impl Iterator<Item = TokenStream> + '_ {
        self.functions.iter().map(|function| {
            let namespace = self.namespace;

            let function_name = function.name();
            let function_wit_name = function_name.to_string().to_kebab_case();

            let runtime_scope = &function.runtime_constraint;
            let parameters = &function.parameter_definitions;
            let parameter_bindings = &function.parameter_bindings;
            let return_type = &function.return_type;
            let interface = &function.interface;

            quote_spanned! { function.span() =>
                pub fn #function_name(
                    &mut self,
                    #parameters
                ) -> Result<#return_type, witty::RuntimeError>  {
                    use witty::{
                        ImportFunctionInterface, Layout, OptionInsertExt, WitLoad, WitStore,
                        WitType,
                    };

                    let function = self.#function_name.get_or_try_insert_with(|| {
                        <Runtime as #runtime_scope>::load_function(
                            &mut self.runtime,
                            &format!("{}#{}", #namespace, #function_wit_name),
                        )
                    })?;

                    let flat_parameters = #interface::lower_parameters(
                        witty::hlist![#parameter_bindings],
                        &mut self.runtime.memory()?,
                    )?;

                    let flat_response = self.runtime.call(function, flat_parameters)?;

                    let result =
                        #interface::lift_from_output(flat_response, &self.runtime.memory()?)?;

                    Ok(result)
                }
            }
        })
    }

    fn runtime_constraints(&self) -> TokenStream {
        let constraint_set: HashSet<_> = self
            .functions
            .iter()
            .map(|function| TokensSetItem::from(&function.runtime_constraint))
            .collect();

        if constraint_set.is_empty() {
            quote! {}
        } else {
            let constraints = constraint_set.into_iter().fold(
                quote! { witty::RuntimeInstanceWithMemoryScope },
                |list, item| quote! { #list + #item },
            );

            quote! {
                where
                    Runtime: #constraints,
                    <Runtime::Family as witty::Runtime>::Memory: witty::RuntimeMemory<Runtime>,
            }
        }
    }
}

impl<'input> FunctionInformation<'input> {
    pub fn new(function: &'input TraitItemFn) -> Self {
        let (parameter_definitions, parameter_bindings, parameter_types) =
            Self::parse_parameters(function.sig.inputs.iter());

        let return_type = match &function.sig.output {
            ReturnType::Default => quote_spanned! { function.sig.output.span() => () },
            ReturnType::Type(_, return_type) => return_type.to_token_stream(),
        };

        let interface = quote_spanned! { function.sig.span() =>
            <(witty::HList![#parameter_types], #return_type) as witty::ImportFunctionInterface>
        };

        let runtime_constraint = quote_spanned! { function.sig.span() =>
            witty::RuntimeInstanceWithFunctionScope<#interface::Input, #interface::Output>
        };

        FunctionInformation {
            function,
            parameter_definitions,
            parameter_bindings,
            return_type,
            interface,
            runtime_constraint,
        }
    }

    fn parse_parameters(
        function_inputs: impl Iterator<Item = &'input FnArg>,
    ) -> (TokenStream, TokenStream, TokenStream) {
        let parameters = function_inputs.map(|input| match input {
            FnArg::Typed(parameter) => parameter,
            FnArg::Receiver(receiver) => abort!(
                receiver.self_token,
                "Imported interfaces can not have `self` parameters"
            ),
        });

        let mut parameter_definitions = quote! {};
        let mut parameter_bindings = quote! {};
        let mut parameter_types = quote! {};

        for parameter in parameters {
            let parameter_binding = &parameter.pat;
            let parameter_type = &parameter.ty;

            parameter_definitions.extend(quote! { #parameter, });
            parameter_bindings.extend(quote! { #parameter_binding, });
            parameter_types.extend(quote! { #parameter_type, });
        }

        (parameter_definitions, parameter_bindings, parameter_types)
    }

    pub fn name(&self) -> &Ident {
        &self.function.sig.ident
    }

    pub fn span(&self) -> Span {
        self.function.span()
    }
}

impl<'input> From<&'input TraitItem> for FunctionInformation<'input> {
    fn from(item: &'input TraitItem) -> Self {
        match item {
            TraitItem::Fn(function) => FunctionInformation::new(function),
            TraitItem::Const(const_item) => abort!(
                const_item.ident,
                "Const items are not supported in imported traits"
            ),
            TraitItem::Type(type_item) => abort!(
                type_item.ident,
                "Type items are not supported in imported traits"
            ),
            TraitItem::Macro(macro_item) => abort!(
                macro_item.mac.path,
                "Macro items are not supported in imported traits"
            ),
            TraitItem::Verbatim(_) | _ => {
                abort!(item, "Only function items are supported in imported traits")
            }
        }
    }
}
