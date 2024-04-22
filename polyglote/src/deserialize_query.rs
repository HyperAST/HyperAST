use std::collections::HashMap;
use std::ops::{AddAssign, MulAssign, Neg};

use error::{Error, Result};
use serde::de::{
    self, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess, VariantAccess,
    Visitor,
};
use serde::Deserialize;
use tree_sitter::{Node, Tree, TreeCursor};



#[derive(Deserialize, Debug)]
#[serde(transparent)]
struct Tags(HashMap<TagedRole, QueryPatt>);

//  {
//     declarations: HashMap<TagedRole, Declaration>,
//     references: HashMap<TagedRole, Reference>,
// }

fn example_use() {
    let tags = r#"
(new_expression
    constructor: (identifier) @name) @reference.class
(new_expression
    constructor: (identifier) @name1) @reference.class1
(new_expression
    constructor: (identifier) @name2) @reference.class2
((identifier) @constant
(#match? @constant "^[A-Z][A-Z_]+")
) @reference.aaa
    "#;
    let tree = deserialize_query::ts_query_tree_from_str(tags);
    let root_node = tree.root_node();
    let cursor = root_node.walk();
    let tags: Tags = deserialize_query::from_str(tags, cursor).unwrap();
    dbg!(tags);
}

type TagedRole = String;

#[derive(Deserialize, Debug)]
struct QueryPatt {
    /// path to named field in pattern
    name: Vec<usize>,
    pattern: Patt,
}

#[derive(Deserialize, Debug)]
enum Patt {
    Leaf(String),
    Role { role: String, patt: Box<Patt> },
    Node { kind: String, patt: Vec<Patt> },
}

pub struct Deserializer<'de> {
    input: &'de [u8],
    cursor: TreeCursor<'de>,
}

impl<'de> Deserializer<'de> {
    pub fn from_ts_cursor(input: &'de str, cursor: TreeCursor<'de>) -> Self {
        let input = input.as_bytes();
        Deserializer { input, cursor }
    }
}

pub fn ts_query_tree_from_str(input: &str) -> Tree {
    let mut query_parser = tree_sitter::Parser::new();
    query_parser
        .set_language(tree_sitter_query::language())
        .unwrap();
    query_parser.parse(input, None).unwrap()
}

// By convention, the public API of a Serde deserializer is one or more
// `from_xyz` methods such as `from_str`, `from_bytes`, or `from_reader`
// depending on what Rust types the deserializer is able to consume as input.
//
// This basic deserializer supports only `from_str`.
pub fn from_str<'a, T>(input: &'a str, cursor: TreeCursor<'a>) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_ts_cursor(input, cursor);
    let t = T::deserialize(&mut deserializer)?;
    if deserializer.cursor.goto_next_sibling() || deserializer.cursor.goto_parent() {
        Err(Error::TrailingCharacters)
    } else {
        Ok(t)
    }
}

// SERDE IS NOT A PARSING LIBRARY. This impl block defines a few basic parsing
// functions from scratch. More complicated formats may wish to use a dedicated
// parsing library to help implement their Serde deserializer.
impl<'de> Deserializer<'de> {
    // // Look at the first character in the input without consuming it.
    // fn peek_char(&mut self) -> Result<char> {
    //     self.input.chars().next().ok_or(Error::Eof)
    // }

    // // Consume the first character in the input.
    // fn next_char(&mut self) -> Result<char> {
    //     let ch = self.peek_char()?;
    //     self.input = &self.input[ch.len_utf8()..];
    //     Ok(ch)
    // }

    // // Parse the JSON identifier `true` or `false`.
    // fn parse_bool(&mut self) -> Result<bool> {
    //     if self.input.starts_with("true") {
    //         self.input = &self.input["true".len()..];
    //         Ok(true)
    //     } else if self.input.starts_with("false") {
    //         self.input = &self.input["false".len()..];
    //         Ok(false)
    //     } else {
    //         Err(Error::ExpectedBoolean)
    //     }
    // }

