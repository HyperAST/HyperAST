
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
        write!(f,"{}:({},{})", self.file,self.offset,self.len)
    }
}

#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub struct Range {
    pub(crate) offset: usize,
    pub(crate) len: usize,
}

impl Display for Range {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"({},{})",self.offset,self.len)
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

#[derive(Serialize, Deserialize, Debug)]
pub struct Relation {
    pub(crate) decl: Position,
    pub(crate) refs: Vec<Position>,
}

pub type Relations = Vec<Relation>;

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