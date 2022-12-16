use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    time::Instant,
};

use hyper_ast::{nodes::TreeJsonSerializer, types};

use crate::tempfile;

pub fn subprocess<'a, IdN, NS, LS>(
    node_store: &'a NS,
    label_store: &'a LS,
    src_root: IdN,
    dst_root: IdN,
    algorithm: &str,
    out_format: &str,
) -> PathBuf
where
    IdN: Clone,
    NS: 'a + types::NodeStore<IdN>,
    <NS as types::NodeStore<IdN>>::R<'a>:
        types::Tree<TreeId = IdN, Type = types::Type, Label = LS::I>,
    LS: types::LabelStore<str>,
{
    let (src, mut src_f) = tempfile().unwrap();
    dbg!(&src);
    src_f
        .write_all(
            TreeJsonSerializer::<_, _, _, true>::new(node_store, label_store, src_root.clone())
                .to_string()
                .as_bytes(),
        )
        .unwrap();
    let (dst, mut dst_f) = tempfile().unwrap();
    dbg!(&dst);
    dst_f
        .write_all(
            TreeJsonSerializer::<_, _, _, true>::new(node_store, label_store, dst_root.clone())
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
