use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(Config)]
pub fn derive_config(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	let Data::Struct(data) = input.data else {
		return quote_spanned!(input.span() => compile_error!("expected struct")).into();
	};

	let fields = data.fields;
	let Fields::Named(fields) = fields else {
		return quote_spanned!( fields.span() => compile_error!("expected named fields")).into();
	};

	let name = input.ident;

	let res = quote! {
		impl ::config::FromConfig for #name {
			type Error = ();

			fn parse() -> Result<Self, Self::Error> {
				todo!()
			}
		}
	};

	res.into()
}
