extern crate proc_macro;

use darling::{FromDeriveInput, FromField};
use proc_macro::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, Data, DeriveInput, GenericArgument, PathArguments, Type};

#[derive(FromDeriveInput)]
#[darling(attributes(from_env), supports(struct_named))]
struct EnvStruct {
    #[darling(default)]
    prefix: Option<String>,
}

#[derive(FromField)]
#[darling(attributes(from_env))]
struct EnvField {
    ident: Option<syn::Ident>,
    ty: syn::Type,
    #[darling(default)]
    default: Option<syn::Lit>,
    #[darling(default)]
    no_prefix: bool,
    #[darling(default)]
    var: Option<syn::Lit>,
    #[darling(default)]
    rename: Option<String>,
    #[darling(default)]
    flatten: bool,
}

#[proc_macro_derive(FromEnv, attributes(from_env))]
pub fn from_env_proc_macro(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::DeriveInput);
    match from_env_proc_macro_impl(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.into_compile_error().into(),
    }
}

fn from_env_proc_macro_impl(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let struct_identifier = &input.ident;

    // Parse struct-level attributes
    let env_struct = EnvStruct::from_derive_input(input)
        .map_err(|e| syn::Error::new(input.ident.span(), e.to_string()))?;
    let struct_prefix = env_struct.prefix;

    match &input.data {
        Data::Struct(syn::DataStruct { fields, .. }) => {
            let mut env_fields = Vec::new();
            for field in fields.iter() {
                let env_field = EnvField::from_field(field)
                    .map_err(|e| syn::Error::new(field.span(), e.to_string()))?;
                if env_field.ident.is_none() {
                    return Err(syn::Error::new(
                        field.span(),
                        "FromEnv does not support tuple structs; use named fields instead",
                    ));
                }
                env_fields.push(env_field);
            }

            let field_identifiers: Vec<_> = env_fields
                .iter()
                .map(|f| f.ident.as_ref().unwrap())
                .collect();
            let field_loaders_with_prefix: Vec<_> =
                env_fields.iter().map(generate_field_loader).collect();

            // If struct has a prefix, from_env() uses it; otherwise no prefix
            // from_env_with_prefix combines incoming prefix with struct's own prefix
            let (from_env_impl, prefix_setup) = if let Some(ref prefix) = struct_prefix {
                // Strip trailing underscore if present for consistent formatting
                let struct_prefix = prefix.trim_end_matches('_');
                (
                    quote! {
                        fn from_env() -> Result<Self, ::derive_from_env::FromEnvError> {
                            Self::from_env_with_prefix("")
                        }
                    },
                    quote! {
                        let prefix = if prefix.is_empty() {
                            #struct_prefix.to_string()
                        } else {
                            format!("{}_{}", prefix, #struct_prefix)
                        };
                        let prefix = prefix.as_str();
                    },
                )
            } else {
                (
                    quote! {
                        fn from_env() -> Result<Self, ::derive_from_env::FromEnvError> {
                            Self::from_env_with_prefix("")
                        }
                    },
                    quote! {},
                )
            };

            Ok(quote! {
                impl ::derive_from_env::_inner_trait::FromEnv for #struct_identifier {
                    #from_env_impl
                    fn from_env_with_prefix(prefix: &str) -> Result<Self, ::derive_from_env::FromEnvError> {
                        use std::str::FromStr;
                        #prefix_setup
                        Ok(Self {
                            #(
                                #field_identifiers: #field_loaders_with_prefix
                            ),*
                        })
                    }
                }
                impl #struct_identifier {
                    pub fn from_env() -> Result<Self, ::derive_from_env::FromEnvError> {
                        <Self as ::derive_from_env::_inner_trait::FromEnv>::from_env()
                    }
                    pub fn from_env_with_prefix(prefix: &str) -> Result<Self, ::derive_from_env::FromEnvError> {
                        <Self as ::derive_from_env::_inner_trait::FromEnv>::from_env_with_prefix(prefix)
                    }
                }
            })
        }
        Data::Enum(_) => Err(syn::Error::new(
            input.ident.span(),
            "FromEnv can only be derived for structs, not enums",
        )),
        Data::Union(_) => Err(syn::Error::new(
            input.ident.span(),
            "FromEnv can only be derived for structs, not unions",
        )),
    }
}

fn extract_inner_type_if_option(ty: &Type) -> Option<&Type> {
    if let Type::Path(type_path) = ty {
        if type_path.qself.is_none() && type_path.path.segments.len() == 1 {
            let segment = &type_path.path.segments[0];
            if segment.ident == "Option" {
                if let PathArguments::AngleBracketed(ref args) = segment.arguments {
                    if args.args.len() == 1 {
                        if let GenericArgument::Type(ref inner_type) = args.args[0] {
                            return Some(inner_type);
                        }
                    }
                }
            }
        }
    }
    None
}

