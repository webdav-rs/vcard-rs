use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{self, spanned::Spanned, Fields, Ident, Item};

fn impl_getter_trait_for_type<T>(
    input: TokenStream,
    field_name: &str,
    error_message: &str,
    callback: impl Fn(&Ident) -> T,
) -> TokenStream
where
    T: Into<TokenStream>,
{
    let item: syn::Item = syn::parse(input).expect("failed to parse input");
    match item {
        Item::Struct(ref struct_item) => match &struct_item.fields {
            Fields::Named(fields) => {
                let field_present = fields
                    .named
                    .iter()
                    .filter_map(|f| f.ident.as_ref())
                    .find(|ident| ident.to_owned() == field_name)
                    .is_some();
                if !field_present {
                    return quote! {
                        compile_error!(#error_message);
                    }
                    .into();
                }

                let name = &struct_item.ident;

                return callback(name).into();
            }
            _ => {
                return quote! {
                    compile_error!(#error_message);
                }
                .into()
            }
        },
        _ => {
            return quote! {
                compile_error!(#error_message);
            }
            .into()
        }
    }
}

#[proc_macro_derive(AltID)]
pub fn alt_id_derive(input: TokenStream) -> TokenStream {
    impl_getter_trait_for_type(
        input,
        "altid",
        "AltID can only be used on structs with an altid field",
        |ident| {
            quote! {
                impl Alternative for #ident {
                    fn get_alt_id(&self) -> &str {
                        self.altid.as_ref().map(String::as_str).unwrap_or_else(||"")
                    }

                }
            }
        },
    )
}

#[proc_macro_derive(Pref)]
pub fn pref_derive(input: TokenStream) -> TokenStream {
    impl_getter_trait_for_type(
        input,
        "pref",
        "Pref can only be used on structs with a pref field",
        |ident| {
            quote! {
                impl Preferable for #ident {
                    fn get_pref(&self) -> u8 {
                        self.pref.unwrap_or_else(||100)
                    }

                }
            }
        },
    )
}

// This macro is intended to ease the repetitive `Display` trait implementation.
#[proc_macro_attribute]
pub fn vcard(_metadata: TokenStream, input: TokenStream) -> TokenStream {
    let item: syn::Item = syn::parse(input).expect("failed to parse input");

    match item {
        Item::Struct(ref struct_item) => match &struct_item.fields {
            Fields::Named(fields) => {
                let struct_name = struct_item.ident.to_string().to_uppercase();
                let mut grp_stmt = quote! {
                    let name = #struct_name;
                    write!(f,"{}",name)?;
                };
                let mut stmts = Vec::new();

                for field in fields.named.iter() {
                    let ident = &field.ident.as_ref().unwrap().to_string();
                    match &ident[..] {
                        "group" => {
                            grp_stmt = quote! {
                                let name = #struct_name;
                                if let Some(grp) = self.group.as_ref() {
                                    write!(f,"{}.{}",grp,name)?;
                                } else {
                                    write!(f,"{}",name)?;
                                }
                            };
                        }
                        "altid" => {
                            stmts.push(quote! {
                                if let Some(altid) = self.altid.as_ref() {
                                    write!(f,";ALTID={}",altid)?;
                                }
                            });
                        }
                        "language" => {
                            stmts.push(quote! {
                                if let Some(language) = self.language.as_ref() {
                                    write!(f,";LANGUAGE={}",language)?;
                                }
                            });
                        }
                        "value_data_type" => {
                            stmts.push(quote! {
                                if let Some(vdt) = self.value_data_type.as_ref() {
                                    write!(f,";VALUE={}",vdt.as_ref())?;
                                }
                            });
                        }
                        "pref" => {
                            stmts.push(quote! {
                                if let Some(p) = self.pref.as_ref() {
                                    write!(f,";PREF={}",p)?;
                                }
                            });
                        }
                        "pid" => {
                            stmts.push(quote! {
                                if let Some(p) = self.pid.as_ref() {
                                    write!(f,";PID={}",p)?;
                                }
                            });
                        }
                        "type_param" => {
                            stmts.push(quote! {
                                if let Some(types) = self.type_param.as_ref() {
                                    for t in types {
                                        write!(f,";TYPE={}",t)?;
                                    }
                                }
                            });
                        }
                        "mediatype" => {
                            stmts.push(quote! {
                                if let Some(m) = self.mediatype.as_ref() {
                                    write!(f,";MEDIATYPE={}",m)?;
                                }
                            });
                        }
                        "calscale" => {
                            stmts.push(quote! {
                                if let Some(c) = self.calscale.as_ref() {
                                    write!(f,";CALSCALE={}",c)?;
                                }
                            });
                        }
                        "sort_as" => {
                            stmts.push(quote! {
                                if let Some(s) = self.sort_as.as_ref() {
                                    write!(f,";SORT-AS=\"{}\"",s.join(","))?;
                                }
                            });
                        }
                        "geo" => {
                            stmts.push(quote! {
                                if let Some(g) = self.geo.as_ref() {
                                    write!(f,";GEO={}",g)?;
                                }
                            });
                        }
                        "tz" => {
                            stmts.push(quote! {
                                if let Some(t) = self.tz.as_ref() {
                                    write!(f,";TZ={}",t)?;
                                }
                            });
                        }

                        _ => {}
                    }
                }

                let value_stmt = match &struct_name[..] {
                    "ORG" => {
                        quote! {
                            write!(f,":{}\r\n",self.value.join(";"))?;
                        }
                    }
                    "CATEGORIES" | "NICKNAME" => {
                        quote! {
                            write!(f,":{}\r\n",self.value.join(","))?;
                        }
                    }
                    "ADR" => {
                        quote! {
                            write!(f,":{};{};{};{};{};{};{}\r\n",self.po_box.join(","),self.extended_address.join(","),self.street.join(","),self.city.join(","),self.region.join(","),self.postal_code.join(","),self.country.join(","))?;
                        }
                    }

                    "N" => {
                        quote! {
                            write!(f,":{};{};{};{};{}\r\n",self.surenames.join(","),self.given_names.join(","),self.additional_names.join(","),self.honorific_prefixes.join(","),self.honorific_suffixes.join(","))?;

                        }
                    }
                    "GENDER" => {
                        quote! {
                            if let Some(s) = self.sex.as_ref(){
                                write!(f,":{}",s.as_ref())?;
                            } else {
                                write!(f,":")?;
                            }
                            if let Some(c) = self.identity_component.as_ref() {
                                write!(f,";{}",c)?;
                            }
                            write!(f,"\r\n")?;
                        }
                    }
                    "VERSION" | "KIND" => {
                        quote! {
                            write!(f,":{}\r\n",self.value.as_ref())?;
                        }
                    }
                    _ => quote! {
                        write!(f,":{}\r\n",self.value.as_str())?;
                    },
                };
                let name = &struct_item.ident;
                let output = quote! {
                    #item

                    impl Display for #name {
                        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                            #grp_stmt
                            #(#stmts)*
                            #value_stmt
                            Ok(())
                        }

                    }
                };
                output.into()
            }
            _ => {
                return quote_spanned! {
                    item.span() =>
                    compile_error!("expected named fields");
                }
                .into()
            }
        },

        // If the attribute was applied to any other kind of item, we want
        // to generate a compiler error.
        _ => {
            return quote_spanned! {
                item.span() =>
                compile_error!("expected struct");
            }
            .into()
        }
    }
}
