use tree_sitter::{Tree, TreeCursor};

pub mod tags;

pub mod highlights;

pub fn ts_query_tree_from_str(input: &str) -> Tree {
    let mut query_parser = tree_sitter::Parser::new();
    query_parser
        .set_language(&tree_sitter_query::language())
        .unwrap();
    query_parser.parse(input, None).unwrap()
}

mod error;

type Error = error::LocatedError<error::StringlyError>;

#[derive(Debug)]
struct Query {
    /// path to named field in pattern
    variables: Vec<(String, Vec<usize>)>, //HashMap<String, Vec<usize>>,
    pattern: Patt,
}

impl Query {
    fn parse_query(input: &[u8], cursor: &mut TreeCursor) -> Result<Query, Error> {
        let mut pattern_parser = PatternParser {
            input,
            cursor,
            variables: Default::default(),
            current_path: Default::default(),
        };
        // TODO test variables ie. check if names at given paths correspond
        let pattern = pattern_parser.parse()?;
        Ok(Query {
            variables: pattern_parser.variables,
            pattern,
        })
    }
}

struct PatternParser<'a, 'b> {
    input: &'a [u8],
    cursor: &'a mut TreeCursor<'b>,
    // variables: HashMap<String, Vec<usize>>,
    variables: Vec<(String, Vec<usize>)>,
    current_path: Vec<usize>,
}

