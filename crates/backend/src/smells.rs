use std::ops::Range;

use axum::Json;
use code2query::QueryLattice;
use hashbrown::HashSet;
use hyperast::position::position_accessors::SolvedPosition;
use hyperast::{
    position::{
        position_accessors::{RootedPosition, WithPreOrderOffsets},
        TreePathMut,
    },
    types::Children,
};
use hyper_diff::actions::Actions;
use serde::{Deserialize, Serialize};
use tokio::time::Instant;

use crate::SharedState;

pub(crate) mod matching;

mod code2query;

mod diffing;

type Idx = u16;

#[derive(Deserialize, Clone)]
pub struct Param {
    user: String,
    name: String,
    commit: String,
    len: usize,
}

#[derive(Deserialize, Clone)]
pub struct Diffs {
    user: String,
    name: String,
    commit: String,
    len: usize,
}

#[derive(Deserialize, Clone)]
pub struct Examples {
    #[serde(default)]
    simple_matching: bool,
    #[serde(default)]
    prepro_matching: bool,
    /// the query configuring the query generation from examples
    /// eg. `(identifier) @label ["{" ";" "." "try" "(" ")" "}" "catch" "import"] @skip (block ["{" "}"] @show) (block) @imm`
    /// eg. `(identifier) (type_identifier)` same as `(identifier) @label (type_identifier) @label`
    meta_gen: String,
    /// the query configuring the query simplification/generalization
    /// eg. `(predicate (identifier) (#EQ? "EQ") (parameters (string) @label )) @pred`
    meta_simp: String,
    /// the list of examples driving the query generation
    examples: Vec<ExamplesValue>,
}

#[derive(Debug, Serialize, Clone)]
pub enum SmellsError {
    Error(String),
}

