use std::net::SocketAddr;

use clap::Parser;

#[derive(Parser)]
#[clap(version, about, long_about = None)]
pub struct Options {
    /// Increase verbosity, and can be used multiple times
    #[clap(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// send logs to rerun endpoint
    #[cfg(feature = "rerun")]
    #[clap(long)]
    pub re_log: bool, // TODO add a variant to select address of endpoint

    /// The address for the server
    #[clap(default_value_t = SocketAddr::from(([127,0,0,1], 8888)))]
    pub address: SocketAddr,

    /// config a specific repository (multiple uses)
    ///
    /// use the following syntax: <forge>/<user>/<name>:<config>
    /// example: github.com/INRIA/spoon:Java
    #[clap(short, long)]
    pub repository: Vec<RepoConfig>,
}

pub struct RepoConfig {
    pub repo: hyperast_vcs_git::git::Repo,
    pub config: hyperast_vcs_git::processing::RepoConfig,
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

pub fn parse() -> Options {
    let opts = Options::parse();

    let debug_level = match opts.verbose {
        0 => log::Level::Info,
        1 => log::Level::Debug,
        _ => log::Level::Trace,
    };

    configure_logging(debug_level);

    opts
}

fn default_log_config(debug_level: log::Level) {
    // TODO should just leverage the env variable... I was just playing with clap :/
    if debug_level == log::Level::Trace {
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
}

#[cfg(not(feature = "rerun"))]
fn configure_logging(debug_level: log::Level) {
    default_log_config(debug_level)
}

#[cfg(feature = "rerun")]
fn configure_logging(debug_level: log::Level) {
    rerun::external::re_log::setup_logging();
    let rec = rerun::RecordingStreamBuilder::new("HyperAST").connect();
    let rec = match rec {
        Ok(rec) => rec,
        Err(e) => {
            default_log_config(debug_level);
            log::warn!("{}", e);
            return;
        }
    };
    rec.log(
        "logs",
        &rerun::TextLog::new("this entry has loglevel TRACE")
            .with_level(rerun::TextLogLevel::TRACE),
    )
    .unwrap();

    just_trying_stuff_with_rerun(&rec);

    let logger = rerun::Logger::new(rec.clone()) // recording streams are ref-counted
        .with_path_prefix("logs/server")
        // You can also use the standard `RUST_LOG` environment variable!
        .with_filter(rerun::default_log_filter());
    rerun::external::re_log::add_boxed_logger(Box::new(logger)).unwrap();
    rerun::RecordingStream::set_global(rerun::StoreKind::Recording, Some(rec));

    let pos = rerun::Position2D::new(3.234, -1.223);
    rerun::external::re_log::error!(target: "app_events", position = ?pos, "New position");
    let rec = rerun::RecordingStream::global(rerun::StoreKind::Recording).unwrap();
    rec.log(
        "logs",
        &rerun::TextLog::new("this entry has loglevel DEBUG")
            .with_level(rerun::TextLogLevel::DEBUG),
    )
    .unwrap();
}

#[cfg(feature = "rerun")]
fn just_trying_stuff_with_rerun(rec: &rerun::RecordingStream) {
    rec.log(
        "text_document",
        &rerun::TextDocument::new("Hello, TextDocument!"),
    )
    .unwrap();
    rec.log(
"markdown",
&rerun::TextDocument::new(
    r#"
[Click here to see the raw text](recording://markdown.Text).

Basic formatting:

| **Feature**       | **Alternative** |
| ----------------- | --------------- |
| Plain             |                 |
| *italics*         | _italics_       |
| **bold**          | __bold__        |
| ~~strikethrough~~ |                 |
| `inline code`     |                 |

----------------------------------

# Support
- [x] [Commonmark](https://commonmark.org/help/) support
- [x] GitHub-style strikethrough, tables, and checkboxes
- Basic syntax highlighting for:
- [x] C and C++
- [x] Python
- [x] Rust
- [ ] Other languages

# Links
You can link to [an entity](recording://markdown),
a [specific instance of an entity](recording://markdown[#0]),
or a [specific component](recording://markdown.Text).

Of course you can also have [normal https links](https://github.com/rerun-io/rerun), e.g. <https://rerun.io>.

# Image
![A random image](https://picsum.photos/640/480)
"#.trim(),
)
.with_media_type(rerun::MediaType::markdown()),
    ).unwrap();
    let origins = vec![rerun::Position3D::ZERO; 100];
    let (vectors, colors): (Vec<_>, Vec<_>) = (0..100)
        .map(|i| {
            let angle = std::f32::consts::TAU * i as f32 * 0.01;
            let length = ((i + 1) as f32).log2();
            let c = (angle / std::f32::consts::TAU * 255.0).round() as u8;
            (
                rerun::Vector3D::from([(length * angle.sin()), 0.0, (length * angle.cos())]),
                rerun::Color::from_unmultiplied_rgba(255 - c, c, 128, 128),
            )
        })
        .unzip();
    rec.log(
        "arrows",
        &rerun::Arrows3D::from_vectors(vectors)
            .with_origins(origins)
            .with_colors(colors),
    )
    .unwrap();
}
