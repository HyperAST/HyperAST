use std::path::{Path, PathBuf};

use hyper_ast::store::{SimpleStores, labels::LabelStore, TypeStore};
use tree_sitter::{Parser};

use crate::legion::XmlTreeGen;



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


#[test]
fn xml_issue_cdata() {
    let text = {
        let source_code1 = r#"<?xml version="1.0" encoding="UTF-8"?>
<project xmlns="http://maven.apache.org/POM/4.0.0" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:schemaLocation="http://maven.apache.org/POM/4.0.0 http://maven.apache.org/maven-v4_0_0.xsd">
<configuration>
    <bottom><![CDATA[<p align="center">Copyright &#169; {inceptionYear}-{currentYear} {organizationName}. All Rights Reserved.<br />
    .</p>]]></bottom>
</configuration>
</project>"#;
        source_code1.as_bytes()
    };
    let tree = match XmlTreeGen::tree_sitter_parse(text) {Ok(t)=>t,Err(t)=>t};
    println!("{:#?}", tree.root_node().to_sexp());
    let mut stores = SimpleStores {
        label_store: LabelStore::new(),
        type_store: TypeStore {},
        node_store: hyper_ast::store::nodes::legion::NodeStore::new(),
    };
    let mut tree_gen = XmlTreeGen {
        line_break: "\n".as_bytes().to_vec(),
        stores: &mut stores,
    };
    let x = tree_gen.generate_file(b"", text, tree.walk()).local;
    // println!("{}", tree.root_node().to_sexp());
}