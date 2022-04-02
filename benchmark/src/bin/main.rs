use std::{
    env,
    fs::File,
    io::{self, BufWriter, Write},
    path::PathBuf,
    str::FromStr,
};

use rusted_gumtree_cvs_git::{
    allrefs::write_referencial_relations, git::fetch_github_repository,
    preprocessed::PreProcessedRepository,
};
use rusted_gumtree_gen_ts_java::utils::memusage_linux;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);
    let repo_name = args
        .get(1)
        .expect("give an argument like openjdk/jdk or INRIA/spoon"); //"openjdk/jdk";//"INRIA/spoon";
    let before = args.get(2).map_or("", |x| x);
    let after = args.get(3).map_or("", |x| x);
    let dir_path = args.get(4).map_or("", |x| x);
    let out = args.get(5).and_then(|x| {
        if x.is_empty() {
            None
        } else {
            PathBuf::from_str(x).ok()
        }
    });

    let mut preprocessed = PreProcessedRepository::new(&repo_name);
    preprocessed.pre_process(
        &mut fetch_github_repository(&repo_name),
        before,
        after,
        dir_path,
    );
    let mu = memusage_linux();
    println!("total memory used {}", mu);
    preprocessed.purge_caches();
    println!("cache size: {}", mu - memusage_linux());
    log::info!("search refs");

    let repository = fetch_github_repository(preprocessed.name());
    // node identifier at after commit
    let root = preprocessed
        .commits
        .get(
            &repository
                .find_reference(after)
                .unwrap()
                .peel_to_commit()
                .unwrap()
                .id(),
        )
        .unwrap()
        .ast_root;

    if let Some(out) = out {
        let mut out = BufWriter::new(File::create(out).unwrap());
        write_referencial_relations(&preprocessed, root, &mut out);
        out.flush().unwrap();
    } else {
        let mut out = io::stdout();
        write_referencial_relations(&preprocessed, root, &mut out);
        out.flush().unwrap();
    }
    log::info!("done searching refs");

    let mu = memusage_linux();
    drop(preprocessed);
    println!("hyperAST size: {}", mu - memusage_linux());
}


#[test]
fn all() {
    use std::fs::read_to_string;
    use std::path::Path;
    use std::path::PathBuf;

    use pommes::Project;

    let path: PathBuf = Path::new("pom.xml").to_path_buf();
    println!("path: {}", &path.display());

    let contents = read_to_string(path).unwrap();
    let _parsed: Project = serde_xml_rs::from_str(&contents).unwrap();

    println!("{:#?}", _parsed);
}