#[derive(Serialize)]
pub struct SearchResults {
    pub prepare_time: f64,
    pub search_time: f64,
    bad: Vec<SearchResult>,
    good: Vec<SearchResult>,
    // additional[examples.len() + i]
    additional: Vec<ExamplesValue>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct SearchResult<Q = String> {
    pub query: Q,
    // the corresponding examples
    pub examples: Vec<usize>,
    // stats
    pub matches: usize,
    pub additional: Vec<usize>,
}

#[derive(Serialize)]
pub struct ExamplesResults {
    pub prepare_time: f64,
    pub search_time: f64,
    examples: Vec<ExamplesValue>,
    moves: Vec<(CodeRange, CodeRange)>,
}

#[derive(Deserialize, Clone, Serialize)]
pub struct ExamplesValue {
    before: CodeRange,
    after: CodeRange,
    deletes: Vec<Range<usize>>,
    inserts: Vec<Range<usize>>,
    moves: Vec<(Range<usize>, Range<usize>)>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub(crate) struct CodeRange {
    user: String,
    name: String,
    commit: String,
    file: String,
    start: usize,
    end: usize,
    path: Vec<Idx>,
}

#[derive(PartialEq, Eq)]
enum QueryGenKind {
    Simple,
    Advanced,
    Advanced2,
}
const QUERY_GENERATOR: QueryGenKind = QueryGenKind::Advanced2;

pub(crate) fn smells(
    examples: Examples,
    state: SharedState,
    path: Param,
) -> Result<Json<SearchResults>, String> {
    let now = Instant::now();
    let Param {
        user,
        name,
        commit,
        len,
    } = path;
    let Examples {
        meta_gen,
        meta_simp,
        examples,
        simple_matching,
        prepro_matching,
    } = examples;
    let prepro_matching = if simple_matching {
        prepro_matching
    } else if prepro_matching {
        prepro_matching
    } else {
        true
    };

    let repo_spec = hyperast_vcs_git::git::Forge::Github.repo(user, name);
    let configs = state.clone();
    let repo_handle = state
        .repositories
        .write()
        .unwrap()
        .get_config(repo_spec)
        .ok_or_else(|| "missing config for repository".to_string())?;
    let mut repository = repo_handle.fetch();
    log::warn!("done cloning {}", repository.spec);
    let commits = state
        .repositories
        .write()
        .unwrap()
        .pre_process_with_limit(&mut repository, "", &commit, 4)
        .map_err(|e| e.to_string())?;
    log::warn!(
        "done construction of {commits:?} in {}",
        repository.spec.user()
    );
    let prepare_time = now.elapsed().as_secs_f64();
    let now = Instant::now();
    let src_oid = commits[0];
    let dst_oid = commits[1];
    use hyperast_vcs_git::processing::ConfiguredRepoTrait;
    let repo_handle = &repository;
    let repositories = state.repositories.read().unwrap();
    let commit_src = repositories
        .get_commit(repo_handle.config(), &src_oid)
        .unwrap();
    let src_tr = commit_src.ast_root;
    let commit_dst = repositories
        .get_commit(repo_handle.config(), &dst_oid)
        .unwrap();
    let dst_tr = commit_dst.ast_root;
    let with_spaces_stores: &hyperast::store::SimpleStores<hyperast_vcs_git::TStore> =
        &repositories.processor.main_stores;

    // NOTE temporary bypass, will be fixed when adding polyglote facilities
    // SAFETY for now TStores are identical enough to be transmuted
    // TODO use a proper wrapper
    // TODO alternatively rework the type store and node types entirely with some compile time links/macros
    let sss: &hyperast::store::SimpleStores<hyperast_gen_ts_java::types::TStore> =
        unsafe { std::mem::transmute(with_spaces_stores) };
    let meta_gen = hyperast_tsquery::Query::new(&meta_gen, hyperast_gen_ts_java::language())
        .map_err(|x| x.to_string())?;
    let meta_simp = hyperast_tsquery::Query::new(&meta_simp, hyperast_gen_ts_tsquery::language())
        .map_err(|x| x.to_string())?;

    let ex_map: std::collections::HashMap<_, Vec<_>> = examples
        .into_iter()
        .enumerate()
        .map(|(i, e)| {
            assert_eq!(&e.before.commit, &dst_oid.to_string());
            assert!(!e.before.path.is_empty());
            let (_, from) = hyperast::position::compute_position(
                dst_tr,
                &mut e.before.path.iter().copied(),
                with_spaces_stores,
            );
            (from, i)
        })
        .fold(Default::default(), |mut acc, x| {
            acc.entry(x.0).or_default().push(x.1);
            acc
        });
    let query_lattice =
        QueryLattice::with_examples(sss, ex_map.keys().copied(), &meta_gen, &meta_simp);
    let bad: Vec<_> = query_lattice
        .iter()
        .filter(|x| 5 < x.1.len() && x.1.len() * 2 < ex_map.len())
        .collect();
    dbg!(bad.len());
    let matches = if simple_matching {
        matching::matches_default(with_spaces_stores, dst_tr, bad.iter().map(|x| x.0.as_str()))?
    } else if prepro_matching {
        matching::matches_with_precomputeds(
            with_spaces_stores,
            dst_tr,
            bad.iter().map(|x| x.0.as_str()),
        )?
    } else {
        unreachable!()
        // TODO
        // let qqq = hyperast_tsquery::Query::big(
        //     &col.iter().map(|x| bad[x[0]].query.as_str()).collect::<Vec<_>>(),
        //     hyperast_gen_ts_java::language(),
        // )
        // .map_err(|e| e.to_string())?;
    };
    let mut bad: Vec<_> = matches
        .iter()
        .enumerate()
        .map(|(i, v)| SearchResult {
            query: bad[i].0.clone(),
            examples: bad[i]
                .1
                .iter()
                .flat_map(|x| ex_map.get(x).unwrap())
                .copied()
                .collect::<HashSet<_>>()
                .into_iter()
                .collect(),
            matches: *v,
            additional: vec![],
        })
        .collect();

    bad.sort_by(|a, b| {
        let cmp = b.examples.len().cmp(&a.examples.len());
        if cmp.is_eq() {
            return b.query.len().cmp(&a.query.len());
        }
        cmp
    });
    let search_time = now.elapsed().as_secs_f64();
    Ok(Json::from(SearchResults {
        prepare_time,
        search_time,
        bad,
        good: vec![],
        additional: vec![],
    }))
}

pub(crate) fn smells_ex_from_diffs(
    state: SharedState,
    path: Diffs,
) -> Result<Json<ExamplesResults>, String> {
    let now = Instant::now();
    let Diffs {
        user,
        name,
        commit,
        len,
    } = path;
    let repo_spec = hyperast_vcs_git::git::Forge::Github.repo(user, name);
    let configs = state.clone();
    let repo_handle = state
        .repositories
        .write()
        .unwrap()
        .get_config(repo_spec)
        .ok_or_else(|| "missing config for repository".to_string())?;
    let mut repository = repo_handle.fetch();
    log::warn!("done cloning {}", repository.spec);
    let commits = state
        .repositories
        .write()
        .unwrap()
        .pre_process_with_limit(&mut repository, "", &commit, 4)
        .map_err(|e| e.to_string())?;
    let prepare_time = now.elapsed().as_secs_f64();
    let now = Instant::now();
    log::warn!(
        "done construction of {commits:?} in {}",
        repository.spec.user()
    );
    let src_oid = commits[0];
    let dst_oid = commits[1];
    let diff = diffing::diff(state, &repository, dst_oid, src_oid).map_err(|e| e.to_string())?;
    dbg!(diff.moves.len());
    dbg!(diff.deletes.len());
    let focuses = diff.focuses;
    let inserts = &diff.inserts;
    let deletes = &diff.deletes;
    let moves = diff.moves;
    let examples = focuses
        .iter()
        .map(|(l, r)| {
            let after = globalize(&repository, src_oid, l.clone());
            let before = globalize(&repository, dst_oid, r.clone());
            let deletes = deletes
                .iter()
                .filter(|x| x.0.file() == l.0.file())
                .map(|x| x.0.range())
                .collect();
            let inserts = inserts
                .iter()
                .filter(|x| x.0.file() == r.0.file())
                .map(|x| x.0.range())
                .collect();
            let moves = moves
                .iter()
                .filter(|(y, x)| x.0.file() == l.0.file() && y.0.file() == r.0.file())
                .map(|(y, x)| (x.0.range(), y.0.range()))
                .collect();
            ExamplesValue {
                before,
                after,
                deletes,
                inserts,
                moves,
            }
        })
        .collect();
    let moves: Vec<_> = moves
        .into_iter()
        .map(|(l, r)| {
            (
                globalize(&repository, src_oid, l),
                globalize(&repository, dst_oid, r),
            )
        })
        .collect();
    let diff_time = now.elapsed().as_secs_f64();
    log::warn!(
        "done computing diff on {commits:?} in {}: found {} focuses, {} moves, {} inserts, and {} deletes",
        repository.spec.user(),
        focuses.len(),
        moves.len(),
        inserts.len(),
        deletes.len(),
    );
    Ok(Json::from(ExamplesResults {
        examples,
        moves,
        prepare_time,
        search_time: diff_time,
    }))
}

pub(crate) fn globalize(
    repository: &hyperast_vcs_git::processing::ConfiguredRepo2,
    oid: hyperast_vcs_git::git::Oid,
    p: Pos,
) -> CodeRange {
    CodeRange {
        user: repository.spec.user().to_string(),
        name: repository.spec.name().to_string(),
        commit: oid.to_string(),
        file: p.0.file().to_str().unwrap().to_owned(),
        start: p.0.range().start,
        end: p.0.range().end,
        path: p.1,
    }
}

pub(crate) struct Diff {
    // actions: Option<ActionsVec<SimpleAction<LabelIdentifier, CompressedTreePath<Idx>, NodeIdentifier>>>,
    focuses: Vec<(Pos, Pos)>,
    deletes: Vec<Pos>,
    inserts: Vec<Pos>,
    moves: Vec<(Pos, Pos)>,
}
type Pos = (
    hyperast::position::file_and_offset::Position<std::path::PathBuf, usize>,
    Vec<Idx>,
);
