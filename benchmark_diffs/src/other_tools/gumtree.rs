use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    time::Instant,
};

use hyperast::{nodes::JsonSerializer2 as JsonSerializer, types};

use crate::tempfile;

pub fn subprocess<'a, HAST>(
    stores: HAST,
    src_root: HAST::IdN,
    dst_root: HAST::IdN,
    mapping_algo: &str,
    diff_algo: &str,
    timeout: u64,
    out_format: &str,
) -> Option<PathBuf>
where
    HAST: types::HyperAST + Copy,
    // HAST: types::LabelStore<str>,
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
    // HAST: types::NodeStore<IdN>,
    // HAST: types::LabelStore<str>,
    // HAST: types::TypeStore<HAST::R<'a>>,
    // HAST::R<'a>: types::Labeled<Label = HAST::I> + types::WithChildren<TreeId = IdN>,
{
    let (src, mut src_f) = tempfile().unwrap();
    dbg!(&src);
    src_f
        .write_all(
            JsonSerializer::<_, HAST, true>::new(stores, src_root)
                .to_string()
                .as_bytes(),
        )
        .unwrap();
    let (dst, mut dst_f) = tempfile().unwrap();
    dbg!(&dst);
    dst_f
        .write_all(
            JsonSerializer::<_, _, true>::new(stores, dst_root)
                .to_string()
                .as_bytes(),
        )
        .unwrap();
    dbg!("start debugging");
    let (gt_out, _) = tempfile().unwrap();
    dbg!(&gt_out);
    let now = Instant::now();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    dbg!(root.join("gt_script.sh").to_str().unwrap());
    let mut child = std::process::Command::new("/usr/bin/bash")
        .arg(root.join("gt_script.sh").to_str().unwrap())
        .arg(&src)
        .arg(&dst)
        .arg(mapping_algo)
        .arg(&out_format)
        .arg(diff_algo)
        .arg(&gt_out)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .spawn()
        .expect("failed to spawn gumtree process");
    // .output()
    // .expect("failed to execute process");
    let wait = 1;
    let status;
    if timeout == 0 {
        match child.wait() {
            Ok(s) => {
                status = Some(s);
            }
            Err(e) => {
                println!("Error waiting: {}", e);
                status = None
            }
        }
    } else {
        let mut timeout = timeout;
        let waitd = std::time::Duration::from_secs(wait);
        loop {
            std::thread::sleep(waitd);
            match child.try_wait() {
                Ok(Some(s)) => {
                    status = Some(s);
                    break;
                }
                Ok(None) => (),
                Err(e) => println!("Error waiting: {}", e),
            }
            if timeout == 0 {
                std::io::stderr().flush().unwrap();
                std::io::stdout().flush().unwrap();
                child.kill().unwrap();
                status = None;
                break;
            }
            timeout = timeout - wait;
        }
    }
    let gt_processing_time = now.elapsed().as_secs_f64();
    dbg!(&gt_processing_time);
    if let Some(status) = status {
        fs::remove_file(&src).unwrap();
        fs::remove_file(&dst).unwrap();
        if !status.success() {
            eprintln!("gumtree process terminated with exit code {}", status);
            None
        } else {
            Some(gt_out)
        }
    } else {
        fs::remove_file(&src).unwrap();
        fs::remove_file(&dst).unwrap();
        eprintln!("gumtree process timedout");
        None
    }
}