fn generate_field_loader(field: &EnvField) -> proc_macro2::TokenStream {
    let field_name = field.ident.as_ref().unwrap().to_string();
    let field_type = &field.ty;
    let inner_field_type = extract_inner_type_if_option(field_type);
    let default_value = &field.default;
    let no_prefix = field.no_prefix;
    let flatten = field.flatten;
    let var_name = &field.var;
    let rename = &field.rename;

    // Use rename if provided, otherwise use field name
    let name_part = rename.as_ref().unwrap_or(&field_name);

    // Build env var name: if prefix is empty, just use name; otherwise PREFIX_NAME
    let env_var_name = quote! {
        if prefix.is_empty() {
            #name_part.to_uppercase()
        } else {
            format!("{}_{}", prefix, #name_part.to_uppercase())
        }
    };

    // Handle flatten (nested structs)
    if flatten {
        if default_value.is_some() {
            panic!("default is not supported for flatten fields");
        }
        if var_name.is_some() {
            panic!("var is not supported for flatten fields");
        }
        if no_prefix {
            // no_prefix: pass current prefix unchanged (don't add field name)
            quote! {
                <#field_type as ::derive_from_env::_inner_trait::FromEnv>::from_env_with_prefix(prefix)?
            }
        } else {
            // Normal: add field name to prefix chain
            quote! {
                <#field_type as ::derive_from_env::_inner_trait::FromEnv>::from_env_with_prefix(&#env_var_name)?
            }
        }
    } else if let Some(inner_type) = inner_field_type {
        // Option<T> field
        if default_value.is_some() {
            panic!("default is not supported for Option fields");
        }
        if let Some(var_name) = var_name {
            quote! {
                match std::env::var(#var_name.to_string()) {
                    Ok(s) => Some(#inner_type::from_str(&s).map_err(|_| {
                        ::derive_from_env::FromEnvError::ParsingFailure {
                            var_name: #var_name.to_string(),
                            expected_type: stringify!(#inner_type).to_string(),
                        }
                    })?),
                    Err(_) => None,
                }
            }
        } else {
            quote! {
                match std::env::var(#env_var_name) {
                    Ok(s) => Some(#inner_type::from_str(&s).map_err(|_| {
                        ::derive_from_env::FromEnvError::ParsingFailure {
                            var_name: #env_var_name.to_string(),
                            expected_type: stringify!(#inner_type).to_string(),
                        }
                    })?),
                    Err(_) => None,
                }
            }
        }
    } else {
        // Regular FromStr field
        match (default_value, var_name) {
            (Some(default), Some(var_name)) => {
                quote! {
                    {
                        let __env_val = std::env::var(#var_name.to_string())
                            .unwrap_or_else(|_| #default.to_string());
                        #field_type::from_str(&__env_val).map_err(|_| {
                            ::derive_from_env::FromEnvError::ParsingFailure {
                                var_name: #var_name.to_string(),
                                expected_type: stringify!(#field_type).to_string(),
                            }
                        })?
                    }
                }
            }
            (Some(default), None) => {
                quote! {
                    {
                        let __env_val = std::env::var(#env_var_name)
                            .unwrap_or_else(|_| #default.to_string());
                        #field_type::from_str(&__env_val).map_err(|_| {
                            ::derive_from_env::FromEnvError::ParsingFailure {
                                var_name: #env_var_name.to_string(),
                                expected_type: stringify!(#field_type).to_string(),
                            }
                        })?
                    }
                }
            }
            (None, Some(var_name)) => {
                quote! {
                    {
                        let __env_val = std::env::var(#var_name.to_string())
                            .map_err(|_| ::derive_from_env::FromEnvError::MissingEnvVar {
                                var_name: #var_name.to_string(),
                            })?;
                        #field_type::from_str(&__env_val).map_err(|_| {
                            ::derive_from_env::FromEnvError::ParsingFailure {
                                var_name: #var_name.to_string(),
                                expected_type: stringify!(#field_type).to_string(),
                            }
                        })?
                    }
                }
            }
            (None, None) => {
                quote! {
                    {
                        let __env_val = std::env::var(#env_var_name)
                            .map_err(|_| ::derive_from_env::FromEnvError::MissingEnvVar {
                                var_name: #env_var_name.to_string(),
                            })?;
                        #field_type::from_str(&__env_val).map_err(|_| {
                            ::derive_from_env::FromEnvError::ParsingFailure {
                                var_name: #env_var_name.to_string(),
                                expected_type: stringify!(#field_type).to_string(),
                            }
                        })?
                    }
                }
            }
        }
    }
}
