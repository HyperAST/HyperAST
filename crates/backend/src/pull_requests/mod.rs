use std::str::from_utf8;

use axum::response::IntoResponse;
use http::header::{AUTHORIZATION, USER_AGENT};
use hyper_rustls::ConfigBuilderExt;
use hyper_util::{client::legacy::Client, rt::TokioExecutor};
use serde::{Deserialize, Serialize};

use super::*;
use graphql_client::GraphQLQuery;

#[allow(clippy::upper_case_acronyms)]
type URI = String;

#[allow(clippy::upper_case_acronyms)]
type GitObjectID = String;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/pull_requests/schema.graphql",
    query_path = "src/pull_requests/query.graphql",
    response_derives = "Debug,Serialize,PartialEq"
)]
struct RepoView;

pub struct RawPrData(String);

type Oid = String;

#[derive(Serialize)]
pub struct PrData {
    merge_commit: Option<Commit>,
    head_commit: Commit,
    title: String,
    number: i64,
}

#[derive(Serialize)]
struct Commit {
    id: Oid,
    user: String,
    name: String,
}

impl IntoResponse for PrData {
    fn into_response(self) -> axum::response::Response {
        serde_json::to_string(&self).unwrap().into_response()
    }
}

#[axum_macros::debug_handler]
pub(super) async fn pr_commits(
    axum::extract::Path(path): axum::extract::Path<commit::Param>,
    axum::extract::State(state): axum::extract::State<SharedState>,
) -> Result<PrData, String> {
    if let Some(x) = state.pr_cache.read().unwrap().get(&path) {
        let data = serde_json::from_str(&x.0).map_err(|e| e.to_string())?;
        return format_result(&data);
    };

    log::info!("Pr not in cache: {path:?}");

    let mut cursor = "".to_string();
    loop {
        let variables = repo_view::Variables {
            query: format!(
                "is:merged type:pr oid:{} repo:{}/{}",
                path.version, path.user, path.name
            ),
            after: cursor,
        };
        let data = query_github(variables).await?;
        let a = data.search.nodes.as_ref().unwrap()[0].as_ref().unwrap();
        match a {
            repo_view::RepoViewSearchNodes::PullRequest(pr) => {
                if pr
                    .merge_commit
                    .as_ref()
                    .unwrap()
                    .oid
                    .starts_with(&path.version)
                {
                    state
                        .pr_cache
                        .write()
                        .unwrap()
                        .insert(path, RawPrData(serde_json::to_string(&data).unwrap()));
                    return format_result(&data);
                } else if !data.search.page_info.has_next_page {
                    return Err("not a merged commit".to_string());
                } else if let Some(c) = &data.search.page_info.end_cursor {
                    cursor = c.to_string();
                } else {
                    panic!()
                }
            }
            _ => unreachable!(),
        }
    }
}

fn format_result(data: &repo_view::ResponseData) -> Result<PrData, String> {
    let a: &repo_view::RepoViewSearchNodes =
        data.search.nodes.as_ref().unwrap()[0].as_ref().unwrap();
    match a {
        repo_view::RepoViewSearchNodes::PullRequest(pr) => {
            let merge_commit = pr.merge_commit.as_ref().map(|c| Commit {
                id: c.oid.clone(),
                user: pr.repository.owner.login.clone(),
                name: pr.repository.name.clone(),
            });
            let head_commit = if let Some(repo) = &pr.head_repository {
                Commit {
                    id: pr.head_ref_oid.clone(),
                    user: repo.owner.login.clone(),
                    name: repo.name.clone(),
                }
            } else {
                Commit {
                    id: pr.head_ref_oid.clone(),
                    user: pr.repository.owner.login.clone(),
                    name: pr.repository.name.clone(),
                }
            };
            Ok(PrData {
                merge_commit,
                head_commit,
                title: pr.title.clone(),
                number: pr.number,
            })
        }
        _ => unreachable!(),
    }
}

async fn query_github(variables: repo_view::Variables) -> Result<repo_view::ResponseData, String> {
    let github_api_token =
        std::env::var("GITHUB_API_TOKEN").expect("Missing GITHUB_API_TOKEN env var");

    // Prepare the TLS client config
    let tls = rustls::ClientConfig::builder()
        .with_native_roots()
        .unwrap()
        .with_no_client_auth();
    // Prepare the HTTPS connector
    let https = hyper_rustls::HttpsConnectorBuilder::new()
        .with_tls_config(tls)
        .https_or_http()
        .enable_http1()
        .build();

    // Build the hyper client from the HTTPS connector.
    let client: Client<_, _> = Client::builder(TokioExecutor::new()).build(https);

    let body = RepoView::build_query(variables);
    dbg!(body.query);
    dbg!(&body.variables.query);
    let req = hyper::Request::builder()
        .header(USER_AGENT, "graphql-rust/0.14.0")
        .header(AUTHORIZATION, format!("Bearer {}", github_api_token))
        .method("POST")
        .uri("https://api.github.com/graphql")
        .body(serde_json::to_string(&body).unwrap())
        .unwrap();
    let resp = client.request(req).await.unwrap();
    let (parts, body) = resp.into_parts();
    dbg!(parts.status);
    use http_body_util::BodyExt;
    let body = body.collect().await.unwrap();
    let bytes = &body.to_bytes();
    log::info!("{}", from_utf8(bytes).unwrap());
    #[derive(Deserialize)]
    struct AAA {
        data: repo_view::ResponseData,
    }
    let b: AAA = serde_json::from_slice(bytes).unwrap();
    dbg!(&b.data);
    let c = b.data.search.nodes.as_ref().unwrap();
    let Some(d) = c.get(0) else {
        return Err("no Pr merging this commit".to_string());
    };
    dbg!(d.as_ref().unwrap());
    let data: repo_view::ResponseData = b.data;
    Ok(data)
}
