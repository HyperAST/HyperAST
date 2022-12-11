use std::path::{Path, PathBuf};

use tree_sitter::Parser;

#[test]
fn html_tree_sitter_simple() {
    let mut parser = Parser::new();

    {
        let language = tree_sitter_html::language();
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
fn xml_tree_sitter_simple() {
    
    let mut parser = Parser::new();

    {
        parser.set_language(tree_sitter_html::language()).unwrap();
    }


    let text = {
        let source_code1 = "<?xml version=\"1.0\"?><!-- q -->
        <project>
        <plugin>
        </plugin>
        <!-- This plugin's configuration is used to store Eclipse m2e settings only.
        It has no influence on the Maven build itself. -->
        <plugin>
        </plugin>
        </project>
          ";
        source_code1.as_bytes()
    };
    let tree = parser.parse(text, None).unwrap();
    println!("{}", tree.root_node().to_sexp());
}

#[test]
fn html_tree_sitter_on_pom() {
    let mut parser = Parser::new();

    {
        let language = tree_sitter_html::language();
        parser.set_language(language).unwrap();
    }

    let path: PathBuf = Path::new("../../../benchmark/pom.xml").to_path_buf();

    let text = std::fs::read(path).unwrap();
    let tree = parser.parse(&text, None).unwrap();
    println!("{:#?}", tree.root_node().to_sexp());
}

#[test]
fn hyper_ast_on_pom() {
    let mut parser = Parser::new();

    {
        let language = tree_sitter_html::language();
        parser.set_language(language).unwrap();
    }

    let path: PathBuf = Path::new("../../../benchmark/pom.xml").to_path_buf();

    let text = std::fs::read(path).unwrap();
    let tree = parser.parse(&text, None).unwrap();
    println!("{:#?}", tree.root_node().to_sexp());
}
