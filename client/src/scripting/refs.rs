use hyper_ast::position::TreePathMut;
use hyper_ast::types::LabelStore;
use hyper_ast::{
    position::{Scout, StructuralPosition, StructuralPositionStore},
    store::{defaults::NodeIdentifier, SimpleStores},
    types::{HyperAST, NodeId},
};
use hyper_ast_cvs_git::TStore;
use hyper_ast_gen_ts_java::impact::element::{IdentifierFormat, LabelPtr, RefsEnum};
use hyper_ast_gen_ts_java::impact::{partial_analysis::PartialAnalysis, usage};

pub fn find_refs<'a>(
    stores: &'a SimpleStores<TStore>,
    id: NodeIdentifier,
    package_ref: &str,
    sig: &str,
) -> Option<usize> {
    let mut ana = PartialAnalysis::default(); //&mut commits[0].meta_data.0;

    macro_rules! scoped {
        ( $o:expr, $i:expr ) => {{
            let o = $o;
            let i = $i;
            let f = IdentifierFormat::from(i);
            let i = stores.label_store().get(i).unwrap();
            let i = LabelPtr::new(i, f);
            ana.solver.intern(RefsEnum::ScopedIdentifier(o, i))
        }};
    }
    macro_rules! scoped_type {
        ( $o:expr, $i:expr ) => {{
            let o = $o;
            let i = $i;
            let f = IdentifierFormat::from(i);
            let i = stores.label_store.get(i).unwrap();
            let i = LabelPtr::new(i, f);
            ana.solver.intern_ref(RefsEnum::TypeIdentifier(o, i))
        }};
    }
    let root = ana.solver.intern(RefsEnum::Root);
    let mm = ana.solver.intern(RefsEnum::MaybeMissing);
    // let package_ref = scoped!(root, "spoon");
    let package_ref = {
        let mut sig = package_ref;
        dbg!(sig);
        let mut i = if sig.starts_with("/") {
            sig = &sig[1..];
            root
        } else {
            mm
        };
        let mut sig = sig.split("#");
        if let Some(sig) = sig.next() {
            for x in sig.split(".") {
                dbg!(x);
                i = scoped!(i, x);
            }
        }
        for x in sig {
            dbg!(x);
            i = scoped_type!(i, x);
        }
        i
    };
    // // let i = scoped!(mm, "spoon");
    // // let i = scoped!(package_ref, "Launcher");
    // // let i = scoped!(package_ref, "SpoonAPI");
    // let i = scoped_type!(package_ref, "SpoonAPI");
    // // let i = scoped_type!(scoped!(scoped!(root, "java"), "lang"), "Object");
    // let i = scoped_type!(scoped!(scoped!(scoped!(root, "spoon"), "compiler"), "builder"), "JDTBuilder");
    // let i = scoped_type!(scoped!(scoped!(scoped!(mm, "spoon"), "compiler"), "builder"), "JDTBuilder");
    // let i = scoped_type!(scoped!(mm, "builder"), "JDTBuilder");
    let mut sig = sig;
    dbg!(sig);
    let mut i = if sig.starts_with("/") {
        sig = &sig[1..];
        root
    } else {
        mm
    };
    let mut sig = sig.split("#");
    if let Some(sig) = sig.next() {
        for x in sig.split(".") {
            dbg!(x);
            i = scoped!(i, x);
        }
    }
    for x in sig {
        dbg!(x);
        i = scoped_type!(i, x);
    }
    // let i = root;
    // let mut sig = sig.split(".").peekable();
    // loop {
    //     let Some(x) = sig.next() else {
    //         return None;
    //     };
    //     if sig.peek().is_none() {
    //         i = scoped_type!(i, x);
    //         break;
    //     }
    //     i = scoped!(i, x);
    // }
    // let i = scoped_type!(mm, "JDTBuilder");
    let mut sp_store = StructuralPositionStore::new(id);
    let mut x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let x = sp_store.type_scout(&mut x, unsafe {
        hyper_ast_gen_ts_java::types::TIdN::from_ref_id(&id)
    });
    let r = usage::RefsFinder::new(stores, &mut ana, &mut sp_store).find_all(package_ref, i, x);
    dbg!(r.len());
    Some(r.len())
}

#[derive(Clone)]
pub(super) struct QPath(pub(super) hyper_ast::position::StructuralPosition<NodeIdentifier, u16>);
impl QPath {
    pub(super) fn new(node: NodeIdentifier) -> Self {
        Self(hyper_ast::position::StructuralPosition::new(node))
    }
    pub(super) fn goto(&mut self, node: NodeIdentifier, i: u16) {
        self.0.goto(node, i)
    }
}
#[derive(Clone)]
pub(super) struct Pos(hyper_ast::position::Position);

impl From<hyper_ast::position::Position> for Pos {
    fn from(value: hyper_ast::position::Position) -> Self {
        Self(value)
    }
} 

// impl QPath {
//     fn convert<HAST>(self, stores: HAST) -> Pos {

//         let position_converter = &hyper_ast::position::PositionConverter::new(&it)
//         .with_stores(stores);
//         let p = position_converter.compute_pos_post_order::<_,Position,_>();
//         Pos(p)
//     }
// }