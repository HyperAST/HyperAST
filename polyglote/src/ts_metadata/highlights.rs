use std::{collections::BTreeMap, fmt::Display, str::FromStr};

use tree_sitter::TreeCursor;

use super::{ts_query_tree_from_str, Error, Patt, Query};

#[derive(Debug)]
pub struct HighLights {
    per_cat: BTreeMap<String, Vec<(String, Vec<usize>, usize)>>,
    patterns: Vec<Patt>,
}

impl FromStr for HighLights {
    type Err = Error;

    fn from_str(tags: &str) -> Result<Self, Self::Err> {
        let tree = ts_query_tree_from_str(tags);
        let root_node = tree.root_node();
        let cursor = root_node.walk();
        HighLights::parse(tags.as_bytes(), cursor)
    }
}

impl HighLights {
    pub fn get(&self, variable: &str) -> Option<Vec<(&[usize], &Patt)>> {
        let (first, second) = variable.split_once(".").unwrap_or((variable, ""));
        let cat = self.per_cat.get(first)?;
        let mut r = vec![];
        for (c, x, idx) in cat {
            let patt = self.patterns.get(*idx).expect("a pattern");
            if c == second || second == "*" {
                r.push((x.as_ref(), patt));
            }
        }
        Some(r)
    }
}

impl Display for HighLights {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (cat, v) in &self.per_cat {
            for (rest, path, patt) in v {
                write!(f, "{}", cat)?;
                if !rest.is_empty() {
                    write!(f, ".{}", rest)?;
                }
                write!(f, " -> ")?;
                if f.alternate() {
                    // TODO show full query and highlight given variable in query
                    unimplemented!()
                } else {
                    let mut pattern = &self.patterns[*patt];
                    for i in path {
                        // dbg!(rest);
                        // dbg!(pattern);
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
                            Patt::Predicated {
                                kind,
                                patt,
                                captures_with_predicates: _,
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
                            captures: _,
                        } => {
                            write!(f, "{}", kind)?;
                        }
                        Patt::Predicated {
                            kind,
                            patt: _,
                            captures_with_predicates: _,
                        } => {
                            write!(f, "{}", kind)?;
                        }
                        _ => panic!("{:?}", pattern),
                    }
                }
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

impl HighLights {
    // fn compress(
    //     &mut self,
    //     partial: BTreeMap<string_interner::DefaultSymbol, Vec<(Option<String>, Vec<usize>, usize)>>,
    //     pos: usize,
    // ) {
    // }
    fn parse(input: &[u8], mut cursor: TreeCursor) -> Result<Self, Error> {
        let mut partial = BTreeMap::default(); //MultiLTree::Rec(Default::default());
        let mut patterns = vec![];
        // let mut string_interner = StringInterner::default();
        // let mut size = 0;
        if cursor.node().kind() != "program" {
            return Err("tree root should be a 'program'".into());
        }
        cursor.goto_first_child();
        loop {
            let node = cursor.node();
            match node.kind() {
                "comment" => {
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                    continue;
                }
                "list" => {
                    let mut captures = vec![];
                    cursor.goto_first_child();
                    // let mut captures = vec![];
                    let mut patts = vec![];
                    loop {
                        if !cursor.node().is_named() {
                        } else if cursor.node().kind() == "anonymous_node" {
                            let Query { variables, pattern } =
                                Query::parse_query(input, &mut cursor)?;
                            assert!(variables.is_empty());
                            patts.push(patterns.len());
                            patterns.push(pattern);
                        } else if cursor.node().kind() == "named_node" {
                            let Query { variables, pattern } =
                                Query::parse_query(input, &mut cursor)?;
                            assert!(variables.is_empty());
                            patts.push(patterns.len());
                            patterns.push(pattern);
                        } else if cursor.node().kind() == "capture" {
                            let ident = cursor.node().child_by_field_name("name").unwrap();
                            assert!(ident.kind() == "identifier");
                            let ident = ident.utf8_text(input).map_err(|x| {
                                format!("Utf8Error at converting identifier: {}", x).into()
                            })?;
                            captures.push(ident.to_string());
                        } else {
                            unimplemented!("{}", cursor.node().kind())
                        }

                        if !cursor.goto_next_sibling() {
                            break;
                        }
                    }
                    for variable in captures {
                        for patt_idx in &patts {
                            let once = variable.split_once(".");
                            if let Some((_, "")) = once {
                                return Err("blank variable name".into());
                                // rec.insert(first.to_string(), (None, query, pattern));
                            } else if let Some((first, rest)) = once {
                                partial.entry(first.to_string()).or_insert(vec![]).push((
                                    rest.to_string(),
                                    vec![],
                                    *patt_idx,
                                ));
                            } else {
                                partial.entry(variable.to_string()).or_insert(vec![]).push((
                                    "".to_string(),
                                    vec![],
                                    *patt_idx,
                                ));
                            }
                        }
                    }
                    assert!(cursor.goto_parent(), "there is an extra goto_parent");
                    assert_eq!(
                        cursor.node(),
                        node,
                        "should have gone back to same query node"
                    );
                }
                _kind => {
                    // dbg!(kind);
                    let Query { variables, pattern } = Query::parse_query(input, &mut cursor)?;
                    if variables.is_empty() {
                        dbg!(cursor.node().to_sexp());
                        // TODO should not append
                        return Err("no_variables".into());
                    }
                    let patt_idx = patterns.len();
                    patterns.push(pattern);
                    for (variable, query) in variables {
                        // dbg!(&variable);
                        let once = variable.split_once(".");
                        if let Some((_, "")) = once {
                            return Err("blank variable name".into());
                            // rec.insert(first.to_string(), (None, query, pattern));
                        } else if let Some((first, rest)) = once {
                            partial.entry(first.to_string()).or_insert(vec![]).push((
                                rest.to_string(),
                                query,
                                patt_idx,
                            ));
                        } else {
                            partial.entry(variable.to_string()).or_insert(vec![]).push((
                                "".to_string(),
                                query,
                                patt_idx,
                            ));
                        }
                    }
                }
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
        // dbg!(&partial);
        // let mut r = Self {
        //     patt_idx: vec![0; size],
        //     patterns,
        //     vartree: Vec::with_capacity(size),
        //     llds: Vec::with_capacity(size),
        //     string_interner,
        // };
        // r.compress(partial, 0);
        Ok(Self {
            per_cat: partial,
            patterns,
        })
    }
}
