use std::fmt::Display;

use serde::{Deserialize, Serialize};
use serde_json::Result;

#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub struct Position {
    pub(crate) file: String,
    pub(crate) offset: usize,
    pub(crate) len: usize,
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:({},{})", self.file, self.offset, self.len)
    }
}

#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub struct Range {
    pub(crate) offset: usize,
    pub(crate) len: usize,
}

impl Display for Range {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{})", self.offset, self.len)
    }
}

impl From<Position> for Range {
    fn from(p: Position) -> Self {
        Self {
            offset: p.offset,
            len: p.len,
        }
    }
}

impl Range {
    pub fn with(&self, file: String) -> Position {
        Position {
            file,
            offset: self.offset,
            len: self.len,
        }
    }
}

impl Into<hyper_ast::position::Position> for Position {
    fn into(self) -> hyper_ast::position::Position {
        hyper_ast::position::Position::new(self.file.into(), self.offset, self.len)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Relation {
    pub(crate) decl: Position,
    pub(crate) refs: Vec<Position>,
}

pub type Relations = Vec<Relation>;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct RelationsWithPerfs {
    pub(crate) relations: Option<Vec<PerModule<Vec<Relation>>>>,
    // pub(super) construction_time:u128,
    // pub(super) search_time:u128,
    // pub(super) construction_memory_fooprint:usize,
    // pub(super) with_search_memory_fooprint:usize,
    pub(super) construction_perfs: Perfs,
    pub(super) search_perfs: Option<Perfs>,
    pub(super) info: Option<Info>,
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct PerModule<T> {
    pub(super) module: String,
    pub(super) content: T,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub struct Perfs {
    /// time in nano seconds
    pub(super) time: u128,
    /// memory in bytes
    pub(super) memory: usize,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Info {
    pub(super) repo_name: String,
    pub(super) commit: String,
    pub(super) no: Option<usize>,
    pub(super) batch_id: Option<String>,
}

pub fn typed_example() -> Result<()> {
    // Some JSON input data as a &str. Maybe this comes from the user.
    let data = r#"[
            {
                "decl": {
                    "offset":22776,"len":485,
                    "path":"src/main/java/spoon/reflect/meta/impl/ModelRoleHandlers.java"
                },
                "refs": [
                    {
                        "offset":10416,"len":33,
                        "path":"src/main/java/spoon/reflect/meta/impl/ModelRoleHandlers.java"
                    }
                ]
            }
        ]"#;
    let p: Relations = serde_json::from_str(data)?;

    println!("{:?}", p);

    Ok(())
}

#[test]
fn all() {
    typed_example().unwrap();
}
