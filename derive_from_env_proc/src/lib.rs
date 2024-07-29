extern crate proc_macro;

use darling::FromField;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Type};

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
            let field_loaders = env_fields.iter().map(|field| {
                let field_name = field.ident.as_ref().unwrap().to_string();
                let field_type = &field.ty;
                let default_value = &field.default;
                let no_prefix = field.no_prefix;
                let from_str = field.from_str;
                let var_name = &field.var;
                if impl_from_str(field_type) || from_str {
                    match (default_value, var_name) {
                        (Some(default), Some(var_name)) => {
                            quote! {
                                #field_type::from_str(
                                    &std::env::var(#var_name.to_string())
                                        .unwrap_or_else(|_| #default.to_string())
                                ).map_err(|_| ::derive_from_env::FromEnvError::ParsingFailure{
                                    var_name:#var_name.to_string(),
                                    str_value: std::env::var(#var_name.to_string()).unwrap_or_else(|_| #default.to_string()),
                                    expected_type: stringify!(#field_type).to_string()
                                })?
                            }
                        },
                        (Some(default), None) => {
                            quote! {                
                                #field_type::from_str(
                                    &std::env::var(#field_name.to_string().to_uppercase())
                                        .unwrap_or_else(|_| #default.to_string())
                                ).map_err(|_|::derive_from_env::FromEnvError::ParsingFailure {
                                    var_name: #field_name.to_string().to_uppercase(),
                                    str_value: std::env::var(#field_name.to_string().to_uppercase()).unwrap_or_else(|_| #default.to_string()),
                                    expected_type: stringify!(#field_type).to_string()
                                })?
                            }
                        },
                        (None, Some(var_name)) => {
                            quote! {
                                #field_type::from_str(&std::env::var(#var_name.to_string())
                                    .map_err(|_| ::derive_from_env::FromEnvError::MissingEnvVar{var_name: #var_name.to_string()})?)
                                    .map_err(|_| ::derive_from_env::FromEnvError::ParsingFailure{
                                        var_name:#var_name.to_string(),
                                        str_value: std::env::var(#var_name.to_string()).unwrap(),
                                        expected_type: stringify!(#field_type).to_string()
                                    })?
                            }
                        },
                        (None, None) => {
                            quote! {
                                #field_type::from_str(&std::env::var(#field_name.to_string().to_uppercase())
                                    .map_err(|_| ::derive_from_env::FromEnvError::MissingEnvVar{var_name: #field_name.to_string().to_uppercase() })?)
                                    .map_err(|_| ::derive_from_env::FromEnvError::ParsingFailure{
                                        var_name:#field_name.to_string().to_uppercase(),
                                        str_value: std::env::var(#field_name.to_string().to_uppercase()).unwrap(),
                                        expected_type: stringify!(#field_type).to_string()
                                    })?
                            }
                        }
                    }
                } else {
                    if default_value.is_some() {
                        panic!("Default value is not supported for structs");
                    } 
                    if field.var.is_some() {
                        panic!("Variable name specification is not suited for structured fields")
                    }
                    if no_prefix {
                        quote! {
                            <#field_type as ::derive_from_env::_inner_trait::FromEnv>::from_env()?
                        }
                    } else {
                        quote! {
                            <#field_type as ::derive_from_env::_inner_trait::FromEnv>::from_env_with_prefix(#field_name.to_string().to_uppercase().as_str())?
                        }
                    }
                }
            }).collect::<Vec<_>>();
            let field_loaders_with_prefix = env_fields.iter().map(|field| {
                let field_name = field.ident.as_ref().unwrap().to_string();
                let field_type = &field.ty;
                let default_value = &field.default;
                let no_prefix = field.no_prefix;
                let from_str = field.from_str;
                let var_name = &field.var;

                if impl_from_str(field_type) || from_str {
                    match (default_value, var_name) {
                        (Some(default), Some(var_name)) => {
                            quote! {
                                <#field_type as std::str::FromStr>::from_str(
                                    &std::env::var(#var_name.to_string())
                                        .unwrap_or_else(|_| #default.to_string())
                                ).map_err(|_| ::derive_from_env::FromEnvError::ParsingFailure{
                                    var_name:#var_name.to_string(),
                                    str_value: std::env::var(#var_name.to_string()).unwrap_or_else(|_| #default.to_string()),
                                    expected_type: stringify!(#field_type).to_string()
                                })?
                            }
                        },
                        (Some(default), None) => {
                            quote! {
                                <#field_type as std::str::FromStr>::from_str(
                                    &std::env::var(format!("{prefix}_{}",#field_name.to_string().to_uppercase()))
                                        .unwrap_or_else(|_| #default.to_string())
                                ).map_err(|_| ::derive_from_env::FromEnvError::ParsingFailure{
                                    var_name:format!("{prefix}_{}",#field_name.to_string().to_uppercase()),
                                    str_value: std::env::var(format!("{prefix}_{}",#field_name.to_string().to_uppercase())).unwrap_or_else(|_| #default.to_string()),
                                    expected_type: stringify!(#field_type).to_string()
                                })?
                            }
                        },
                        (None, Some(var_name)) => {
                            quote! {
                                <#field_type as std::str::FromStr>::from_str(&std::env::var(#var_name.to_string())
                                    .map_err(|_| ::derive_from_env::FromEnvError::MissingEnvVar{var_name: #var_name.to_string()})?)
                                    .map_err(|_| ::derive_from_env::FromEnvError::ParsingFailure{
                                        var_name:#var_name.into(),
                                        str_value: std::env::var(#var_name).unwrap(),
                                        expected_type: stringify!(#field_type).to_string()
                                    })?
                            }
                        },
                        (None, None) => {
                            quote! {
                                <#field_type as std::str::FromStr>::from_str(&std::env::var(format!("{prefix}_{}",#field_name.to_string().to_uppercase())).
                                    map_err(|_| ::derive_from_env::FromEnvError::MissingEnvVar{var_name: format!("{prefix}_{}",#field_name.to_string().to_uppercase()) })?)
                                    .map_err(|_| ::derive_from_env::FromEnvError::ParsingFailure{
                                        var_name:format!("{prefix}_{}",#field_name.to_string().to_uppercase()),
                                        str_value: std::env::var(format!("{prefix}_{}",#field_name.to_string().to_uppercase())).unwrap(),
                                        expected_type: stringify!(#field_type).to_string()
                                    })?
                            }
                        }
                    }
                } else {
                    if default_value.is_some() {
                        panic!("Default value is not supported for structs");
                    } else {
                        if no_prefix {
                            quote! {
                                <#field_type as ::derive_from_env::_inner_trait::FromEnv>::from_env()?
                            }
                        } else {
                            quote! {
                                <#field_type as ::derive_from_env::_inner_trait::FromEnv>::from_env_with_prefix(format!("{prefix}_{}",#field_name.to_string().to_uppercase()).as_str())?
                            }
                        }
                    }
                }
            }).collect::<Vec<_>>();

            quote! {
                impl ::derive_from_env::_inner_trait::FromEnv for #struct_identifier {
                    fn from_env() -> Result<Self,::derive_from_env::FromEnvError> {
                        use std::str::FromStr;
                        Ok(Self {
                            #(
                                #field_identifiers: #field_loaders
                            ),*
                        })
                    }
                    fn from_env_with_prefix(prefix: &str) -> Result<Self,::derive_from_env::FromEnvError> {
                        use std::str::FromStr;
                        Ok(Self {
                            #(
                                #field_identifiers: #field_loaders_with_prefix
                            ),*
                        })
                    }
                }
                impl #struct_identifier {
                    pub fn from_env() -> Result<Self,::derive_from_env::FromEnvError> {
                        <Self as ::derive_from_env::_inner_trait::FromEnv>::from_env()
                    }
                    pub fn from_env_with_prefix(prefix: &str) -> Result<Self,::derive_from_env::FromEnvError> {
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
