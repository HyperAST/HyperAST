use std::{
    io::Write,
    path::{Path, PathBuf},
    time::Instant, fs,
};

use hyper_ast::store::{defaults::NodeIdentifier, SimpleStores};
use hyper_ast_gen_ts_java::legion_with_refs::TreeJsonSerializer;

use crate::tempfile;

pub fn subprocess(
    stores: &SimpleStores,
    src_root: NodeIdentifier,
    dst_root: NodeIdentifier,
    algorithm: &str,
    out_format: &str,
) -> PathBuf {
    let (src, mut src_f) = tempfile().unwrap();
    dbg!(&src);
    src_f
        .write_all(
            TreeJsonSerializer::<true>::new(&stores.node_store, &stores.label_store, src_root.clone())
                .to_string()
                .as_bytes(),
        )
        .unwrap();
    let (dst, mut dst_f) = tempfile().unwrap();
    dbg!(&dst);
    dst_f
        .write_all(
            TreeJsonSerializer::<true>::new(&stores.node_store, &stores.label_store, dst_root.clone())
                .to_string()
                .as_bytes(),
        )
        .unwrap();
    dbg!("start debugging");
    let (gt_out, _) = tempfile().unwrap();
    dbg!(&gt_out);
    let now = Instant::now();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    std::process::Command::new("/usr/bin/bash")
        .arg(root.join("gt_script.sh").to_str().unwrap())
        .arg(&src)
        .arg(&dst)
        .arg(algorithm)
        .arg(&out_format)
        .arg(&gt_out)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .output()
        .expect("failed to execute process");
    let gt_processing_time = now.elapsed().as_secs_f64();
    dbg!(&gt_processing_time);
    fs::remove_file(&src).unwrap();
    fs::remove_file(&dst).unwrap();
    gt_out
}