use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Expr, Fields, Meta};

#[proc_macro_derive(Config, attributes(env, env_file))]
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

		let ty = &field.ty;

		let env_var = match find_name_value_attribute_value(&field.attrs, "env") {
			Err(err) => return err,
			Ok(v) => v,
		};

		let env_file_var = match find_name_value_attribute_value(&field.attrs, "env_file") {
			Err(err) => return err,
			Ok(v) => v,
		};

		let field_constructor = match (env_var, env_file_var) {
			(None, None) => {
				return quote_spanned!( field.span()=> compile_error!("no env nor env_file attribute for field");)
			}

			(Some(_), Some(_)) => {
				return quote_spanned!( field.span()=> compile_error!("both env and env_file attribute for field");)
			}

			(Some(env_var), None) => quote! {
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
								source: ::std::boxed::Box::new(err) as ::std::boxed::Box<dyn ::std::error::Error>,
							}
						})
					})?
			},

			(None, Some(env_file_var)) => quote! {
				#ident: ::std::env::var_os(#env_file_var)
					.ok_or(::config::ArgumentParseError::Missing { name: #env_file_var })
					.and_then(|arg| {
						arg.into_string()
							.map_err(|actual| ::config::ArgumentParseError::NotUnicode {
								name: #env_file_var,
								actual,
							})
					})
					.and_then(|path| {
						let file = ::std::fs::File::open(&path).map_err(|err| {
							::config::ArgumentParseError::NotAccessible {
								name: #env_file_var,
								path: path.clone(),
								source: ::std::boxed::Box::new(err)
									as ::std::boxed::Box<dyn ::std::error::Error>,
							}
						})?;

						let res = ::std::io::read_to_string(file).map_err(|err| {
							::config::ArgumentParseError::NotReadable {
								name: #env_file_var,
								path,
								source: ::std::boxed::Box::new(err)
									as ::std::boxed::Box<dyn ::std::error::Error>,
							}
						})?;

						Ok(res)
					})
					.and_then(|arg| {
						<#ty as ::config::FromArg>::parse_arg(arg.as_str().trim()).map_err(|err| {
							::config::ArgumentParseError::NotParseable {
								name: #env_file_var,
								ty: ::std::any::type_name::<#ty>(),
								source: ::std::boxed::Box::new(err)
									as ::std::boxed::Box<dyn ::std::error::Error>,
							}
						})
					})?
			},
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

fn find_name_value_attribute_value<'a>(
	attributes: &'a [Attribute],
	ident: &str,
) -> Result<Option<&'a Expr>, TokenStream> {
	let mut res = None;
	for attribute in attributes {
		let value = match &attribute.meta {
			Meta::Path(p) if p.is_ident(ident) => {
				return Err(
					quote_spanned!( attribute.span()=> compile_error!(concat!("expected attribute ", #ident, " to be follow name value form (e.g. #[", #ident, " = \"FOO\"])"));),
				);
			}

			Meta::List(l) if l.path.is_ident(ident) => {
				return Err(
					quote_spanned!( attribute.span()=> compile_error!(concat!("expected attribute ", #ident, " to be follow name value form (e.g. #[", #ident, " = \"FOO\"])"));),
				);
			}

			Meta::NameValue(n) if n.path.is_ident(ident) => &n.value,

			_ => continue,
		};

		if let Some(_) = res.replace(value) {
			return Err(
				quote_spanned!( attribute.span()=> compile_error!(concat!("duplicate ", #ident, " attribute for field"));),
			);
		}
	}

	Ok(res)
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
