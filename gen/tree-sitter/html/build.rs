use std::path::PathBuf;

fn main() {
    let dir: PathBuf = ["tree-sitter-html", "src"].iter().collect();

    cc::Build::new()
        .compiler("clang")
        .cpp(true)
        .cpp_link_stdlib("stdc++")
        // .flag("-lstdc++")
        .include(&dir)
        .file(dir.join("parser.c"))
        .file(dir.join("scanner.cc"))
        .compile("tree-sitter-html");
}
