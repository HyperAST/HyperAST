use derive_deref::Deref;

use super::*;

type NodeIdentifier = hecs::Entity;

#[derive(Default)]
pub(crate) struct TypeSys {
    pub(crate) list: Vec<hecs::Entity>,
    pub(crate) index: BTreeMap<String, hecs::Entity>,
    pub(crate) types: World,
}

impl Debug for TypeSys {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // for (t, e) in &self.index {
        for e in &self.list {
            let v = self.types.entity(*e).unwrap();
            writeln!(f, "{:?}: {:?}", v.get::<&T>().unwrap().0, e)?;
            if v.has::<Named>() {
                writeln!(f, "\tnamed")?;
            }
            if let Some(st) = v.get::<&SubTypes>() {
                writeln!(f, "\tsubtypes: {:?}", st.0)?;
            }
            if let Some(fi) = v.get::<&Fields>() {
                writeln!(f, "\tfields: {:?}", fi.0)?;
            }
            if let Some(cs) = v.get::<&DChildren>() {
                if v.has::<MultipleChildren>() {
                    if v.has::<RequiredChildren>() {
                        writeln!(f, "\tchildren: + {:?}", cs.0)?;
                    } else {
                        writeln!(f, "\tchildren: * {:?}", cs.0)?;
                    }
                } else if v.has::<RequiredChildren>() {
                    writeln!(f, "\tchildren: ! {:?}", cs.0)?;
                } else {
                    writeln!(f, "\tchildren: {:?}", cs.0)?;
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Deref)]
pub(crate) struct T(pub(crate) String);
#[derive(Debug)]
pub(crate) struct Named;
#[derive(Debug)]
pub(crate) struct Hidden;
#[derive(Debug)]
pub(crate) struct SubType;
#[derive(Debug)]
pub(crate) struct Field;
#[derive(Debug)]
pub(crate) struct Child;
#[derive(Debug)]
pub(crate) struct MultipleChildren;
#[derive(Debug)]
pub(crate) struct RequiredChildren;
#[derive(Debug, Deref)]
pub(crate) struct SubTypes(pub(crate) Vec<NodeIdentifier>);
#[derive(Debug, Deref)]
pub(crate) struct Fields(pub(crate) Vec<NodeIdentifier>);
#[derive(Debug, Deref)]
pub(crate) struct Role(pub(crate) String);
#[derive(Debug, Deref)]
pub(crate) struct DChildren(pub(crate) Vec<NodeIdentifier>);

impl Display for T {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum TsType {
    AbstractType {
        r#type: String,
        named: bool,
        subtypes: Vec<TsType>,
    },
    ConcreteType {
        r#type: String,
        named: bool,
        fields: HashMap<String, Chidlren>,
        children: Option<Chidlren>,
    },
    Leaf {
        r#type: String,
        named: bool,
    },
}

impl TsType {
    fn ty(&self) -> &str {
        match self {
            TsType::AbstractType { r#type, .. } => r#type,
            TsType::ConcreteType { r#type, .. } => r#type,
            TsType::Leaf { r#type, .. } => r#type,
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct Chidlren {
    multiple: bool,
    required: bool,
    types: Vec<TsType>,
}

pub(crate) fn consider_tags(tags: ts_metadata::Tags, typesys : &mut TypeSys) {
}

pub(crate) fn consider_highlights(tags: ts_metadata::HighLights, typesys : &mut TypeSys) {
}

impl TypeSys {
    pub(crate) fn new(lang: Language, types: Vec<TsType>) -> Self {
        let mut r = Self {
            list: Default::default(),
            index: Default::default(),
            types: Default::default(),
        };
        r.add_token_hierarchy(types);
        r.mod_token_hierarchy(lang);
        r
    }
    fn mod_token_hierarchy(&mut self, language: Language) {
        let count = language.node_kind_count();
        for i in 0..count {
            let named = language.node_kind_is_named(i as u16);
            let visible = language.node_kind_is_visible(i as u16);
            let kind = language.node_kind_for_id(i as u16).unwrap();
            dbg!(named);
            dbg!(visible);
            dbg!(kind);
            // let name = kind.to_string();//sanitize_identifier(kind);
            // let ts_name = kind.to_string();//sanitize_string(kind, escape);
            match self.index.entry(kind.to_string()) {
                std::collections::btree_map::Entry::Vacant(vac) => {
                    let mut builder = EntityBuilder::new();
                    if named {
                        builder.add(Named);
                    }
                    if !visible {
                        builder.add(Hidden);
                    }
                    let t = kind.to_string();
                    dbg!(&t);
                    builder.add(T(t));
                    let ent = self.types.spawn(builder.build());
                    vac.insert(ent);
                    self.list.push(ent);
                },
                std::collections::btree_map::Entry::Occupied(occ) => {
                    self.list.push(occ.get().clone())
                },
            }
            // let name = camel_case(name);
            // use std::collections::hash_map::Entry;
            // let e = match name_count.entry(name.clone()) {
            //     Entry::Occupied(mut e) => {
            //         *e.get_mut() += 1;
            //         (format!("{}{}", name, e.get()), true, ts_name)
            //     }
            //     Entry::Vacant(e) => {
            //         e.insert(1);
            //         (name, false, ts_name)
            //     }
            // };
            // names.insert(i, e);
        }
    }
    fn add_token_hierarchy(&mut self, types: Vec<TsType>, ) {
        let mut world = &mut self.types;
        let mut names = &mut self.index;
        for ty in types {
            let mut builder = EntityBuilder::new();
            match ty {
                TsType::AbstractType {
                    r#type: t,
                    named,
                    subtypes,
                } => {
                    if named {
                        builder.add(Named);
                    }
                    builder.add(T(t));
                    builder.add(SubTypes(Vec::with_capacity(subtypes.len())));
                    builder.add(subtypes);
                }
                TsType::ConcreteType {
                    r#type: t,
                    named,
                    fields,
                    children,
                } => {
                    if named {
                        builder.add(Named);
                    }
                    builder.add(T(t));
                    if !fields.is_empty() {
                        builder.add(Fields(
                            fields
                                .into_iter()
                                .map(|(r, children)| {
                                    let mut builder = EntityBuilder::new();
                                    builder.add_bundle((
                                        Role(r),
                                        DChildren(Vec::with_capacity(children.types.len())),
                                        children.types,
                                    ));
                                    if children.multiple {
                                        builder.add(MultipleChildren);
                                    }
                                    if children.required {
                                        builder.add(RequiredChildren);
                                    }
                                    world.spawn(builder.build())
                                })
                                .collect(),
                        ));
                    }
                    if let Some(children) = children {
                        if children.multiple {
                            builder.add(MultipleChildren);
                        }
                        if children.required {
                            builder.add(RequiredChildren);
                        }
                        builder.add(DChildren(Vec::with_capacity(children.types.len())));
                        builder.add(children.types);
                    }
                }
                TsType::Leaf { r#type: t, named } => {
                    if named {
                        builder.add(Named);
                    }
                    builder.add(T(t));
                }
            };
            let t: &T = builder.get().unwrap();
            let t = t.0.clone();
            let e = world.spawn(builder.build());
            names.insert(t, e);
        }
    
        let mut cmd = CommandBuffer::new();
        world
            .query_mut::<(&mut SubTypes, &mut Vec<TsType>)>()
            .into_iter()
            .for_each(|(e, (s, v))| {
                v.drain(..)
                    .map(|x| names.get(x.ty()).unwrap().to_owned())
                    .collect_into(&mut s.0);
                cmd.remove::<(Vec<TsType>,)>(e);
                for e in &mut s.0 {
                    cmd.insert_one(*e, SubType);
                }
            });
        cmd.run_on(&mut world);
        world
            .query_mut::<(&mut DChildren, &mut Vec<TsType>)>()
            .into_iter()
            .for_each(|(e, (s, v))| {
                // dbg!(&v);
                // dbg!(&names);
                v.drain(..)
                    .filter_map(|x| names.get(x.ty()).copied())
                    .collect_into(&mut s.0);
                cmd.remove::<(Vec<TsType>,)>(e);
                for e in &mut s.0 {
                    cmd.insert_one(*e, Child);
                }
            });
        cmd.run_on(&mut world);
    }
}

pub(crate) fn get_token_hierarchy(types: Vec<TsType>, escape: bool) -> TypeSys {
    let mut r = TypeSys {
        list: Default::default(),
        index: Default::default(),
        types: Default::default(),
    };
    r.add_token_hierarchy(types);
    r
}


pub fn get_token_names(language: &Language, escape: bool) -> Vec<(String, bool, String)> {
    let count = language.node_kind_count();
    let mut names: BTreeMap<usize, (String, bool, String)> = BTreeMap::default();
    let mut name_count = HashMap::new();
    // for anon in &[false, true] {
        for i in 0..count {
            let named = language.node_kind_is_named(i as u16);
            let visible = language.node_kind_is_visible(i as u16);
            // if anonymous != *anon {
            //     continue;
            // }
            let kind = language.node_kind_for_id(i as u16).unwrap();
            dbg!(named);
            dbg!(visible);
            dbg!(kind);
            let name = kind.to_string();//sanitize_identifier(kind);
            let ts_name = kind.to_string();//sanitize_string(kind, escape);
            let name = camel_case(name);
            use std::collections::hash_map::Entry;
            let e = match name_count.entry(name.clone()) {
                Entry::Occupied(mut e) => {
                    *e.get_mut() += 1;
                    (format!("{}{}", name, e.get()), true, ts_name)
                }
                Entry::Vacant(e) => {
                    e.insert(1);
                    (name, false, ts_name)
                }
            };
            names.insert(i, e);
        }
    // }
    let mut names: Vec<_> = names.values().cloned().collect();
    names.push(("Error".to_string(), false, "ERROR".to_string()));

    names
}