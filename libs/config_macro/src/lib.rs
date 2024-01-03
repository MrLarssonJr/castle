use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Meta};

#[proc_macro_derive(Config, attributes(env))]
pub fn derive_config(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	inner_derive_config(input).into()
}

fn inner_derive_config(input: DeriveInput) -> TokenStream {
	let Data::Struct(data) = input.data else {
		return quote_spanned!(input.span()=> compile_error!("expected struct"););
	};

	let fields = data.fields;
	let Fields::Named(fields) = fields else {
		return quote_spanned!( fields.span()=> compile_error!("expected named fields"););
	};

	let mut field_constructors = Vec::new();
	for field in fields.named {
		let ident = field
			.ident
			.as_ref()
			.expect("fields in FieldsNamed ought to have identifier");

		let mut env_var = None;
		for attribute in &field.attrs {
			let value = match &attribute.meta {
				Meta::Path(p) if p.is_ident("env") => {
					return quote_spanned!( attribute.span()=> compile_error!("expected attribute env to be follow name value form (e.g. #[env = \"FOO\"])"););
				}

				Meta::List(l) if l.path.is_ident("env") => {
					return quote_spanned!( attribute.span()=> compile_error!("expected attribute env to be follow name value form (e.g. #[env = \"FOO\"])"););
				}

				Meta::NameValue(n) if n.path.is_ident("env") => &n.value,

				_ => continue,
			};

			if let Some(_) = env_var.replace(value) {
				return quote_spanned!( attribute.span()=> compile_error!("duplicate env attribute for field"););
			}
		}
		let Some(env_var) = env_var else {
			return quote_spanned!( field.span()=> compile_error!("no env attribute for field"););
		};

		let ty = &field.ty;

		let field_constructor = quote! {
			#ident: ::std::env::var_os(#env_var)
				.ok_or(::config::ArgumentParseError::Missing { name: #env_var })
				.and_then(|arg| {
					arg.into_string()
						.map_err(|actual| ::config::ArgumentParseError::NotUnicode {
							name: #env_var,
							actual,
						})
				})
				.and_then(|arg| {
					<#ty as ::config::FromArg>::parse_arg(arg.as_str()).map_err(|err| {
						::config::ArgumentParseError::NotParseable {
							name: #env_var,
							ty: ::std::any::type_name::<#ty>(),
							source: Box::new(err) as Box<dyn ::std::error::Error>,
						}
					})
				})?
		};

		field_constructors.push(field_constructor);
	}

	let name = input.ident;

	quote! {
		impl ::config::FromConfig for #name {
			fn parse() -> Result<Self, ::config::ArgumentParseError> {
				Ok(#name {
					#(#field_constructors,)*
				})
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn basic() {
		// Arrange
		let input = quote! {
			#[derive(Config)]
			struct Foo {
				#[env = "BAR"]
				bar: u32
			}
		};
		let input = syn::parse2::<DeriveInput>(input).expect("input should be valid DeriveInput");

		let expected = quote! {
			impl ::config::FromConfig for Foo {
				fn parse() -> Result<Self, ::config::ArgumentParseError> {
					Ok(
						Foo {
							bar: ::std::env::var_os("BAR")
								.ok_or(::config::ArgumentParseError::Missing { name: "BAR" })
								.and_then(|arg| {
									arg.into_string()
										.map_err(|actual| ::config::ArgumentParseError::NotUnicode {
											name: "BAR",
											actual,
										})
								})
								.and_then(|arg| {
									<u32 as ::config::FromArg>::parse_arg(arg.as_str()).map_err(|err| {
										::config::ArgumentParseError::NotParseable {
											name: "BAR",
											ty: ::std::any::type_name::<u32>(),
											source: Box::new(err) as Box<dyn ::std::error::Error>,
										}
									})
								})?,
						}
					)
				}
			}
		}
		.to_string();

		// Act
		let actual = inner_derive_config(input).to_string();

		// Assert
		assert_eq!(expected, actual);
	}
}