impl<'a, 'b> PatternParser<'a, 'b> {
    fn parse(&mut self) -> Result<Patt, Error> {
        let node = self.cursor.node();
        match node.kind() {
            "named_node" | "anonymous_node" => {
                // dbg!(self.cursor.node().utf8_text(self.input).unwrap());
                // dbg!(self.cursor.node().to_sexp());
                self.cursor.goto_first_child();
                self.current_path.push(0);
                let mut patt = vec![];
                let mut captures = vec![];
                let mut kind = None;
                loop {
                    let ts_role = self.cursor.field_name();
                    if !self.cursor.node().is_named() {
                        if self.cursor.node().kind() == "_" {
                            kind = Some("_");
                        }
                    } else if self.cursor.node().kind() == "anonymous_node" {
                        dbg!(self.cursor.node().utf8_text(self.input).unwrap());
                        dbg!(self.cursor.node().to_sexp());
                        let r = self.parse()?;
                        patt.push(r);
                        *self.current_path.last_mut().unwrap() += 1;
                        // panic!()
                    } else if self.cursor.node().kind() == "list" {
                        dbg!(self.cursor.node().utf8_text(self.input).unwrap());
                        dbg!(self.cursor.node().to_sexp());
                        // unimplemented!()
                        // TODO do not ignore
                    } else if self.cursor.node().kind() == "quantifier" {
                        // TODO do not ignore
                    } else if self.cursor.node().kind() == "negated_field" {
                        // dbg!(self.cursor.node().utf8_text(self.input).unwrap());
                        // dbg!(self.cursor.node().to_sexp());
                        // TODO do not ignore
                    } else if let Some(ts_role) = ts_role {
                        assert_eq!("name", ts_role);
                        let node = self.cursor.node();
                        assert_eq!("identifier", node.kind());
                        assert_eq!(kind, None);
                        let value = node
                            .utf8_text(self.input)
                            .map_err(|x| format!("Utf8Error at converting kind: {}", x).into())?;
                        kind = Some(value);
                    } else if self.cursor.node().kind() == "capture" {
                        let ident = self.cursor.node().child_by_field_name("name").unwrap();
                        assert!(ident.kind() == "identifier");
                        let ident = ident.utf8_text(self.input).map_err(|x| {
                            format!("Utf8Error at converting identifier: {}", x).into()
                        })?;
                        // dbg!(&self.variables);
                        self.variables.push((
                            ident.to_string(),
                            self.current_path[..self.current_path.len() - 1].to_vec(),
                        ));
                        captures.push(ident.to_string());
                    } else {
                        dbg!();
                        let r = self.parse()?;
                        patt.push(r);
                        *self.current_path.last_mut().unwrap() += 1;
                    }

                    if !self.cursor.goto_next_sibling() {
                        break;
                    }
                }
                assert!(self.cursor.goto_parent(), "missed a goto_parent");
                self.current_path.pop().unwrap();
                assert_eq!(
                    self.cursor.node(),
                    node,
                    "should have gone back to same query node"
                );
                Ok(Patt::Node {
                    kind: kind.ok_or("missing type of node".into())?.to_string(),
                    patt,
                    captures,
                })
            }
            "list" => {
                self.cursor.goto_first_child();
                let mut patt = vec![];
                loop {
                    if !self.cursor.node().is_named() {
                    } else if self.cursor.node().kind() == "anonymous_node" {
                        dbg!(self.cursor.node().utf8_text(self.input).unwrap());
                        dbg!(self.cursor.node().to_sexp());
                        let r = self.parse()?;
                        patt.push(r);
                        // panic!()
                    } else if self.cursor.node().kind() == "list" {
                        dbg!(self.cursor.node().utf8_text(self.input).unwrap());
                        dbg!(self.cursor.node().to_sexp());
                        unimplemented!()
                    } else {
                        let r = self.parse()?;
                        patt.push(r);
                    }
                    if !self.cursor.goto_next_sibling() {
                        break;
                    }
                }
                assert!(self.cursor.goto_parent(), "missed a goto_parent");
                assert_eq!(
                    self.cursor.node(),
                    node,
                    "should have gone back to same query node"
                );
                Ok(Patt::Alternation { patt })
            }
            "grouping" => {
                println!("{}", self.cursor.node().utf8_text(self.input).unwrap());
                dbg!(self.cursor.node().to_sexp());
                self.cursor.goto_first_child();
                let mut patt = None;
                let mut captures_with_predicates = vec![];
                loop {
                    if !self.cursor.node().is_named() {
                    } else if self.cursor.node().kind() == "anonymous_node" {
                        dbg!(self.cursor.node().utf8_text(self.input).unwrap());
                        dbg!(self.cursor.node().to_sexp());
                        panic!()
                    } else if self.cursor.node().kind() == "predicate" {
                        let node = self.cursor.node();
                        let parameters = node.child_by_field_name("parameters").unwrap();
                        let pred = node.utf8_text(self.input).unwrap();
                        for _c in parameters.children(&mut self.cursor.clone()) {
                            if node.kind() == "capture" {
                                captures_with_predicates.push((
                                    node.child_by_field_name("name")
                                        .unwrap()
                                        .utf8_text(self.input)
                                        .unwrap(),
                                    pred,
                                ))
                            }
                        }
                    } else {
                        let r = self.parse()?;
                        assert!(patt.is_none());
                        patt = Some(r);
                    }
                    if !self.cursor.goto_next_sibling() {
                        break;
                    }
                }
                assert!(self.cursor.goto_parent(), "missed a goto_parent");
                assert_eq!(
                    self.cursor.node(),
                    node,
                    "should have gone back to same query node"
                );
                Ok(match patt.expect("a pattern") {
                    Patt::Node {
                        kind,
                        patt,
                        captures,
                    } => Patt::Predicated {
                        kind: kind,
                        patt,
                        captures_with_predicates: captures
                            .into_iter()
                            .map(|cap| {
                                let x = vec_extract_if_polyfill::MakeExtractIf::extract_if(
                                    &mut captures_with_predicates,
                                    |(c, _)| *c == cap,
                                )
                                .map(|(_x, pred)| pred.to_string())
                                .collect();
                                (cap, x)
                            })
                            .collect(),
                    },
                    Patt::FieldDefinition { field: _, patt: _ } => unreachable!(),
                    Patt::Alternation { patt: _ } => unimplemented!(),
                    Patt::Predicated {
                        kind: _,
                        patt: _,
                        captures_with_predicates: _,
                    } => panic!("possible ? grammar too lenient"),
                })
            }
            "field_definition" => {
                self.cursor.goto_first_child();
                let mut patt = None;
                let mut field = None;
                loop {
                    let ts_role = self.cursor.field_name();
                    if !self.cursor.node().is_named() {
                    } else if self.cursor.node().kind() == "anonymous_node" {
                        dbg!(self.cursor.node().utf8_text(self.input).unwrap());
                        dbg!(self.cursor.node().to_sexp());
                        panic!()
                    } else if self.cursor.node().kind() == "list" {
                        dbg!(self.cursor.node().utf8_text(self.input).unwrap());
                        dbg!(self.cursor.node().to_sexp());
                        let r = self.parse()?;
                        patt = Some(r);
                        // self.cursor.goto_parent();
                        // println!("{}",self.cursor.node().utf8_text(self.input).unwrap());
                        // self.cursor.goto_parent();
                        // println!("{}",self.cursor.node().utf8_text(self.input).unwrap());
                        // panic!()
                    } else if let Some(ts_role) = ts_role {
                        assert_eq!("name", ts_role);
                        let node = self.cursor.node();
                        assert_eq!("identifier", node.kind());
                        assert_eq!(field, None);
                        field =
                            Some(node.utf8_text(self.input).map_err(|x| {
                                format!("Utf8Error at converting kind: {}", x).into()
                            })?)
                    } else {
                        let r = self.parse()?;
                        patt = Some(r);
                    }
                    if !self.cursor.goto_next_sibling() {
                        break;
                    }
                }
                assert!(self.cursor.goto_parent(), "missed a goto_parent");
                assert_eq!(
                    self.cursor.node(),
                    node,
                    "should have gone back to same query node"
                );
                Ok(Patt::FieldDefinition {
                    field: field.expect("missing field of node").to_string(),
                    patt: Box::new(patt.expect("a pattern")),
                })
            }
            "program" => Err("nothing to do".into()),
            x => {
                dbg!(self.cursor.node().utf8_text(self.input).unwrap());
                panic!("{} not handled", x)
            } //Err(format!("{} not handled", x).into()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Patt {
    FieldDefinition {
        field: String,
        patt: Box<Patt>,
    },
    Node {
        kind: String,
        patt: Vec<Patt>,
        captures: Vec<String>,
    },
    Predicated {
        kind: String,
        patt: Vec<Patt>,
        #[allow(unused)]
        captures_with_predicates: Vec<(String, Vec<String>)>,
    },
    Alternation {
        patt: Vec<Patt>,
    },
}
