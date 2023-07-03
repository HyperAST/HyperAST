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
    sig: String,
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
    let package_ref = scoped!(root, "spoon");
    let i = scoped!(mm, "spoon");
    let i = scoped!(scoped!(mm, "spoon"), "Launcher");
    let i = scoped!(package_ref, "SpoonAPI");
    let i = scoped_type!(package_ref, "SpoonAPI");
    let i = scoped_type!(scoped!(scoped!(root, "java"), "lang"), "Object");
    let mut sp_store = StructuralPositionStore::new(id);
    let mut x = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let x = sp_store.type_scout(&mut x, unsafe {
        hyper_ast_gen_ts_java::types::TIdN::from_ref_id(&id)
    });
    let r = usage::RefsFinder::new(stores, &mut ana, &mut sp_store).find_all(package_ref, i, x);
    dbg!(r.len());
    Some(r.len())
}