    fn parse_number<T>(&mut self) -> Result<T> {
        unimplemented!()
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, _visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::FormatIsNotSelfDescribing)
    }

    fn deserialize_bool<V>(self, _visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_i8<V>(self, _visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.parse_number()
    }

    fn deserialize_i16<V>(self, _visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.parse_number()
    }

    fn deserialize_i32<V>(self, _visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.parse_number()
    }

    fn deserialize_i64<V>(self, _visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.parse_number()
    }

    fn deserialize_u8<V>(self, _visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.parse_number()
    }

    fn deserialize_u16<V>(self, _visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.parse_number()
    }

    fn deserialize_u32<V>(self, _visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.parse_number()
    }

    fn deserialize_u64<V>(self, _visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.parse_number()
    }

    fn deserialize_f32<V>(self, _visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.parse_number()
    }

    fn deserialize_f64<V>(self, _visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.parse_number()
    }

    fn deserialize_char<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_str<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_string<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        // should not be reached to access outer name
        todo!()
    }

    fn deserialize_bytes<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_option<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_unit<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        dbg!(name);
        // visitor.visit_map(map);
        todo!()
    }

    fn deserialize_seq<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_tuple<V>(
        self,
        len: usize,
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_map<V>(mut self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        struct MapAux<'a, 'de> {
            inner: &'a mut Deserializer<'de>,
        }
        impl<'a, 'de> MapAccess<'de> for MapAux<'a, 'de> {
            type Error = error::Error;

            fn next_key_seed<K>(
                &mut self,
                seed: K,
            ) -> std::result::Result<Option<K::Value>, Self::Error>
            where
                K: DeserializeSeed<'de>,
            {
                // // directly get the outer name
                // seed.deserialize(&mut *self.inner).map(|x|Some(x))
                assert_eq!(None, self.inner.cursor.field_name());
                let node = self.inner.cursor.node();
                assert_eq!("named_node", node.kind());
                assert_eq!(5, node.child_count());
                let capture = node.child(node.child_count() - 1).expect("");
                assert_eq!("capture", capture.kind());
                let mut cap_de = CaptureDeserializer {
                    input: self.inner.input,
                    node: capture,
                };
                seed.deserialize(&mut cap_de).map(|x| Some(x))
            }

            fn next_value_seed<V>(&mut self, seed: V) -> std::result::Result<V::Value, Self::Error>
            where
                V: DeserializeSeed<'de>,
            {
                let r = seed.deserialize(&mut *self.inner)?;
                self.inner.cursor.goto_next_sibling();
                Ok(r)
            }
        }
        self.cursor.goto_first_child();
        visitor.visit_map(MapAux { inner: &mut self })
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        dbg!(_name);
        let named_node = self.cursor.node();
        assert_eq!("named_node", named_node.kind());
        const PATTERN: &str = "pattern";
        let Some(pattern_pos) = fields.iter().position(|x|x==&PATTERN) else {
            return Err(Error::MISSING_PATTERN_FIELD);
        };

        struct MapAccess<'a> {
            input: &'a [u8],
            cursor: TreeCursor<'a>,
            pos: usize,
            len: usize,
            pattern_pos: usize,
        }

        impl<'de> de::MapAccess<'de> for MapAccess<'de> {
            type Error = Error;

            fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
            where
                K: de::DeserializeSeed<'de>,
            {
                // let peek = match tri!(self.de.parse_whitespace()) {
                //     Some(b'}') => {
                //         return Ok(None);
                //     }
                //     Some(b',') if !self.first => {
                //         self.de.eat_char();
                //         tri!(self.de.parse_whitespace())
                //     }
                //     Some(b) => {
                //         if self.first {
                //             self.first = false;
                //             Some(b)
                //         } else {
                //             return Err(self.de.peek_error(ErrorCode::ExpectedObjectCommaOrEnd));
                //         }
                //     }
                //     None => {
                //         return Err(self.de.peek_error(ErrorCode::EofWhileParsingObject));
                //     }
                // };

                // match peek {
                //     Some(b'"') => seed.deserialize(MapKey { de: &mut *self.de }).map(Some),
                //     Some(b'}') => Err(self.de.peek_error(ErrorCode::TrailingComma)),
                //     Some(_) => Err(self.de.peek_error(ErrorCode::KeyMustBeAString)),
                //     None => Err(self.de.peek_error(ErrorCode::EofWhileParsingValue)),
                // }
                if self.pos == self.len {
                    return Ok(None)
                }
                
                dbg!(self.cursor.node().to_sexp());
                let mut patt_de = PatternDeserializer {
                    input: self.input,
                    cursor: self.cursor.clone(),
                };
                seed.deserialize(&mut patt_de).map(|x| Some(x))
            }

            fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
            where
                V: de::DeserializeSeed<'de>,
            {
                dbg!(self.cursor.node().to_sexp());
                let mut patt_de = PatternDeserializer {
                    input: self.input,
                    cursor: self.cursor.clone(),
                };
                let r = seed.deserialize(&mut patt_de)?;
                // self.cursor;
                self.pos += 1;
                Ok(r)
            }
        }

        // let mut patt_de = PatternDeserializer {
        //     input: self.input,
        //     cursor: self.cursor.clone(),
        // };
        // // seed.deserialize(&mut patt_de);
        // todo!();

        visitor.visit_map({
            let input = self.input;
            let cursor = self.cursor.clone();
            MapAccess {
                input,
                cursor,
                pos:0,
                len: fields.len(),
                pattern_pos,
            }
        })
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_identifier<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }
}

