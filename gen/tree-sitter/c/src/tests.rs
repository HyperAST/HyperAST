use hyperast::tree_gen::NoOpMore;
use tree_sitter::Parser;

use crate::{legion::tree_sitter_parse, types::TStore};

type CTreeGen<'store, 'cache, HAST, Acc> =
    crate::legion::CTreeGen<'store, 'cache, TStore, NoOpMore<HAST, Acc>, true>;
type SimpleStores = hyperast::store::SimpleStores<TStore>;

static EX: &str = r#"
void read_string(char *buf) {
    scanf("%s ", buf);
}"#;

#[test]
pub(crate) fn cpp_tree_sitter_simple() {
    let mut parser = Parser::new();

    {
        parser.set_language(&crate::language()).unwrap();
    }

    let text = { EX.as_bytes() };
    let tree = parser.parse(text, None).unwrap();
    println!("{}", tree.root_node().to_sexp());
}

#[test]
pub(crate) fn cpp_simple_test() {
    let text = { EX.as_bytes() };
    let tree = match tree_sitter_parse(text) {
        Ok(t) => t,
        Err(t) => t,
    };
    println!("{:#?}", tree.root_node().to_sexp());
    let mut stores = SimpleStores::default();
    let mut md_cache = Default::default();
    let mut tree_gen = CTreeGen::new(&mut stores, &mut md_cache);
    let x = tree_gen.generate_file(b"", text, tree.walk()).local;
    let x = x.compressed_node;
    println!("{}", hyperast::nodes::SyntaxSerializer::new(&stores, x));
    println!("{}", hyperast::nodes::SexpSerializer::new(&stores, x));
    println!("{}", hyperast::nodes::TextSerializer::new(&stores, x));
}
