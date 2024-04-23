pub type ApiAddr = str;

#[derive(Hash, PartialEq, Eq, Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Repo {
    pub user: String,
    pub name: String,
}

impl Default for Repo {
    fn default() -> Self {
        Self {
            // user: "INRIA".to_string(),
            // name: "spoon".to_string(),
            user: "official-stockfish".to_string(),
            name: "Stockfish".to_string(),
        }
    }
}

pub type CommitId = String;

#[derive(Hash, PartialEq, Eq, Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct Commit {
    #[serde(flatten)]
    pub repo: Repo,
    #[serde(alias = "commit")]
    pub id: CommitId,
}

impl Default for Commit {
    fn default() -> Self {
        Self {
            repo: Default::default(),
            id: "7f2eb10e93879bc569c7ddf6fb51d6f812cc477c".into(),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Default)]
#[serde(default)]
pub struct ComputeConfigMulti {
    pub list: Vec<Commit>,
}

#[derive(serde::Deserialize, serde::Serialize, Default)]
#[serde(default)]
pub struct ComputeConfigDiff {
    pub repo: Repo,
    pub before: CommitId,
    pub after: CommitId,
}
