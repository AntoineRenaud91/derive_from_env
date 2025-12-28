extern crate proc_macro;

use darling::FromField;
use proc_macro::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, Data, DeriveInput, GenericArgument, PathArguments, Type};

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
    from_str: bool,
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
            let field_loaders: Vec<_> = env_fields
                .iter()
                .map(|field| generate_field_loader(field, false))
                .collect();
            let field_loaders_with_prefix: Vec<_> = env_fields
                .iter()
                .map(|field| generate_field_loader(field, true))
                .collect();

            Ok(quote! {
                impl ::derive_from_env::_inner_trait::FromEnv for #struct_identifier {
                    fn from_env() -> Result<Self, ::derive_from_env::FromEnvError> {
                        use std::str::FromStr;
                        Ok(Self {
                            #(
                                #field_identifiers: #field_loaders
                            ),*
                        })
                    }
                    fn from_env_with_prefix(prefix: &str) -> Result<Self, ::derive_from_env::FromEnvError> {
                        use std::str::FromStr;
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

fn impl_from_str(ty: &Type) -> bool {
    matches!(ty,
        Type::Path(type_path) if type_path.path.segments.iter().all(|seg|
            matches!(seg.ident.to_string().as_str(),
                "i8" | "i16" | "i32" | "i64" | "i128" |
                "u8" | "u16" | "u32" | "u64" | "u128" |
                "f32" | "f64" | "bool" | "char" | "usize" |
                "isize" | "String" | "IpAddr" | "SocketAddr" |
                "PathBuf" | "Ipv4Addr" | "Ipv6Addr"
            )
        )
    )
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

fn generate_field_loader(field: &EnvField, prefix: bool) -> proc_macro2::TokenStream {
    let field_name = field.ident.as_ref().unwrap().to_string();
    let field_type = &field.ty;
    let inner_field_type = extract_inner_type_if_option(field_type);
    let default_value = &field.default;
    let no_prefix = field.no_prefix;
    let from_str = field.from_str;
    let var_name = &field.var;

    let env_var_name = if prefix {
        quote! { format!("{}_{}", prefix, #field_name.to_uppercase()) }
    } else {
        quote! { #field_name.to_uppercase() }
    };
    if let Some(field_type) = inner_field_type {
        if !(impl_from_str(field_type) || from_str) {
            panic!("Inner type of Option must implement FromStr");
        }
        if default_value.is_some() {
            panic!("Default value is not supported for Option fields");
        }
        if let Some(var_name) = var_name {
            quote! {
                match std::env::var(#var_name.to_string()) {
                    Ok(s) => Some(#field_type::from_str(&s).map_err(|_| {
                        ::derive_from_env::FromEnvError::ParsingFailure {
                            var_name: #var_name.to_string(),
                            str_value: s,
                            expected_type: stringify!(#field_type).to_string(),
                        }
                    })?),
                    Err(_) => None,
                }
            }
        } else {
            quote! {
                match std::env::var(#env_var_name) {
                    Ok(s) => Some(#field_type::from_str(&s).map_err(|_| {
                        ::derive_from_env::FromEnvError::ParsingFailure {
                            var_name: #env_var_name.to_string(),
                            str_value: s,
                            expected_type: stringify!(#field_type).to_string(),
                        }
                    })?),
                    Err(_) => None,
                }
            }
        }
    } else if impl_from_str(field_type) || from_str || inner_field_type.is_some() {
        match (default_value, var_name) {
            (Some(default), Some(var_name)) => {
                quote! {
                    {
                        let __env_val = std::env::var(#var_name.to_string())
                            .unwrap_or_else(|_| #default.to_string());
                        #field_type::from_str(&__env_val).map_err(|_| {
                            ::derive_from_env::FromEnvError::ParsingFailure {
                                var_name: #var_name.to_string(),
                                str_value: __env_val,
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
                                str_value: __env_val,
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
                                str_value: __env_val,
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
                                str_value: __env_val,
                                expected_type: stringify!(#field_type).to_string(),
                            }
                        })?
                    }
                }
            }
        }
    } else {
        if default_value.is_some() {
            panic!("Default value is not supported for structs");
        }
        if var_name.is_some() {
            panic!("Variable name specification is not suited for structured fields")
        }
        if no_prefix {
            quote! {
                <#field_type as ::derive_from_env::_inner_trait::FromEnv>::from_env()?
            }
        } else {
            quote! {
                <#field_type as ::derive_from_env::_inner_trait::FromEnv>::from_env_with_prefix(&#env_var_name)?
            }
        }
    }
}
