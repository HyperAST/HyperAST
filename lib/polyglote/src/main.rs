use std::collections::BTreeMap;
mod macr;

pub fn get_token_names(language: &Language, escape: bool) -> Vec<(String, bool, String)> {
    use std::collections::hash_map::{Entry, HashMap};
    let count = language.node_kind_count();
    let mut names = BTreeMap::default();
    let mut name_count = HashMap::new();
    for anon in &[false, true] {
        for i in 0..count {
            let anonymous = !language.node_kind_is_named(i as u16);
            if anonymous != *anon {
                continue;
            }
            let kind = language.node_kind_for_id(i as u16).unwrap();
            let name = sanitize_identifier(kind);
            let ts_name = sanitize_string(kind, escape);
            let name = camel_case(&name);
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
    }
    let mut names: Vec<_> = names.values().cloned().collect();
    names.push(("Error".to_string(), false, "ERROR".to_string()));

    names
}

pub fn sanitize_identifier(name: &str) -> String {
    if name == "ï»¿" {
        return "BOM".to_string();
    }
    if name == "_" {
        return "UNDERSCORE".to_string();
    }
    if name == "self" {
        return "Zelf".to_string();
    }
    if name == "Self" {
        return "SELF".to_string();
    }

    let mut result = String::with_capacity(name.len());
    for c in name.chars() {
        if ('a'..='z').contains(&c)
            || ('A'..='Z').contains(&c)
            || ('0'..='9').contains(&c)
            || c == '_'
        {
            result.push(c);
        } else {
            let replacement = match c {
                '~' => "TILDE",
                '`' => "BQUOTE",
                '!' => "BANG",
                '@' => "AT",
                '#' => "HASH",
                '$' => "DOLLAR",
                '%' => "PERCENT",
                '^' => "CARET",
                '&' => "AMP",
                '*' => "STAR",
                '(' => "LPAREN",
                ')' => "RPAREN",
                '-' => "DASH",
                '+' => "PLUS",
                '=' => "EQ",
                '{' => "LBRACE",
                '}' => "RBRACE",
                '[' => "LBRACK",
                ']' => "RBRACK",
                '\\' => "BSLASH",
                '|' => "PIPE",
                ':' => "COLON",
                ';' => "SEMI",
                '"' => "DQUOTE",
                '\'' => "SQUOTE",
                '<' => "LT",
                '>' => "GT",
                ',' => "COMMA",
                '.' => "DOT",
                '?' => "QMARK",
                '/' => "SLASH",
                '\n' => "LF",
                '\r' => "CR",
                '\t' => "TAB",
                _ => continue,
            };
            if !result.is_empty() && !result.ends_with('_') {
                result.push('_');
            }
            result += replacement;
        }
    }
    result
}

pub fn sanitize_string(name: &str, escape: bool) -> String {
    let mut result = String::with_capacity(name.len());
    if escape {
        for c in name.chars() {
            match c {
                '\"' => result += "\\\\\\\"",
                '\\' => result += "\\\\\\\\",
                '\t' => result += "\\\\t",
                '\n' => result += "\\\\n",
                '\r' => result += "\\\\r",
                _ => result.push(c),
            }
        }
    } else {
        for c in name.chars() {
            match c {
                '\"' => result += "\\\"",
                '\\' => result += "\\\\",
                '\t' => result += "\\t",
                '\n' => result += "\\n",
                '\r' => result += "\\r",
                _ => result.push(c),
            }
        }
    }
    result
}

use macr::Lang;
use polyglote::{camel_case, preprocess_aux};
// use macroquad::miniquad::conf::{LinuxBackend, Platform};
use tree_sitter::Language;

#[test]
fn generate_typescript_type_enum() -> std::io::Result<()> {
    // let lang = Lang::Typescript;
    // let tags = tree_sitter_typescript::TAGGING_QUERY;
    // let hi = tree_sitter_typescript::HIGHLIGHT_QUERY;
    // let n_types = tree_sitter_typescript::TSX_NODE_TYPES;
    // let types = preprocess_aux(n_types, lang, Some(tags), Some(hi))?;
    // generate_types::ggg(&types);
    Ok(())
}

fn main() -> std::io::Result<()> {
    let mut args = std::env::args();
    args.next().unwrap();
    let input_lang_name = args.next();
    for lang in Lang::into_enum_iter() {
        if let Some(name) = &input_lang_name {
            if lang.as_ref() != name {
                continue;
            }
        }
        let types = preprocess_aux(&lang)?;
        println!("{}", types);
        types.pp_fields();
    }
    Ok(())
}

mod ts_metadata;

// mod deserialize_query;
