use std::net::SocketAddr;

use clap::Parser;

#[derive(Parser)]
#[clap(version, about, long_about = None)]
pub(super) struct Options {
    /// Increase verbosity, and can be used multiple times
    #[clap(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// The address for the server
    #[clap(default_value_t = SocketAddr::from(([127,0,0,1], 8080)))]
    pub address: SocketAddr,

    /// config a specific repository (multiple uses)
    /// 
    /// use the following syntax: <forge>/<user>/<name>:<config>
    /// example: github.com/INRIA/spoon:Java
    #[clap(short, long)]
    pub repository: Vec<RepoConfig>,
}

pub(super) struct RepoConfig {
    pub(super) repo: hyper_ast_cvs_git::git::Repo,
    pub(super) config: hyper_ast_cvs_git::processing::RepoConfig,
}

impl std::str::FromStr for RepoConfig {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (repo, config) = s.split_once(":").ok_or("")?;
        let repo = repo.parse()?;
        let config = config.parse()?;

        Ok(Self { repo, config })
    }
}

pub(super) fn parse() -> Options {
    let opts = Options::parse();

    let debug_level = match opts.verbose {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };
    // env_logger::Builder::from_env(Env::default().default_filter_or(debug_level)).init();
    if debug_level == "trace" {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    } else {
        use tracing_subscriber::layer::SubscriberExt;
        use tracing_subscriber::util::SubscriberInitExt;
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "client=debug,client::file=debug,tower_http=debug".into()),
            )
            .with(tracing_subscriber::fmt::layer())
            .init();
    }
    opts
}
