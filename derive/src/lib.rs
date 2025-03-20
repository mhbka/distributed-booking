use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// Derives `Byteable` for a struct whose fields are all `Byteable`.
#[proc_macro_derive(ByteableDerive)]
pub fn derive_byteable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = if let syn::Data::Struct(data) = &input.data {
        &data.fields
    } else {
        panic!("Byteable can only be derived for structs");
    };

    let field_names: Vec<_> = fields.iter().map(|f| f.ident.as_ref().unwrap()).collect();

    let from_bytes_fields = field_names.iter().map(|name| {
        quote! {
            let #name = Byteable::from_bytes(data)?;
        }
    });

    let to_bytes_fields = field_names.iter().map(|name| {
        quote! {
            bytes.extend(self.#name.to_bytes());
        }
    });

    let expanded = quote! {
        impl Byteable for #name {
            fn from_bytes(data: &mut Vec<u8>) -> Result<Self, String> {
                #(#from_bytes_fields)*

                Ok(Self {
                    #(#field_names),*
                })
            }

            fn to_bytes(self) -> Vec<u8> {
                let mut bytes = Vec::new();
                #(#to_bytes_fields)*
                bytes
            }
        }
    };

    TokenStream::from(expanded)
}
