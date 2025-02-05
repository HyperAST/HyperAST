//! Metadata 2:
//! Similar to Metadata 1,
//! but uses lossy compression, i.e., need to cache the full data and not the compressed one.

use string_interner::DefaultHashBuilder;

use crate::types::LabelStore as _;

use super::primary::acc::*;
use super::*;

pub type Data<P, Md0, Md1, Md2> = (P, Md0, Md1, Md2);

pub fn fn_zip_data_init<P, Md0, Md1, Md2, X, Y>(
    primary: impl Fn(X, Y) -> P + 'static,
    f_md0: impl Fn() -> Md0 + 'static,
    f_md1: impl Fn() -> Md1 + 'static,
    f_md2: impl Fn() -> Md2 + 'static,
) -> impl Fn(X, Y) -> Data<P, Md0, Md1, Md2> {
    move |x, y| {
        let p = primary(x, y);
        let md0 = f_md0();
        let md1 = f_md1();
        let md2 = f_md2();
        (p, md0, md1, md2)
    }
}

pub fn fn_zip_data_acc<P, Md0, Md1, Md2, X, Y, Z, A>(
    primary: impl Fn(&mut P, &X) + 'static,
    f_md0: impl Fn(&mut Md0, &Y) + 'static,
    f_md1: impl Fn(&mut Md1, &Z) + 'static,
    f_md2: impl Fn(&mut Md2, &A) + 'static,
) -> impl Fn(&mut Data<P, Md0, Md1, Md2>, (X, Y, Z, A)) {
    move |a, x| {
        primary(&mut a.0, &x.0);
        f_md0(&mut a.1, &x.1);
        f_md1(&mut a.2, &x.2);
        f_md2(&mut a.3, &x.3);
    }
}

pub fn fn_zip_data_finish<P, Md0, Md1, Md2, X, _P, _Md0, _Md1, _Md2>(
    primary: impl Fn(P, X) -> _P + 'static,
    f_md0: impl Fn(Md0, &P) -> _Md0 + 'static,
    f_md1: impl Fn(Md1, &P) -> _Md1 + 'static,
    f_md2: impl Fn(Md2, &P) -> _Md2 + 'static,
) -> impl Fn(Data<P, Md0, Md1, Md2>, X) -> Data<_P, _Md0, _Md1, _Md2> {
    move |a, x| {
        let md0 = f_md0(a.1, &a.0);
        let md1 = f_md1(a.2, &a.0);
        let md2 = f_md2(a.3, &a.0);
        (primary(a.0, x), md0, md1, md2)
    }
}

#[test]
fn test_construction_dedup_entry_raw_md1() {
    let mut string_interner = crate::store::labels::LabelStore::new();
    let it = BIN_DUP
        .into_iter()
        .map(|x| x.map(|x| x.to_string(), |x| string_interner.get_or_insert(*x)));

    let backend = World::new();
    let dedup =
        hashbrown::HashMap::<_, (), ()>::with_capacity_and_hasher(1 << 10, Default::default());
    let builder = DefaultHashBuilder::default();

    let mut node_store = (backend, dedup, builder);

    let init = fn_zip_data_init(
        init,
        fn_zip0(md_sys_tree_size::init, md_sys_tree_hash::init),
        md_sys_byte_len::init,
        || ()
    );

    let acc = fn_zip_data_acc(
        |a, x| acc(a, *x),
        fn_zip3(md_sys_tree_size::acc, md_sys_tree_hash::acc),
        md_sys_byte_len::acc,
        |_, _| ()
    );

    let finish = fn_zip_data_finish(
        finish,
        |acc, prim| {
            fn_zip12(md_sys_tree_size::finish, |a, (x, s): ((_, _), _)| {
                md_sys_tree_hash::finish(a, (x.0, x.1, s))
            })(acc, (&prim.0, prim.1 .0.as_ref()))
        },
        |acc, prim| md_sys_byte_len::finish(acc, (&prim.0, prim.1 .0.as_ref(), !prim.2.is_empty())),
        |_, _| ()
    );

    let insert = |backend: &mut World, t, md0, md1, md2| insert(backend, t).insert((md0, md1, md2)).id();

    let x = construction(it, init, acc, |acc: _, l: Option<IdL>| {
        let (t, md0, md1, md2) = finish(acc, l);
        let eq = eq;

        let backend = &mut node_store.0;
        let dedup = &mut node_store.1;
        let builder = &node_store.2;

        let hash = md0.1.to_u64(); //crate::utils::make_hash(builder, &t);
        let entry = dedup.raw_entry_mut().from_hash(hash, |symbol| {
            let entity = backend.entity(*symbol);
            eq(entity, &t)
        });

        use hashbrown::hash_map::RawEntryMut::*;
        match entry {
            Occupied(occupied) => (*occupied.key(), md0, md1, md2),
            Vacant(vacant) => {
                let symbol = insert(backend, t, md0.clone(), md1.clone(), md2.clone());
                vacant.insert_with_hasher(hash, symbol, (), |id| {
                    let entity = backend.entity(*id);
                    entity.get::<md_sys_tree_hash::TreeHash>().unwrap().to_u64()
                });
                (symbol, md0, md1, md2)
            }
        }
    });
    dbg!(x);
}
