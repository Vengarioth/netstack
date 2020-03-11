use syn::{spanned::Spanned, Data, DataEnum, DataStruct};
use quote::{quote, quote_spanned};

pub(crate) fn derive_deserialize_impl(input: syn::DeriveInput) -> Result<proc_macro2::TokenStream, syn::Error> {
    match &input.data {
        Data::Struct(s) => derive_struct(&input, s),
        Data::Enum(e) => derive_enum(&input, e),
        Data::Union(u) => Err(syn::Error::new(
            u.union_token.span(),
            "Deserialize implementations cannot be derived from unions",
        )),
    }
}

fn derive_struct(input: &syn::DeriveInput, s: &DataStruct) -> Result<proc_macro2::TokenStream, syn::Error> {
    let derive_type = &input.ident;

    let fields: Vec<proc_macro2::TokenStream> = s.fields.iter()
        .map(|field| {
            let name = &field.ident;
            let ty = &field.ty;
            quote_spanned! { ty.span() => #name: #ty::deserialize(deserializer)? }
        }).collect();

    let result = quote! {
        impl netstack::serialization::Deserialize for #derive_type {
            type Item = #derive_type;

            fn deserialize(deserializer: &mut impl netstack::serialization::Deserializer) -> Result<Self::Item, netstack::serialization::SerializationError> {
                Ok(Self::Item {
                    #(#fields),*
                })
            }
        }
    };

    Ok(result)
}

fn derive_enum(input: &syn::DeriveInput, _s: &DataEnum) -> Result<proc_macro2::TokenStream, syn::Error> {
    dbg!(&input.ident);
    unimplemented!()
}
