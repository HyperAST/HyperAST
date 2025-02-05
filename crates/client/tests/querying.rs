use client::AppState;

#[ignore] // ignore (from normal cargo test) for now, later make a feature
#[test]
// slow test, more of an integration test, try using release
fn test_querying() -> Result<(), Box<dyn std::error::Error>> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("client=debug")
        .try_init()
        .unwrap();

    let repo_spec = hyperast_vcs_git::git::Forge::Github.repo("graphhopper", "graphhopper");
    let config = hyperast_vcs_git::processing::RepoConfig::JavaMaven;
    let commit = "f5f2b7765e6b392c5e8c7855986153af82cc1abe";
    let query = r#"(try_statement
    (block
        (expression_statement 
            (method_invocation
                (identifier) (#EQ? "fail")
            )
        )
    )
    (catch_clause)
) @root
    "#;
    let language = "Java";
    compare_querying_with_and_without_skipping(repo_spec, config, commit, language, query)
}

/// use this in a test if you suspect a querying discrepancy on a commit due to the subtree skipping feature,
/// it might help you find where the query verdicts where not bubbled up.
fn compare_querying_with_and_without_skipping(
    repo_spec: hyperast_vcs_git::git::Repo,
    config: hyperast_vcs_git::processing::RepoConfig,
    commit: &str,
    language: &str,
    query: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let state = AppState::default();
    state
        .repositories
        .write()
        .unwrap()
        .register_config(repo_spec.clone(), config);
    let repo = state
        .repositories
        .read()
        .unwrap()
        .get_config(repo_spec)
        .ok_or_else(|| "missing config for repository".to_string())?;
    let mut repository = repo.fetch();
    log::debug!("done cloning {}", repository.spec);
    let commits = state.repositories.write().unwrap().pre_process_with_limit(
        &mut repository,
        "",
        &commit,
        1,
    )?;
    let repositories = state.repositories.read().unwrap();
    let commit = repositories
        .get_commit(&repository.config, &commits[0])
        .unwrap();
    let language: tree_sitter::Language = hyperast_vcs_git::resolve_language(&language).unwrap();
    let query_incr = hyperast_tsquery::Query::with_precomputed(
        &query,
        language.clone(),
        hyperast_vcs_git::java_processor::sub_queries(),
    )
    .map(|x| x.1)
    .unwrap();
    let query = hyperast_tsquery::Query::new(&query, language).unwrap();
    let code = commit.ast_root;
    let stores = &repositories.processor.main_stores;
    let mut qcursor_incr = {
        let pos = hyperast::position::StructuralPosition::new(code);
        let cursor = hyperast_tsquery::hyperast::TreeCursor::new(stores, pos);
        query_incr.matches(cursor)
    }
    .into_iter();
    let mut qcursor = {
        let pos = hyperast::position::StructuralPosition::new(code);
        let cursor = hyperast_tsquery::hyperast::TreeCursor::new(stores, pos);
        query.matches(cursor)
    }
    .into_iter();
    let root_incr = query_incr.capture_index_for_name("root").unwrap();
    let root = query.capture_index_for_name("root").unwrap();
    loop {
        let m = qcursor.next();
        if m.is_none() {
            let m_incr = qcursor_incr.next();
            assert!(m_incr.is_none());
            return Ok(());
        }
        let m = &m
            .as_ref()
            .unwrap()
            .nodes_for_capture_index(root)
            .next()
            .unwrap()
            .pos;
        log::debug!("m: {:?}", m);
        log::debug!("m: {:?}", m.make_file_line_range(stores));
        let m_incr = qcursor_incr.next();
        let m_incr = &m_incr
            .as_ref()
            .unwrap()
            .nodes_for_capture_index(root_incr)
            .next()
            .unwrap()
            .pos;
        log::debug!("m_incr: {:?}", m_incr);
        log::debug!("m_incr: {:?}", m_incr.make_file_line_range(stores));
        assert_eq!(
            m.make_file_line_range(stores),
            m_incr.make_file_line_range(stores)
        );
    }
}

// TODO test more of the high level API