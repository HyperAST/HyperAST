use derive_deref::Deref;

use super::*;

type NodeIdentifier = hecs::Entity;

pub(crate) struct TypeSys {
    index: BTreeMap<String, hecs::Entity>,
    pub(crate) abstract_types: World,
}

impl Debug for TypeSys {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (t, e) in &self.index {
            let v = self.abstract_types.entity(*e).unwrap();
            writeln!(f, "{:?}: {:?}", t, e)?;
            if v.get::<&Named>().is_some() {
                writeln!(f, "\tnamed")?;
            }
            if let Some(st) = v.get::<&SubTypes>() {
                writeln!(f, "\tsubtypes: {:?}", st.0)?;
            }
            if let Some(fi) = v.get::<&Fields>() {
                writeln!(f, "\tfields: {:?}", fi.0)?;
            }
            if let Some(cs) = v.get::<&DChildren>() {
                writeln!(f, "\tchildren: {:?}", cs.0)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Deref)]
pub(crate) struct T(String);
#[derive(Debug)]
pub(crate) struct Named;
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
pub(crate) struct SubTypes(Vec<NodeIdentifier>);
#[derive(Debug, Deref)]
pub(crate) struct Fields(Vec<NodeIdentifier>);
#[derive(Debug, Deref)]
pub(crate) struct Role(String);
#[derive(Debug, Deref)]
pub(crate) struct DChildren(Vec<NodeIdentifier>);

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

pub(crate) fn get_token_hierarchy(types: Vec<TsType>, escape: bool) -> TypeSys {
    let mut world = World::new();
    let mut names = BTreeMap::<String, NodeIdentifier>::default();
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

    TypeSys {
        index: names,
        abstract_types: world,
    }
}
