use {
    proc_macro2::TokenStream,
    proc_macro_error::abort,
    quote::{format_ident, quote, ToTokens},
    syn::{Fields, Ident, Index, LitInt, Type, Variant},
};

pub fn derive_for_struct(fields: &Fields) -> TokenStream {
    let field_names = field_names(fields);
    let field_bindings = field_bindings(fields);
    let field_types = fields.iter().map(|field| &field.ty);
    let field_pairs = field_bindings.clone().zip(field_types);

    let store_fields = field_pairs.map(store_field);

    let lower_fields = field_names.clone().map(|field_name| {
        quote! {
            let field_layout = WitStore::lower(&self.#field_name, memory)?;
            let flat_layout = flat_layout + field_layout;
        }
    });

    let construction = match fields {
        Fields::Unit => quote! {},
        Fields::Named(_) => quote! { { #( #field_bindings ),* } },
        Fields::Unnamed(_) => quote! { ( #( #field_bindings ),* ) },
    };

    quote! {
        fn store<Memory>(
            &self,
            memory: &mut Memory,
            mut location: witty::GuestPointer,
        ) -> Result<(), Memory::Error>
        where
            Memory: witty::GuestMemory,
        {
            let Self #construction = self;

            #( #store_fields )*

            Ok(())
        }

        fn lower<Memory>(
            &self,
            memory: &mut Memory,
        ) -> Result<<Self::Layout as witty::Layout>::Flat, Memory::Error>
        where
            Memory: witty::GuestMemory,
        {
            use witty::FlatLayout;

            let flat_layout = witty::HList![];

            #( #lower_fields )*

            Ok(flat_layout)
        }
    }
}

pub fn derive_for_enum<'variants>(
    name: &Ident,
    variants: impl DoubleEndedIterator<Item = &'variants Variant> + Clone,
) -> TokenStream {
    let variant_count = variants.clone().count();

    let discriminant_type = if variant_count <= u8::MAX.into() {
        quote! { u8 }
    } else if variant_count <= u16::MAX.into() {
        quote! { u16 }
    } else if variant_count <= u32::MAX as usize {
        quote! { u32 }
    } else {
        abort!(name, "Too many variants in `enum`");
    };

    let store_variants = variants.clone().enumerate().map(|(index, variant)| {
        let variant_name = &variant.ident;
        let discriminant =
            LitInt::new(&format!("{index}_{discriminant_type}"), variant_name.span());

        let field_bindings = field_bindings(&variant.fields);
        let field_types = variant.fields.iter().map(|field| &field.ty);
        let field_pairs = field_bindings.clone().zip(field_types);

        let store_fields = field_pairs.map(store_field);

        let pattern = match variant.fields {
            Fields::Unit => quote! {},
            Fields::Named(_) => quote! { { #( #field_bindings ),* } },
            Fields::Unnamed(_) => quote! { ( #( #field_bindings ),* ) },
        };

        quote! {
            #name::#variant_name #pattern => {
                #discriminant.store(memory, location.after_padding_for::<#discriminant_type>())?;
                location = location.after::<#discriminant_type>();

                #( #store_fields )*

                Ok(())
            }
        }
    });

    let lower_variants = variants.clone().enumerate().map(|(index, variant)| {
        let variant_name = &variant.ident;
        let discriminant =
            LitInt::new(&format!("{index}_{discriminant_type}"), variant_name.span());

        let field_bindings = field_bindings(&variant.fields);

        let pattern = {
            let field_bindings = field_bindings.clone();
            match variant.fields {
                Fields::Unit => quote! {},
                Fields::Named(_) => quote! { { #( #field_bindings ),* } },
                Fields::Unnamed(_) => quote! { ( #( #field_bindings ),* ) },
            }
        };

        quote! {
            #name::#variant_name #pattern => {
                let variant_flat_layout = witty::hlist![#(#field_bindings),*].lower(memory)?;

                let flat_layout: <Self::Layout as witty::Layout>::Flat =
                    witty::JoinFlatLayouts::join(
                        #discriminant.lower(memory)? + variant_flat_layout,
                    );

                Ok(flat_layout)
            }
        }
    });

    quote! {
        fn store<Memory>(
            &self,
            memory: &mut Memory,
            mut location: witty::GuestPointer,
        ) -> Result<(), Memory::Error>
        where
            Memory: witty::GuestMemory,
        {
            match self {
                #( #store_variants )*
            }
        }

        fn lower<Memory>(
            &self,
            memory: &mut Memory,
        ) -> Result<<Self::Layout as witty::Layout>::Flat, Memory::Error>
        where
            Memory: witty::GuestMemory,
        {
            match self {
                #( #lower_variants )*
            }
        }
    }
}

fn field_names(fields: &Fields) -> impl Iterator<Item = TokenStream> + Clone + '_ {
    fields.iter().enumerate().map(|(index, field)| {
        field
            .ident
            .as_ref()
            .map(ToTokens::to_token_stream)
            .unwrap_or_else(|| Index::from(index).to_token_stream())
    })
}

fn field_bindings(fields: &Fields) -> impl Iterator<Item = Ident> + Clone + '_ {
    fields.iter().enumerate().map(|(index, field)| {
        field
            .ident
            .as_ref()
            .cloned()
            .unwrap_or_else(|| format_ident!("field{index}"))
    })
}

fn store_field((field_name, field_type): (Ident, &Type)) -> TokenStream {
    quote! {
        WitStore::store(#field_name, memory, location.after_padding_for::<#field_type>())?;
        location = location.after::<#field_type>();
    }
}
