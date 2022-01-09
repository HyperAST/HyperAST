use std::{
    collections::BTreeMap, fs::File, io, os::unix::prelude::FileExt, path::Path, result::Result,
};

use heck::CamelCase;
use quote::{__private::TokenStream, format_ident, quote};
use serde::{self, Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Default, PartialOrd, Ord)]
pub(crate) struct NodeInfoJSON {
    #[serde(rename = "type")]
    kind: String,
    named: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    fields: Option<BTreeMap<String, FieldInfoJSON>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    children: Option<FieldInfoJSON>,
    #[serde(skip_serializing_if = "Option::is_none")]
    subtypes: Option<Vec<NodeTypeJSON>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct NodeTypeJSON {
    #[serde(rename = "type")]
    kind: String,
    named: bool,
}

const fn default_as_true() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct FieldInfoJSON {
    #[serde(default = "bool::default")]
    multiple: bool, // false
    #[serde(default = "default_as_true")]
    required: bool, // true
    types: Vec<NodeTypeJSON>,
}

pub fn gen_types_from_ts_json(aa: &Path, out: &Path) -> std::result::Result<(), io::Error> {
    let x = read_types_from_file(&aa).unwrap();
    let mut r = quote! {};
    let mut m: HM = Default::default();
    x.into_iter().for_each(|x| {
        let y = node_info_to_struct(&mut m, x);
        r.extend(y)
    });
    let file = File::create(out.to_str().unwrap()).unwrap();
    file.write_all_at(r.to_string().as_bytes(), 0)?;
    Ok(())
}

pub fn gen_enum_from_ts_json(aa: &Path, out: &Path) -> std::result::Result<(), io::Error> {
    let x = read_types_from_file(&aa).unwrap();
    let mut r = quote! {};
    let mut m: HM = Default::default();
    x.into_iter().for_each(|x| {
        let y = node_info_to_enum(&mut m, x);
        r.extend(y)
    });
    let r = quote! {
        enum A {
            #r
        }
    };
    let file = File::create(out.to_str().unwrap()).unwrap();
    file.write_all_at(r.to_string().as_bytes(), 0)?;
    Ok(())
}

#[derive(Default)]
struct HM {
    unamed: BTreeMap<String, String>,
    esc_c: u32,
}

impl HM {
    fn fmt(&mut self, x: &String) -> String {
        if let Some(v) = self.unamed.get(x) {
            v
        } else {
            self.unamed.insert(x.to_string(), self.esc_c.to_string());
            self.esc_c += 1;
            &self.unamed.get(x).unwrap()
        }
        .to_string()
    }
}

fn node_info_to_enum(m: &mut HM, x: NodeInfoJSON) -> TokenStream {
    if !x.named {
        // leaf/token

        assert_eq!(x.children, None);
        assert_eq!(x.fields, None);
        assert_eq!(x.subtypes, None);
        let kind = m.fmt(&x.kind);
        let kind = format_ident!("TS{}", &kind.to_camel_case());
        let raw = x.kind.clone();

        quote! {
            #[strum(serialize = #raw)]
            #kind,
        }
    } else {
        let kind = format_ident!("{}", &x.kind.to_camel_case());
        quote! {
            #kind,
        }
    }
}

fn node_info_to_struct(m: &mut HM, x: NodeInfoJSON) -> TokenStream {
    if !x.named {
        // leaf/token

        assert_eq!(x.children, None);
        assert_eq!(x.fields, None);
        assert_eq!(x.subtypes, None);
        let kind = m.fmt(&x.kind);
        let kind = format_ident!("TSType{}", &kind.to_camel_case());
        let raw = x.kind.clone();

        return quote! {
            struct #kind {

            }

            impl Display for #kind {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    f.write_str(#raw)
                }
            }
        };
    }

    let kind = format_ident!("TSType{}", &x.kind.to_camel_case());
    let mut helpers = quote! {};
    let mut impls = quote! {};

    if let Some(subtypes) = &x.subtypes {
        assert_eq!(x.fields, None);
        assert_eq!(x.children, None);

        let (ed,edp): (Vec<_>,Vec<_>) = subtypes
            .iter()
            .map(|x| {
                let x = if !x.named {
                    m.fmt(&x.kind)
                } else {
                    x.kind.to_camel_case()
                };
                let a = format_ident!("TSField{}", &x);
                let b = format_ident!("TSType{}", &x);
                (quote! {
                    #a(#b)
                },quote! {
                    #a(x)
                })
            })
            .unzip();


        impls.extend(quote! {
            impl Display for #kind {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    match self {
                        #( #ed => x.fmt(f)),*
                    }
                }
            }
        });

        quote! {
            #helpers

            enum #kind {
                #( #ed),*
            }

            #impls
        }
    } else {
        let children = if let Some(children) = &x.children {
            assert_eq!(x.subtypes, None);

            let e = format_ident!("TSChildren{}", &x.kind.to_camel_case());
            let ed: Vec<_> = children
                .types
                .iter()
                .map(|x| {
                    let a = if !x.named {
                        m.fmt(&x.kind)
                    } else {
                        x.kind.to_camel_case()
                    };
                    let x = format_ident!("TSType{}", &a);
                    quote! {
                            #x(#x)
                    }
                })
                .collect();

            let cs = if children.multiple {
                quote! {
                    Vec<#e>
                }
            } else {
                quote! {
                    #e
                }
            };
            let cs = if children.required {
                quote! {
                    #cs
                }
            } else {
                quote! {
                    Option<#cs>
                }
            };

            helpers.extend(quote! {
                enum #e {
                    #( #ed),*
                }
            });

            Some(quote! {_children: #cs,})
        } else {
            None
        };

        let fields = if let Some(fields) = &x.fields {
            assert_eq!(x.subtypes, None);

            let mut enums = vec![];
            let fields: Vec<_> = fields
                .iter()
                .map(|(k, v)| {
                    let e =
                        format_ident!("TSField{}{}", &x.kind.to_camel_case(), k.to_camel_case());

                    let ed: Vec<_> = v
                        .types
                        .iter()
                        .map(|x| {
                            let x = if !x.named {
                                m.fmt(&x.kind)
                            } else {
                                x.kind.to_camel_case()
                            };
                            let a = format_ident!("TSField{}", &x);
                            let b = format_ident!("TSType{}", &x);
                            quote! {
                                #a(#b)
                            }
                        })
                        .collect();

                    enums.push(quote! {
                        enum #e {
                            #( #ed),*
                        }
                    });

                    let w = if v.multiple {
                        quote! {
                            Vec<#e>
                        }
                    } else {
                        quote! {
                            #e
                        }
                    };

                    let k = format_ident!("TSField{}", k.to_camel_case());
                    if v.required {
                        quote! {
                            #k:#w
                        }
                    } else {
                        quote! {
                            #k:Option<#w>
                        }
                    }
                })
                .collect();

            Some(quote! {
                #( #fields ),*
            })
        } else {
            None
        };

        let raw = if fields.is_none() && children.is_none() {
            x.kind.clone()
        } else {
            "?".to_string()
        };

        impls.extend(quote! {
            impl Display for #kind {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    f.write_str(#raw)
                }
            }
        });

        quote! {
            #helpers

            struct #kind {
                #children
                #fields
            }

            #impls
        }
    }
}

fn read_types_from_file<P: AsRef<Path>>(path: P) -> Result<Vec<NodeInfoJSON>, io::Error> {
    // Open the file in read-only mode with buffer.
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);

    // Read the JSON contents of the file as an instance of `User`.
    let u = serde_json::from_reader(reader)?;

    // Return the `User`.
    Ok(u)
}
