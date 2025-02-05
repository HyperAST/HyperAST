use client::{AppState, track::*};
use hyperast_vcs_git::git::Forge;
use hyperast_vcs_git::processing::RepoConfig;

#[ignore] // ignore (from normal cargo test) for now, later make a feature
#[test]
// slow test, more of an integration test, benefits from being run in release mode
fn test_track_at_file_pos() -> Result<(), Box<dyn std::error::Error>> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("client=debug")
        .try_init()
        .unwrap();
    let state: std::sync::Arc<AppState> = AppState::default().into();
    state
        .repositories
        .write()
        .unwrap()
        .register_config(Forge::Github.repo("INRIA", "spoon"), RepoConfig::JavaMaven);
    let path = TrackingParam {
        user: "INRIA".to_string(),
        name: "spoon".to_string(),
        commit: "5f250ead2df52d7fe26a3ed2bdd7a38355f764b1".to_string(),
        file: "src/main/java/spoon/SpoonModelBuilder.java".to_string(),
    };
    let mut flags = Flags::default();
    flags.upd = true;
    let query = TrackingQuery {
        start: Some(10),
        end: Some(200),
        before: Some("8cafc796a3afdda4d52e90f3d17f12c09735be02".to_string()),
        flags,
    };
    match track_code(state, path, query) {
        Ok(x) => {
            let s = serde_json::to_string_pretty(&x);
            eprintln!("{}", s.unwrap());
        }
        Err(x) => {
            dbg!(x.message);
            panic!()
        }
    }
    Ok(())
}
