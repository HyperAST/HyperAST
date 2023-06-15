use std::fmt::Display;

use serde::{Deserialize, Serialize};
use serde_json::Result;
use termion::color;
// use serde_with::skip_serializing_none;

use crate::relations::{Position, Range, Relation};

#[derive(Serialize, Deserialize, Debug)]
pub struct Comparison {
    pub(crate) decl: Position,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub(crate) exact: Vec<Position>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub(crate) per_file: Vec<ComparedRanges>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub(crate) left: Vec<Position>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub(crate) right: Vec<Position>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub(crate) left_contained: Vec<Position>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub(crate) right_contained: Vec<Position>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ComparedRanges {
    pub(crate) file: String,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub(crate) left: Vec<Range>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub(crate) right: Vec<Range>,
}

impl Display for ComparedRanges {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.file,
        )?;
        if !self.left.is_empty() {
            write!(
                f,
                " left [{}{}{}]",
                color::Bg(color::Magenta),
                self.left
                    .iter()
                    .map(|x| x.to_string())
                    .intersperse("".to_string())
                    .collect::<String>(),
                color::Bg(color::Reset),
                )?
        }
        if !self.right.is_empty() {
            write!(
                f,
                " right [{}{}{}]",
                color::Bg(color::Blue),
                self.right
                    .iter()
                    .map(|x| x.to_string())
                    .intersperse("".to_string())
                    .collect::<String>(),
                color::Bg(color::Reset),
            )?
        }
        Ok(())
    }
}

impl From<(std::string::String, (Vec<Range>, Vec<Range>))> for ComparedRanges {
    fn from((file, (left, right)): (std::string::String, (Vec<Range>, Vec<Range>))) -> Self {
        Self { file, left, right }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Comparisons {
    // pub(crate) api_version: String,
    pub(crate) left_name: String,
    pub(crate) right_name: String,
    /// declarations that match exactly
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub(crate) exact: Vec<Comparison>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub(crate) left: Vec<Relation>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub(crate) right: Vec<Relation>,
}

pub fn typed_example() -> Result<()> {
    // Some JSON input data as a &str. Maybe this comes from the user.
    let data = r#"{
        "left_name": "a.json",
        "right_name": "b.json",
        "exact": [
            {
                "decl": {
                    "offset":22776,"len":485,
                    "path":"src/main/java/spoon/reflect/meta/impl/ModelRoleHandlers.java"
                },
                "exact": [
                    {
                        "offset":10416,"len":33,
                        "path":"src/main/java/spoon/reflect/meta/impl/ModelRoleHandlers.java"
                    }
                ],
                "left": [
                    {
                        "offset":10416,"len":33,
                        "path":"src/main/java/spoon/reflect/meta/impl/ModelRoleHandlers.java"
                    }
                ],
                "left_contained": [
                    {
                        "offset":10416,"len":33,
                        "path":"src/main/java/spoon/reflect/meta/impl/ModelRoleHandlers.java"
                    }

                ]
            }
        ],
        "left": []
    }"#;
    let p: Comparisons = serde_json::from_str(data)?;

    println!("{:?}", p);

    Ok(())
}

#[test]
fn all() {
    typed_example().unwrap();
}