struct CaptureDeserializer<'de> {
    input: &'de [u8],
    node: Node<'de>,
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut CaptureDeserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_bool<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_i8<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_i16<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_i32<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_i64<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_u8<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_u16<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_u32<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_u64<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_f32<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_f64<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_char<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_str<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_string<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let capture_name = self
            .node
            .child_by_field_name("name")
            .ok_or(Error::CaptureWithoutNameField)?;
        dbg!(capture_name.to_sexp());
        assert_eq!("identifier", capture_name.kind());
        let capture_name = capture_name.utf8_text(self.input).map_err(|x| todo!())?;
        visitor.visit_str(capture_name)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_option<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_unit<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_seq<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_tuple<V>(
        self,
        len: usize,
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_map<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_identifier<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }
}
struct PatternDeserializer<'de> {
    input: &'de [u8],
    cursor: TreeCursor<'de>,
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut PatternDeserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_bool<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_i8<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_i16<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_i32<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_i64<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_u8<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_u16<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_u32<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_u64<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_f32<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_f64<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_char<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_str<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_string<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_bytes<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_option<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_unit<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_seq<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_tuple<V>(
        self,
        len: usize,
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_map<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_identifier<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        // self.deserialize_str(visitor)
        dbg!("aaa name");
        visitor.visit_str("name")
        // todo!()
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_bool(true) // TODO not sure
    }
}

mod error {
    use std::{fmt::Display, result};

    /// This type represents all possible errors that can occur when serializing or
    /// deserializing JSON data.
    #[derive(Debug)]
    pub enum Error {
        TrailingCharacters,
        FormatIsNotSelfDescribing,
        CaptureWithoutNameField,
        MISSING_PATTERN_FIELD,
    }

    /// Alias for a `Result` with the error type `serde_json::Error`.
    pub type Result<T> = result::Result<T, Error>;

    impl Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            todo!()
        }
    }

    impl std::error::Error for Error {}

    impl serde::de::Error for Error {
        fn custom<T>(msg: T) -> Self
        where
            T: std::fmt::Display,
        {
            panic!("{}",msg)
        }
    }
}
