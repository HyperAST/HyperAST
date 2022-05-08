use std::path::{Path, PathBuf};

use tree_sitter::{Parser};

use crate::xml_tree_gen::XmlTreeGen;



#[test]
fn xml_tree_sitter_simple() {
    
    let mut parser = Parser::new();

    {
        parser.set_language(tree_sitter_xml::language()).unwrap();
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
fn xml_tree_sitter_simple2() {
    let text = {
        let source_code1 = "<?xml version=\"1.0\"?><!-- q -->
<project>

    <require.bzip>false</require.bzip>
    <zstd.prefix></zstd.prefix>
    <zstd.lib></zstd.lib>
    <zstd.include></zstd.include>
    <require.zstd>false</require.zstd>
    <openssl.prefix></openssl.prefix>
    <openssl.lib></openssl.lib>
    <openssl.include></openssl.include>
    <require.isal>false</require.isal>
    <isal.prefix></isal.prefix>
    <isal.lib></isal.lib>
    <require.openssl>false</require.openssl>
    <runningWithNative>true</runningWithNative>
    <bundle.openssl.in.bin>false</bundle.openssl.in.bin>
    <extra.libhadoop.rpath></extra.libhadoop.rpath>
</project>
          ";
        source_code1.as_bytes()
    };
    let tree = match XmlTreeGen::tree_sitter_parse(text) {Ok(t)=>t,Err(t)=>t};
    println!("{}", tree.root_node().to_sexp());
}

#[test]
fn xml_tree_sitter_on_pom() {

    let path: PathBuf = Path::new("../../../benchmark/pom.xml").to_path_buf();
    
    let text = std::fs::read(path).unwrap();
    let tree = match XmlTreeGen::tree_sitter_parse(&text) {Ok(t)=>t,Err(t)=>t};
    println!("{:#?}", tree.root_node().to_sexp());
}

#[test]
fn hyperAST_on_pom() {

    let path: PathBuf = Path::new("../../../benchmark/pom.xml").to_path_buf();
    
    let text = std::fs::read(path).unwrap();
    let tree = match XmlTreeGen::tree_sitter_parse(&text) {Ok(t)=>t,Err(t)=>t};
    println!("{:#?}", tree.root_node().to_sexp());
}

