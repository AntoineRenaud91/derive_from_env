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
                let field_name = field.ident.as_ref().unwrap();
                let field_type = &field.ty;
                let default_value = &field.default;
                let no_prefix = field.no_prefix;

                if impl_from_str(field_type) {
                    if let Some(default) = default_value {
                        quote! {
                            {
                                let var_name = stringify!(#field_name).to_uppercase();
                                let str_value = std::env::var(&var_name)
                                    .unwrap_or_else(|_| #default.to_string());
                                #field_type::from_str(&str_value).map_err(|_| ::derive_from_env::FromEnvError::ParsingFailure{
                                    var_name,
                                    str_value,
                                    expected_type: stringify!(#field_type).to_string()
                                })?
    
                            }
                        }
                    } else {
                        quote! {
                            {
                                let var_name = stringify!(#field_name).to_uppercase();
                                let str_value = std::env::var(&var_name).map_err(|_| ::derive_from_env::FromEnvError::MissingEnvVar{var_name: var_name.clone()})?;
                                #field_type::from_str(&str_value)
                                    .map_err(|_| ::derive_from_env::FromEnvError::ParsingFailure{
                                        var_name,
                                        str_value,
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
                                <#field_type as ::derive_from_env::_inner_trait::FromEnv>::from_env_with_prefix(stringify!(#field_name).to_uppercase().as_str())?
                            }
                        }
                    }
                }
            }).collect::<Vec<_>>();
            let field_loaders_with_prefix = env_fields.iter().map(|field| {
                let field_name = field.ident.as_ref().unwrap();
                let field_type = &field.ty;
                let default_value = &field.default;
                let no_prefix = field.no_prefix;

                if impl_from_str(field_type) {
                    if let Some(default) = default_value {
                        quote! {
                            #field_type::from_str(
                                &std::env::var(format!("{prefix}_{}",stringify!(#field_name).to_uppercase()).as_str())
                                    .unwrap_or_else(|_| #default.to_string())
                            ).unwrap()
                        }
                    } else {
                        quote! {
                            #field_type::from_str(&std::env::var(format!("{prefix}_{}",stringify!(#field_name).to_uppercase()).as_str()).unwrap()).unwrap()
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
                                <#field_type as ::derive_from_env::_inner_trait::FromEnv>::from_env_with_prefix(format!("{prefix}_{}",stringify!(#field_name).to_uppercase()).as_str())?
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
        Type::Path(type_path) if type_path.path.segments.iter().all(|seg| {
                let ty_name = seg.ident.to_string();
                println!("{}", ty_name);
                matches!(ty_name.as_str(),
                    "i8" | "i16" | "i32" | "i64" | "i128" |
                    "u8" | "u16" | "u32" | "u64" | "u128" |
                    "f32" | "f64" | "bool" | "char" | "usize" |
                    "isize" | "String" | "IpAddr" | "SocketAddr" |
                    "PathBuf" | "IpV4Addr" | "IpV6Addr"
                )
            }
        )
    )
}
