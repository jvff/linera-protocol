use {
    proc_macro2::TokenStream,
    proc_macro_error::abort,
    quote::{format_ident, quote, ToTokens},
    syn::{Fields, Ident, LitInt, Variant},
};

pub fn derive_for_struct(fields: &Fields) -> TokenStream {
    let field_pairs: Vec<_> = field_names_and_types(fields).collect();

    let load_fields = loads_for_fields(field_pairs.iter().cloned());
    let construction = construction_for_fields(field_pairs.iter().cloned(), &fields);

    let lift_fields = field_pairs.iter().map(|(field_name, field_type)| {
        quote! {
            let (field_layout, flat_layout) = flat_layout.split();
            let #field_name = <#field_type as WitLoad>::lift_from(field_layout, memory)?;
        }
    });

    quote! {
        fn load<Memory>(
            memory: &Memory,
            mut location: witty::GuestPointer,
        ) -> Result<Self, Memory::Error>
        where
            Memory: witty::GuestMemory,
        {
            #( #load_fields )*

            Ok(Self #construction)
        }

        fn lift_from<Memory>(
            flat_layout: <Self::Layout as witty::Layout>::Flat,
            memory: &Memory,
        ) -> Result<Self, Memory::Error>
        where
            Memory: witty::GuestMemory,
        {
            use witty::Split;

            #( #lift_fields )*

            Ok(Self #construction)
        }
    }
}

pub fn derive_for_enum<'variants>(
    name: &Ident,
    variants: impl DoubleEndedIterator<Item = &'variants Variant> + Clone,
) -> TokenStream {
    let variant_count = variants.clone().count();
    let variants = variants.enumerate();
    // let variant_type_lists = variants.map(|variant| variant.fields.iter().map(|field| &field.ty));

    let discriminant_type = if variant_count <= u8::MAX.into() {
        quote! { u8 }
    } else if variant_count <= u16::MAX.into() {
        quote! { u16 }
    } else if variant_count <= u32::MAX as usize {
        quote! { u32 }
    } else {
        abort!(name, "Too many variants in `enum`");
    };

    let load_variants = variants.clone().map(|(index, variant)| {
        let variant_name = &variant.ident;
        let index = LitInt::new(&index.to_string(), variant_name.span());
        let field_pairs = field_names_and_types(&variant.fields);
        let load_fields = loads_for_fields(field_pairs.clone());
        let construction = construction_for_fields(field_pairs, &variant.fields);

        quote! {
            #index => {
                #( #load_fields )*
                Ok(#name::#variant_name #construction)
            }
        }
    });

    let lift_variants = variants.map(|(index, variant)| {
        let variant_name = &variant.ident;
        let index = LitInt::new(&index.to_string(), variant_name.span());
        let field_pairs = field_names_and_types(&variant.fields);
        let field_names = field_pairs.clone().map(|(name, _)| name);
        let field_types = field_pairs.clone().map(|(_, field_type)| field_type);
        let construction = construction_for_fields(field_pairs, &variant.fields);

        quote! {
            #index => {
                let witty::hlist_pat![#( #field_names, )*] =
                    <witty::HList![#( #field_types ),*] as WitLoad>::lift_from(
                        witty::SplitFlatLayouts::split(flat_layout),
                        memory,
                    )?;

                Ok(#name::#variant_name #construction)
            }
        }
    });

    quote! {
        fn load<Memory>(
            memory: &Memory,
            mut location: witty::GuestPointer,
        ) -> Result<Self, Memory::Error>
        where
            Memory: witty::GuestMemory,
        {
            let discriminant = <#discriminant_type as witty::WitLoad>::load(
                memory,
                location.after_padding_for::<#discriminant_type>(),
            )?;
            location = location.after::<#discriminant_type>();

            match discriminant {
                #( #load_variants )*
                _ => Err(witty::InvalidLayoutError.into()),
            }
        }

        fn lift_from<Memory>(
            witty::hlist_pat![discriminant_flat_type, ...flat_layout]:
                <Self::Layout as witty::Layout>::Flat,
            memory: &Memory,
        ) -> Result<Self, Memory::Error>
        where
            Memory: witty::GuestMemory,
        {
            let discriminant = <#discriminant_type as witty::WitLoad>::lift_from(
                witty::hlist![discriminant_flat_type],
                memory,
            )?;

            match discriminant {
                #( #lift_variants )*
                _ => Err(witty::InvalidLayoutError.into()),
            }
        }
    }
}

fn field_names_and_types(
    fields: &Fields,
) -> impl Iterator<Item = (Ident, TokenStream)> + Clone + '_ {
    let field_names = fields.iter().enumerate().map(|(index, field)| {
        field
            .ident
            .as_ref()
            .cloned()
            .unwrap_or_else(|| format_ident!("field{index}"))
    });

    let field_types = fields.iter().map(|field| field.ty.to_token_stream());
    field_names.zip(field_types)
}

fn loads_for_fields(
    field_names_and_types: impl Iterator<Item = (Ident, TokenStream)> + Clone,
) -> impl Iterator<Item = TokenStream> {
    field_names_and_types.map(|(field_name, field_type)| {
        quote! {
            let #field_name = <#field_type as WitLoad>::load(
                memory,
                location.after_padding_for::<#field_type>(),
            )?;
            location = location.after::<#field_type>();
        }
    })
}

fn construction_for_fields(
    field_names_and_types: impl Iterator<Item = (Ident, TokenStream)>,
    fields: &Fields,
) -> TokenStream {
    let field_names = field_names_and_types.map(|(name, _)| name);

    match fields {
        Fields::Unit => quote! {},
        Fields::Named(_) => quote! { { #( #field_names ),* } },
        Fields::Unnamed(_) => quote! {( #( #field_names ),* ) },
    }
}
