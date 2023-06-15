use hyper_ast::{store::labels::LabelStore, types::LabelStore as _};

use super::{
    element::{IdentifierFormat, LabelPtr, RefsEnum},
    solver::Solver,
};

#[test]
fn test() {
    let mut l_store = LabelStore::new();
    let mut intern_label = |x| LabelPtr::new(l_store.get_or_insert(x), IdentifierFormat::from(x));
    let mut s = Solver::default();

    let x = s.intern(RefsEnum::MaybeMissing);
    let x = s.intern_ref(RefsEnum::ScopedIdentifier(x, intern_label("E")));

    let on_demand = s.intern(RefsEnum::Root);
    let on_demand = s.intern(RefsEnum::ScopedIdentifier(on_demand, intern_label("java")));
    let on_demand = s.intern_ref(RefsEnum::ScopedIdentifier(on_demand, intern_label("lang")));

    let object = s.intern(RefsEnum::Root);
    let object = s.intern(RefsEnum::ScopedIdentifier(object, intern_label("java")));
    let object = s.intern(RefsEnum::ScopedIdentifier(object, intern_label("lang")));
    let object = s.intern_ref(RefsEnum::ScopedIdentifier(object, intern_label("Object")));

    let root = s.intern(RefsEnum::Root);

    let package = s.intern(RefsEnum::Root);
    let package = s.intern_ref(RefsEnum::ScopedIdentifier(package, intern_label("p")));


    let given = {
        let mask = s.intern(RefsEnum::Mask(root, vec![on_demand,package].into()));
        let mask = s.intern(RefsEnum::Mask(mask, vec![object].into()));
        
        s
        .try_solve_node_with(x, mask)
        .expect("maybemissing to be replaced")
    };
    let given2 = {
        let mask = s.intern(RefsEnum::Mask(root, vec![on_demand, package, object].into()));
        
        s
        .try_solve_node_with(x, mask)
        .expect("maybemissing to be replaced")
    };
    let expect = {
        let mask = s.intern(RefsEnum::Mask(root, vec![on_demand, package, object].into()));
        s.intern_ref(RefsEnum::ScopedIdentifier(mask, intern_label("E")))
    };

    assert!(expect==given,"given {:?} but expected {:?}",s.nodes.with(given),s.nodes.with(expect));
    assert!(expect==given2,"given {:?} but expected {:?}",s.nodes.with(given2),s.nodes.with(expect));
}
