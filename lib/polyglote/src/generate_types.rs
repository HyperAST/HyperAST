use std::collections::{BTreeMap, HashMap};

use crate::keywords::{AdditionalKeyword, CppKeyword, JavaKeyword};
use crate::preprocess::{DChildren, Fields, Hidden, MultipleChildren, RequiredChildren};
use crate::preprocess::{Named, SubTypes};

use super::*;
use heck::ToUpperCamelCase;
use proc_macro2::Ident;
use quote::{format_ident, quote};
// use syn::{parse_macro_input, DeriveInput};

pub trait BijectiveFormatedIdentifier: ToOwned {
    /// Convert this type to camel case.
    fn try_format_ident(&self) -> Option<Self::Owned>;
}
impl BijectiveFormatedIdentifier for str {
    fn try_format_ident(&self) -> Option<Self::Owned> {
        let mut camel_case = heck::ToUpperCamelCase::to_upper_camel_case(self);
        let trimmed = self.trim_start_matches(|c| c == '_');
        if camel_case.is_empty() {
            if trimmed.is_empty() && !self.is_empty() {
                // return Some(self.to_owned())
                return None;
            }
            return None;
        }
        let u_count = self.len() - trimmed.len();
        if u_count > 0 {
            camel_case.insert_str(0, &"_".repeat(u_count));
        }
        heck::ToSnakeCase::to_snake_case(&camel_case as &str)
            .eq(trimmed)
            .then_some(camel_case)
    }
}

pub fn serialize_types(typesys: &TypeSys) {
    let res = process_types_into_tokens(typesys);
    println!("{}", res);
    let res = syn::parse_file(&res.to_string()).unwrap();
    let res = prettyplease::unparse(&res);
    println!("{}", res);
}

