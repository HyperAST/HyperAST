use std::path::{Path, PathBuf};

use tree_sitter::{Language, Parser};



extern "C" {
    fn tree_sitter_html() -> Language;
}

#[test]
fn html_tree_sitter_simple() {
    
    let mut parser = Parser::new();

    {
        let language = unsafe { tree_sitter_html() };
        parser.set_language(language).unwrap();
    }


    let text = {
        let source_code1 = "<html><body></body></html>";
        source_code1.as_bytes()
    };
    let tree = parser.parse(text, None).unwrap();
    println!("{}", tree.root_node().to_sexp());
}

#[test]
fn html_tree_sitter_on_pom() {
    
    let mut parser = Parser::new();

    {
        let language = unsafe { tree_sitter_html() };
        parser.set_language(language).unwrap();
    }

    let path: PathBuf = Path::new("../../../benchmark/pom.xml").to_path_buf();
    
    let text = std::fs::read(path).unwrap();
    let tree = parser.parse(&text, None).unwrap();
    println!("{:#?}", tree.root_node().to_sexp());
}

#[test]
fn hyperAST_on_pom() {
    
    let mut parser = Parser::new();

    {
        let language = unsafe { tree_sitter_html() };
        parser.set_language(language).unwrap();
    }

    let path: PathBuf = Path::new("../../../benchmark/pom.xml").to_path_buf();
    
    let text = std::fs::read(path).unwrap();
    let tree = parser.parse(&text, None).unwrap();
    println!("{:#?}", tree.root_node().to_sexp());
}