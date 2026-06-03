use std::{collections::HashMap, fmt::Display, str::FromStr};

use tree_sitter::TreeCursor;

type TagedRole = String;

use super::{ts_query_tree_from_str, Error, Patt, Query};

#[derive(Debug, Default)]
pub struct Tags {
    pub(crate) declarations: HashMap<TagedRole, Tag>,
    pub(crate) references: HashMap<TagedRole, Tag>,
}
impl Display for Tags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "declarations:")?;
        for (k, v) in &self.declarations {
            writeln!(f, "\t{}: {}", k, v)?;
        }
        writeln!(f, "references:")?;
        for (k, v) in &self.references {
            writeln!(f, "\t{}: {}", k, v)?;
        }
        Ok(())
    }
}

impl FromStr for Tags {
    type Err = Error;

    fn from_str(tags: &str) -> Result<Self, Self::Err> {
        let tree = ts_query_tree_from_str(tags);
        let root_node = tree.root_node();
        let cursor = root_node.walk();
        Tags::parse(tags.as_bytes(), cursor)
    }
}

impl Tags {
    fn parse(input: &[u8], mut cursor: TreeCursor) -> Result<Self, Error> {
        let mut declarations = HashMap::default();
        let mut references = HashMap::default();
        if cursor.node().kind() != "program" {
            return Err("tree root should be a 'program'".into());
        }
        cursor.goto_first_child();
        loop {
            // dbg!(cursor.node().to_sexp());
            match cursor.node().kind() {
                "comment" => {
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                    continue;
                }
                _ => (),
            }

            let tag = Tag::parse(input, &mut cursor)?;
            // dbg!(tag.0.clone());
            let mut iter_split = tag.0.split(".");
            if let Some(cat) = iter_split.next() {
                let rest = iter_split.next().unwrap();
                for tag in tag.1 {
                    if cat == "reference" {
                        references.insert(rest.to_string(), tag);
                    } else if cat == "definition" {
                        declarations.insert(rest.to_string(), tag);
                    } else {
                        return Err(format!("bad category name: {}", cat).into());
                    }
                }
                assert!(iter_split.next().is_none());
            }
            // dbg!(tag.0.split(".").collect::<Vec<_>>());

            if !cursor.goto_next_sibling() {
                break;
            }
        }
        if !cursor.goto_parent() {
            return Err("should be at root".into());
        }
        if cursor.node().kind() != "program" {
            return Err("tree root should be a 'program'".into());
        }
        if cursor.goto_parent() {
            return Err("program should be the root".into());
        }

        Ok(Self {
            declarations,
            references,
        })
    }
}

#[derive(Debug)]
pub(crate) struct Tag {
    /// path to named field in pattern
    name: Vec<usize>,
    pattern: Patt,
}
impl Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            unimplemented!()
        } else {
            write!(f, " -> ")?;
            let mut pattern = &self.pattern;
            for i in &self.name {
                if let Patt::FieldDefinition { field, patt } = pattern {
                    write!(f, "{}: ", field)?;
                    pattern = &patt;
                };
                match pattern {
                    Patt::Node {
                        kind,
                        patt,
                        captures: _,
                    } => {
                        write!(f, "{}", kind)?;
                        pattern = &patt[*i];
                    }
                    _ => unreachable!(),
                }
                write!(f, " ")?;
            }
            if let Patt::FieldDefinition { field, patt } = pattern {
                write!(f, "{}: ", field)?;
                pattern = &patt;
            };
            match pattern {
                Patt::Node {
                    kind,
                    patt: _,
                    captures,
                } => {
                    write!(f, "{}", kind)?;
                    assert!(captures.contains(&"name".to_string()));
                }
                _ => panic!("{:?}", pattern),
            }
            write!(f, "")?;
        }
        Ok(())
    }
}

impl Tag {
    fn parse(input: &[u8], cursor: &mut TreeCursor) -> Result<(TagedRole, Vec<Tag>), Error> {
        let Query {
            mut variables,
            pattern,
        } = Query::parse_query(input, cursor)?;
        // dbg!(&pattern);
        // dbg!(&variables);
        let mut names =
            vec_extract_if_polyfill::MakeExtractIf::extract_if(&mut variables, |x| x.0 == "name")
                .collect::<Vec<_>>();
        if names.len() == 0 {
            return Err("Missing name variable".into())?;
        }
        dbg!(&variables);
        let role = variables
            .into_iter()
            .rev()
            .next()
            .ok_or_else(|| "Missing tag variable name variable".into())
            .and_then(|x| {
                dbg!(&x);
                if x.1.len() > 1 {
                    Ok(x.0) // should be ok actually
                            // Err("missplaced tag variable name".into())
                } else {
                    Ok(x.0)
                }
            })?;
        if names.len() == 1 {
            let (_, name) = names.remove(0);
            Ok((role, vec![Tag { name, pattern }]))
        } else {
            let pattern = Alternate {}.alternate(pattern, vec![]);
            assert_eq!(pattern.len(), names.len());
            let pattern = pattern
                .into_iter()
                .map(|(name, pattern)| Tag { name, pattern })
                .collect();
            Ok((role, pattern))
        }
    }
}

struct Alternate {}

impl Alternate {
    // precond: path are ordered by position
    fn alternate(&mut self, pattern: Patt, path: Vec<usize>) -> Vec<(Vec<usize>, Patt)> {
        match pattern {
            Patt::FieldDefinition { field, patt } => {
                let patt = self.alternate(*patt, path);
                patt.into_iter()
                    .map(|(name, patt)| {
                        (
                            name,
                            Patt::FieldDefinition {
                                field: field.clone(),
                                patt: Box::new(patt),
                            },
                        )
                    })
                    .collect()
            }
            Patt::Node {
                kind,
                patt,
                captures,
            } => {
                let mut r = vec![];

                for _ in captures.iter().filter(|c| *c == "name") {
                    r.push((
                        path.clone(),
                        Patt::Node {
                            kind: kind.clone(),
                            patt: patt.clone(),
                            captures: captures.clone(),
                        },
                    ));
                }

                for i in 0..patt.len() {
                    let mut path = path.clone();
                    path.push(i);
                    self.alternate(patt[i].clone(), path.clone())
                        .into_iter()
                        .for_each(|(n, p)| {
                            let mut pa = patt.clone();
                            pa[i] = p;
                            r.push((
                                n,
                                Patt::Node {
                                    kind: kind.clone(),
                                    patt: pa,
                                    captures: captures.clone(),
                                },
                            ));
                        });
                }

                r
                // if let Some(offset) = self.path.next() {
                //     match patt.get_mut(index).unwrap() {
                //         Patt::FieldDefinition { field, patt } => {}
                //         patt => {
                //             let p = self.alternate(patt);
                //             p
                //         }
                //     }
                // } else {
                //     pattern.clone()
                // }
            }
            Patt::Predicated {
                kind: _,
                patt: _,
                captures_with_predicates: _,
            } => {
                unimplemented!()
            }
            Patt::Alternation { patt } => {
                let mut r = vec![];
                for patt in patt {
                    self.alternate(patt, path.clone())
                        .into_iter()
                        .for_each(|(n, p)| {
                            r.push((n, p));
                        });
                }
                r
            }
        }
    }
}