pub(crate) fn process_types_into_tokens(typesys: &TypeSys) -> proc_macro2::TokenStream {
    let mut merged = quote! {};
    let mut from_u16 = quote! {};
    let mut cat_from_u16 = quote! {};
    let mut from_str = quote! {};
    let mut to_str = quote! {};
    let mut as_vec_toks = quote! {};
    let mut hidden_toks = quote! {};
    let mut hidden_toks_pred = quote! {};
    let mut keyword_toks = quote! {};
    let mut concrete_toks = quote! {};
    let mut with_field_toks = quote! {};
    let mut abstract_toks = quote! {};
    let mut supertype_pred = quote! {};
    let mut named_pred = quote! {};

    let mut alias_dedup = HashMap::<hecs::Entity, Ident>::default();
    let mut leafs = HM::default();
    <JavaKeyword as strum::IntoEnumIterator>::iter().for_each(|x| {
        leafs.unamed.insert(x.to_string(), format!("{:?}", x));
    });
    <CppKeyword as strum::IntoEnumIterator>::iter().for_each(|x| {
        leafs.unamed.insert(x.to_string(), format!("{:?}", x));
    });
    <AdditionalKeyword as strum::IntoEnumIterator>::iter().for_each(|x| {
        leafs.unamed.insert(x.to_string(), format!("{:?}", x));
    });
    let mut count = 0;

    for (i, e) in typesys.list.iter().enumerate() {
        let i = i as u16;
        let v = typesys.types.entity(*e).unwrap();
        let t = v.get::<&preprocess::T>().unwrap().0.to_string();
        if let Some(kind) = alias_dedup.get(e) {
            from_u16.extend(quote! {
                #i => Type::#kind,
            });
            continue;
        }

        if !v.get::<&Named>().is_some() {
            // leaf/token
            let camel_case = t.try_format_ident();
            let raw = t.clone();
            let (q, kind) = if let Some(camel_case) = &camel_case {
                assert!(!camel_case.is_empty(), "{},{}", t, t.to_upper_camel_case());
                let kind = if camel_case == "0" {
                    let camel_case = leafs.fmt(&t, |k| format!("TS{}", &k));
                    format_ident!("{}", &camel_case)
                } else {
                    format_ident!("{}", &camel_case)
                };

                (
                    quote! {
                        #kind,
                    },
                    kind,
                )
            } else {
                let k = leafs.fmt(&t, |k| format!("TS{}", &k.to_upper_camel_case()));
                let kind = format_ident!("{}", &k);

                (
                    quote! {
                        // #[strum(serialize = #raw)]
                        #kind(Raw<#raw>),
                    },
                    kind,
                )
            };

            if v.has::<Hidden>() {
                hidden_toks.extend(q);
                hidden_toks_pred.extend(quote! {
                    Type::#kind => true,
                });
                cat_from_u16.extend(quote! {
                    #i => TypeEnum::Hidden(Hidden::#kind),
                });
                as_vec_toks.extend(quote! {
                    Hidden(#kind),
                });
            } else {
                keyword_toks.extend(q);
                cat_from_u16.extend(quote! {
                    // #i => TypeEnum::Keyword(Keyword::#kind),
                    #i => Type::#kind,
                });
                as_vec_toks.extend(quote! {
                    Keyword(#kind),
                });
            }
            from_u16.extend(quote! {
                #i => Type::#kind,
            });
            merged.extend(quote! {
                #kind,
            });
            to_str.extend(quote! {
                Type::#kind => #raw,
            });
            from_str.extend(quote! {
                #raw => Type::#kind,
            });
            alias_dedup.insert(*e, kind);
        } else if let Some(st) = v.get::<&SubTypes>() {
            let camel_case = t.try_format_ident();
            let kind = format_ident!(
                "{}",
                &camel_case
                    .clone()
                    .unwrap_or_else(|| t.to_upper_camel_case())
            );
            let raw = t.clone();
            let mut sub_toks = quote! {};
            for e in &st.0 {
                let v = typesys.types.entity(*e).unwrap();
                let t = &v.get::<&preprocess::T>().unwrap().0;
                let camel_case = t.try_format_ident();
                if let Some(camel_case) = camel_case {
                    let kind = format_ident!("{}", &camel_case);
                    sub_toks.extend(quote! {
                        // #[strum(serialize = #raw)]
                        #kind,
                    });
                } else {
                    let kind = if !v.get::<&Named>().is_some() {
                        let k = leafs.fmt(t, |k| format!("TS{}", &k.to_upper_camel_case()));
                        format_ident!("{}", &k)
                    } else {
                        format_ident!("{}", &t.to_upper_camel_case())
                    };
                    sub_toks.extend(quote! {
                        // #[strum(serialize = #raw)]
                        #kind,
                    });
                }
            }
            hidden_toks_pred.extend(quote! {
                Type::#kind => true,
            });
            supertype_pred.extend(quote! {
               Type::#kind => true,
            });
            named_pred.extend(quote! {
               Type::#kind => true,
            });
            if camel_case.is_none() {
                abstract_toks.extend(quote! {
                    // #[strum(serialize = #raw)]
                    #kind(Raw<#raw>, #sub_toks),
                });
            } else {
                abstract_toks.extend(quote! {
                    #kind(#sub_toks),
                });
            }
            cat_from_u16.extend(quote! {
                #i => TypeEnum::Abstract(Abstract::#kind),
            });
            as_vec_toks.extend(quote! {
                Abstract(#kind),
            });

            merged.extend(quote! {
                #kind,
            });
            from_u16.extend(quote! {
                #i => Type::#kind,
            });
            to_str.extend(quote! {
                Type::#kind => #raw,
            });
            from_str.extend(quote! {
                #raw => Type::#kind,
            });
            alias_dedup.insert(*e, kind);
        } else if let Some(fields) = v.get::<&Fields>() {
            let camel_case = t.try_format_ident();
            let kind = format_ident!(
                "{}",
                &camel_case
                    .clone()
                    .unwrap_or_else(|| t.to_upper_camel_case())
            );
            let raw = t.clone();
            let mut fields_toks = quote! {};
            for e in &fields.0 {
                let v = typesys.types.entity(*e).unwrap();
                let t = &v.get::<&preprocess::Role>().unwrap().0;
                let camel_case = t.try_format_ident();
                assert_ne!(camel_case, None);
                let t = if t == "type" { "r#type" } else { t };
                let kind = format_ident!("{}", &t);
                let cs = &v.get::<&preprocess::DChildren>().unwrap().0;
                let mut cs_toks = quote! {};
                for e in cs {
                    let v = typesys.types.entity(*e).unwrap();
                    let t = &v.get::<&preprocess::T>().unwrap().0;
                    let camel_case = t.try_format_ident();
                    if let Some(camel_case) = camel_case {
                        let kind = format_ident!("{}", &camel_case);
                        cs_toks.extend(quote! {
                            #kind,
                        });
                    } else {
                        let kind = if !v.get::<&Named>().is_some() {
                            let k = leafs.fmt(t, |k| format!("TS{}", &k.to_upper_camel_case()));
                            format_ident!("{}", &k)
                        } else {
                            format_ident!("{}", &t.to_upper_camel_case())
                        };
                        cs_toks.extend(quote! {
                            #kind,
                        });
                    }
                }
                if v.has::<RequiredChildren>() {
                    if v.has::<MultipleChildren>() {
                        fields_toks.extend(quote! {
                            #kind:MultReq<(#cs_toks)>,
                        });
                    } else {
                        fields_toks.extend(quote! {
                            #kind:Req<(#cs_toks)>,
                        });
                    }
                } else if v.has::<MultipleChildren>() {
                    fields_toks.extend(quote! {
                        #kind:Mult<(#cs_toks)>,
                    });
                } else {
                    fields_toks.extend(quote! {
                        #kind: (#cs_toks),
                    });
                }
            }
            if let Some(cs) = v.get::<&preprocess::DChildren>() {
                let mut cs_toks = quote! {};
                for e in &cs.0 {
                    let v = typesys.types.entity(*e).unwrap();
                    let t = &v.get::<&preprocess::T>().unwrap().0;
                    let camel_case = t.try_format_ident();
                    if let Some(camel_case) = camel_case {
                        let kind = format_ident!("{}", &camel_case);
                        cs_toks.extend(quote! {
                            #kind,
                        });
                    } else {
                        let kind = if !v.get::<&Named>().is_some() {
                            let k = leafs.fmt(t, |k| format!("TS{}", &k.to_upper_camel_case()));
                            format_ident!("{}", &k)
                        } else {
                            format_ident!("{}", &t.to_upper_camel_case())
                        };
                        cs_toks.extend(quote! {
                            #kind,
                        });
                    }
                }
                // fields_toks.extend(quote! {
                //     _cs:(#cs_toks),
                // });

                if v.has::<RequiredChildren>() {
                    if v.has::<MultipleChildren>() {
                        fields_toks.extend(quote! {
                            _cs:MultReq<(#cs_toks)>,
                        });
                    } else {
                        fields_toks.extend(quote! {
                            _cs:Req<(#cs_toks)>,
                        });
                    }
                } else if v.has::<MultipleChildren>() {
                    fields_toks.extend(quote! {
                        _cs:Mult<(#cs_toks)>,
                    });
                } else {
                    fields_toks.extend(quote! {
                        _cs: (#cs_toks),
                    });
                }
            }
            if camel_case.is_none() {
                with_field_toks.extend(quote! {
                    // #[strum(serialize = #raw)]
                    #kind{_ser: Raw<#raw>, #fields_toks},
                });
            } else {
                with_field_toks.extend(quote! {
                    #kind{#fields_toks},
                });
            }
            cat_from_u16.extend(quote! {
                #i => TypeEnum::WithFields(WithFields::#kind),
            });
            as_vec_toks.extend(quote! {
                WithFields(#kind),
            });

            merged.extend(quote! {
                #kind,
            });
            from_u16.extend(quote! {
                #i => Type::#kind,
            });
            to_str.extend(quote! {
                Type::#kind => #raw,
            });
            from_str.extend(quote! {
                #raw => Type::#kind,
            });
            named_pred.extend(quote! {
               Type::#kind => true,
            });
            alias_dedup.insert(*e, kind);
        } else if let Some(cs) = v.get::<&DChildren>() {
            let camel_case = t.try_format_ident();
            let kind = format_ident!(
                "{}",
                &camel_case
                    .clone()
                    .unwrap_or_else(|| t.to_upper_camel_case())
            );
            let raw = t.clone();
            let mut cs_toks = quote! {};
            for e in &cs.0 {
                let v = typesys.types.entity(*e).unwrap();
                let t = &v.get::<&preprocess::T>().unwrap().0;
                let camel_case = t.try_format_ident();
                if let Some(camel_case) = camel_case {
                    let kind = format_ident!("{}", &camel_case);
                    cs_toks.extend(quote! {
                        #kind,
                    });
                } else {
                    let kind = if !v.get::<&Named>().is_some() {
                        let k = leafs.fmt(t, |k| format!("TS{}", &k.to_upper_camel_case()));
                        format_ident!("{}", &k)
                    } else {
                        format_ident!("{}", &t.to_upper_camel_case())
                    };
                    cs_toks.extend(quote! {
                        #kind,
                    });
                }
            }
            let cs_toks = if v.has::<RequiredChildren>() {
                if v.has::<MultipleChildren>() {
                    quote! {
                        MultReq<(#cs_toks)>,
                    }
                } else {
                    quote! {
                        Req<(#cs_toks)>,
                    }
                }
            } else if v.has::<MultipleChildren>() {
                quote! {
                    Mult<(#cs_toks)>,
                }
            } else {
                quote! {
                    #cs_toks
                }
            };
            if camel_case.is_none() {
                concrete_toks.extend(quote! {
                    // #[strum(serialize = #raw)]
                    #kind(Raw<#raw>,#cs_toks),
                });
            } else {
                concrete_toks.extend(quote! {
                    #kind(#cs_toks),
                });
            }
            cat_from_u16.extend(quote! {
                #i => TypeEnum::Concrete(Concrete::#kind),
            });
            as_vec_toks.extend(quote! {
                Concrete(#kind),
            });

            merged.extend(quote! {
                #kind,
            });
            from_u16.extend(quote! {
                #i => Type::#kind,
            });
            to_str.extend(quote! {
                Type::#kind => #raw,
            });
            from_str.extend(quote! {
                #raw => Type::#kind,
            });
            named_pred.extend(quote! {
               Type::#kind => true,
            });
            alias_dedup.insert(*e, kind);
        } else {
            let camel_case = t.try_format_ident();
            let kind = format_ident!(
                "{}",
                &camel_case
                    .clone()
                    .unwrap_or_else(|| t.to_upper_camel_case())
            );
            let raw = t.clone();
            if camel_case.is_none() {
                concrete_toks.extend(quote! {
                    // #[strum(serialize = #raw)]
                    #kind(Raw<#raw>),
                });
            } else {
                concrete_toks.extend(quote! {
                    #kind,
                });
            }
            cat_from_u16.extend(quote! {
                #i => TypeEnum::Concrete(Concrete::#kind),
            });
            as_vec_toks.extend(quote! {
                Concrete(#kind),
            });
            if v.has::<Hidden>() {
                panic!();
            }

            merged.extend(quote! {
                #kind,
            });
            from_u16.extend(quote! {
                #i => Type::#kind,
            });
            to_str.extend(quote! {
                Type::#kind => #raw,
            });
            from_str.extend(quote! {
                #raw => Type::#kind,
            });
            named_pred.extend(quote! {
               Type::#kind => true,
            });
            alias_dedup.insert(*e, kind);
        }
        // let v = self.abstract_types.entity(*e).unwrap();
        // writeln!(f, "{:?}: {:?}", t, e)?;
        // if v.get::<&Named>().is_some() {
        //     writeln!(f, "\tnamed")?;
        // }
        // if let Some(st) = v.get::<&SubTypes>() {
        //     writeln!(f, "\tsubtypes: {:?}", st.0)?;
        // }
        // if let Some(fi) = v.get::<&Fields>() {
        //     writeln!(f, "\tfields: {:?}", fi.0)?;
        // }
        // if let Some(cs) = v.get::<&DChildren>() {
        //     writeln!(f, "\tchildren: {:?}", cs.0)?;
        // }
        count += 1;
    }

    let len = typesys.list.len() as u16;
    dbg!(count, len);

    let res = quote! {
        // enum TypeEnum {
        //     Keyword(Keyword),
        //     Concrete(Concrete),
        //     WithFields(WithFields),
        //     Abstract(Abstract),
        //     Hidden(Hidden),
        //     OutOfBound,
        // }
        // enum Hidden {
        //     #hidden_toks
        // }
        // enum Keyword {
        //     #keyword_toks
        // }
        // /// Type of nodes actually stored
        // /// ie. what should be stored on CST nodes
        // /// but anyway encode it as a number
        // /// and it would be better to take the smallest numbers for concrete nodes
        // /// to facilitate convertion
        // enum Concrete {
        //     #concrete_toks
        //     // #named_concrete_types_toks
        // }
        // enum WithFields {
        //     #with_field_toks
        // }
        // enum Abstract {
        //     #abstract_toks
        // }
        // pub fn from_u16(t: u16) -> TypeResult {
        //     match t {
        //         #cat_from_u16
        //         #len => TypeEnum::ERROR
        //     }
        // }
        // const COUNT: usize = #count;
        // const TS2Enum: &[()] = [
        //     #as_vec_toks
        // ];

        #[repr(u16)]
        #[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
        pub enum Type {
            #merged
            Spaces,
            Directory,
            ERROR,
        }
        impl Type {
            pub fn from_u16(t: u16) -> Type {
                match t {
                    #from_u16
                    //#len => Type::ERROR,
                    u16::MAX => Type::ERROR,
                    x => panic!("{}",x),
                }
            }
            pub fn from_str(t: &str) -> Option<Type> {
                Some(match t {
                    #from_str
                    "Spaces" => Type::Spaces,
                    "Directory" => Type::Directory,
                    "ERROR" => Type::ERROR,
                    _ => return None,
                })
            }
            pub fn to_str(&self) -> &'static str {
                match self {
                    #to_str
                    Type::Spaces => "Spaces",
                    Type::Directory => "Directory",
                    Type::ERROR => "ERROR",
                }
            }
            pub fn is_hidden(&self) -> bool {
                match self {
                    #hidden_toks_pred
                    _ => false,
                }
            }
            pub fn is_supertype(&self) -> bool {
                match self {
                    #supertype_pred
                    _ => false,
                }
            }
            pub fn is_named(&self) -> bool {
                match self {
                    #named_pred
                    _ => false,
                }
            }

        }
        // /// all types
        // enum Types {
        //     #types_toks
        // }
        // impl Types {
        //     // pub fn parse_xml(t: &str) -> Self {
        //     //     match t {
        //     //         #into_types_toks
        //     //     }
        //     // }
        // }
        // mod abstract_types {
        //     #abstract_types_toks
        // }
    };
    res
}

pub fn serialize_types2(typesys: &TypeSys) {
    let mut concrete_types_toks = quote! {};
    let mut abstract_types_toks = quote! {};
    let mut types_toks = quote! {};
    let mut into_types_toks = quote! {};
    let mut leafs = HM::default();
    let mut count = 0;

    for (t, e) in &typesys.index {
        let v = typesys.types.entity(*e).unwrap();

        if !v.get::<&Named>().is_some() {
            // leaf/token
            let k = leafs.fmt(t, |k| format!("cpp_TS{}", &k.to_upper_camel_case()));
            let kind = format_ident!("{}", &k);
            let raw = t.clone();

            concrete_types_toks.extend(quote! {
                #[strum(serialize = #raw)]
                #kind,
            });
            types_toks.extend(quote! {
                #[strum(serialize = #raw)]
                #kind,
            });
            into_types_toks.extend(quote! {
                #raw => #kind,
            });
        } else if let Some(st) = v.get::<&SubTypes>() {
            let kind = format_ident!("cpp_{}", &t.to_upper_camel_case());
            let raw = t.clone();
            let mut sub_toks = quote! {};
            for e in &st.0 {
                let v = typesys.types.entity(*e).unwrap();
                let t = &v.get::<&preprocess::T>().unwrap().0;
                let kind = format_ident!("{}", &t.to_upper_camel_case());
                let raw = t.clone();
                sub_toks.extend(quote! {
                    #[strum(serialize = #raw)]
                    #kind,
                });
            }
            let ty = quote! {
                enum #kind {
                    #sub_toks
                }
            };
            abstract_types_toks.extend(ty);
            types_toks.extend(quote! {
                #[strum(serialize = #raw)]
                #kind,
            });
            into_types_toks.extend(quote! {
                #raw => #kind,
            });
        } else {
            let kind = format_ident!("cpp_{}", &t.to_upper_camel_case());
            let raw = t.clone();
            concrete_types_toks.extend(quote! {
                #[strum(serialize = #raw)]
                #kind,
            });
            types_toks.extend(quote! {
                #[strum(serialize = #raw)]
                #kind,
            });
            into_types_toks.extend(quote! {
                #raw => #kind,
            });
        }
        // let v = self.abstract_types.entity(*e).unwrap();
        // writeln!(f, "{:?}: {:?}", t, e)?;
        // if v.get::<&Named>().is_some() {
        //     writeln!(f, "\tnamed")?;
        // }
        // if let Some(st) = v.get::<&SubTypes>() {
        //     writeln!(f, "\tsubtypes: {:?}", st.0)?;
        // }
        // if let Some(fi) = v.get::<&Fields>() {
        //     writeln!(f, "\tfields: {:?}", fi.0)?;
        // }
        // if let Some(cs) = v.get::<&DChildren>() {
        //     writeln!(f, "\tchildren: {:?}", cs.0)?;
        // }
        count += 1;
    }
    dbg!(count);

    let res = quote! {
        /// Type of nodes actually stored
        /// ie. what should be stored on CST nodes
        /// but anyway encode it as a number
        /// and it would be better to take the smallest numbers for concrete nodes
        /// to facilitate convertion
        enum ConcreteTypes {
            #concrete_types_toks
        }
        /// all types
        enum Types {
            #types_toks
        }
        impl Types {
            pub fn parse_xml(t: &str) -> Self {
                match t {
                    #into_types_toks
                }
            }
        }
        mod abstract_types {
            #abstract_types_toks
        }
    };
    println!("{}", res);
    let res = syn::parse_file(&res.to_string()).unwrap();
    let res = prettyplease::unparse(&res);
    println!("{}", res);
}

#[derive(Default)]
struct HM {
    unamed: BTreeMap<String, String>,
    esc_c: u32,
}

impl HM {
    fn fmt(&mut self, x: &str, f: impl Fn(&str) -> String) -> String {
        if let Some(v) = self.unamed.get(x) {
            v.to_string()
        } else {
            let value = f(&self.esc_c.to_string());
            self.unamed.insert(x.to_string(), value);
            self.esc_c += 1;
            self.unamed.get(x).unwrap().to_string()
        }
    }
}
