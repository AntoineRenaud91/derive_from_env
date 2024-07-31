extern crate proc_macro;

use darling::FromField;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, GenericArgument, PathArguments, Type};

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
    let DeriveInput { ident, data, .. } = syn::parse_macro_input!(item as syn::DeriveInput);
    let struct_identifier = &ident;

    match &data {
        Data::Struct(syn::DataStruct { fields, .. }) => {
            let env_fields = fields
                .iter()
                .map(|field| EnvField::from_field(field).unwrap())
                .collect::<Vec<_>>();
            let field_identifiers = env_fields
                .iter()
                .map(|f| f.ident.as_ref().unwrap())
                .collect::<Vec<_>>();
            let field_loaders = env_fields
                .iter()
                .map(|field| generate_field_loader(field, false))
                .collect::<Vec<_>>();
            let field_loaders_with_prefix = env_fields
                .iter()
                .map(|field| generate_field_loader(field, true))
                .collect::<Vec<_>>();

            quote! {
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
            }.into()
        }
        _ => unimplemented!(),
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
                "PathBuf" | "IpV4Addr" | "IpV6Addr"
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
                std::env::var(#var_name.to_string())
                    .ok()
                    .map(|s| #field_type::from_str(&s))
                    .transpose()
                    .map_err(|_| ::derive_from_env::FromEnvError::ParsingFailure{
                        var_name: #var_name.to_string(),
                        str_value: std::env::var(#var_name.to_string()).unwrap(),
                        expected_type: stringify!(#field_type).to_string()
                    })?
            }
        } else {
            quote! {
                std::env::var(#env_var_name)
                    .ok()
                    .map(|s| #field_type::from_str(&s))
                    .transpose()
                    .map_err(|_| ::derive_from_env::FromEnvError::ParsingFailure{
                        var_name: #env_var_name.to_string(),
                        str_value: std::env::var(#env_var_name).unwrap(),
                        expected_type: stringify!(#field_type).to_string()
                    })?
            }
        }
    } else if impl_from_str(field_type) || from_str || inner_field_type.is_some() {
        match (default_value, var_name) {
            (Some(default), Some(var_name)) => {
                quote! {
                    #field_type::from_str(
                        &std::env::var(#var_name.to_string())
                            .unwrap_or_else(|_| #default.to_string())
                    ).map_err(|_| ::derive_from_env::FromEnvError::ParsingFailure{
                        var_name: #var_name.to_string(),
                        str_value: std::env::var(#var_name.to_string()).unwrap_or_else(|_| #default.to_string()),
                        expected_type: stringify!(#field_type).to_string()
                    })?
                }
            }
            (Some(default), None) => {
                quote! {
                    #field_type::from_str(
                        &std::env::var(#env_var_name)
                            .unwrap_or_else(|_| #default.to_string())
                    ).map_err(|_|::derive_from_env::FromEnvError::ParsingFailure {
                        var_name: #env_var_name.to_string(),
                        str_value: std::env::var(#env_var_name).unwrap_or_else(|_| #default.to_string()),
                        expected_type: stringify!(#field_type).to_string()
                    })?
                }
            }
            (None, Some(var_name)) => {
                quote! {
                    #field_type::from_str(&std::env::var(#var_name.to_string())
                        .map_err(|_| ::derive_from_env::FromEnvError::MissingEnvVar{var_name: #var_name.to_string()})?)
                        .map_err(|_| ::derive_from_env::FromEnvError::ParsingFailure{
                            var_name: #var_name.to_string(),
                            str_value: std::env::var(#var_name.to_string()).unwrap(),
                            expected_type: stringify!(#field_type).to_string()
                        })?
                }
            }
            (None, None) => {
                quote! {
                    #field_type::from_str(&std::env::var(#env_var_name)
                        .map_err(|_| ::derive_from_env::FromEnvError::MissingEnvVar{var_name: #env_var_name.to_string()})?)
                        .map_err(|_| ::derive_from_env::FromEnvError::ParsingFailure{
                            var_name: #env_var_name.to_string(),
                            str_value: std::env::var(#env_var_name).unwrap(),
                            expected_type: stringify!(#field_type).to_string()
                        })?
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
