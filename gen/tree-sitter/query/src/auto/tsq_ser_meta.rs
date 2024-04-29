use hyper_ast::store;
use hyper_ast::types::{self, HyperType, IterableChildren};
use std::fmt::{Debug, Display};
use std::marker::PhantomData;
use std::sync::LazyLock;

pub struct TreeToQuery<
    'a,
    HAST: types::HyperAST<'a>,
    TIdN: hyper_ast::types::TypedNodeId,
    const PP: bool = true,
> {
    stores: &'a HAST,
    root: HAST::IdN,
    matcher: crate::search::PreparedMatcher<TIdN::Ty>,
    phantom: PhantomData<TIdN>,
}

static Q_STORE: LazyLock<QStore<crate::types::TStore>> = LazyLock::new(|| Default::default());

struct QStore<
    TS,
    NS = hyper_ast::store::nodes::DefaultNodeStore,
    LS = hyper_ast::store::labels::LabelStore,
>(std::sync::RwLock<hyper_ast::store::SimpleStores<TS, NS, LS>>);

impl<TS: Default> Default for QStore<TS> {
    fn default() -> Self {
        let stores = hyper_ast::store::SimpleStores::default();
        Self(std::sync::RwLock::new(stores))
    }
}
pub(crate) struct QStoreRef<
    'a,
    TS,
    NS = hyper_ast::store::nodes::DefaultNodeStore,
    LS = hyper_ast::store::labels::LabelStore,
>(std::sync::RwLockReadGuard<'a, hyper_ast::store::SimpleStores<TS, NS, LS>>);

impl<'a, TS, NS, LS> std::ops::Deref for QStoreRef<'a, TS, NS, LS> {
    type Target = store::SimpleStores<TS, NS, LS>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'store, 'a, TS> types::HyperAST<'store> for QStoreRef<'a, TS, store::nodes::DefaultNodeStore>
where
    // <TS as TypeStore>::Ty: HyperType + Send + Sync,
    TS: types::TypeStore<
        store::nodes::legion::HashedNodeRef<'store, store::nodes::DefaultNodeIdentifier>,
        Ty = types::AnyType,
    >,
{
    type IdN = store::nodes::DefaultNodeIdentifier;

    type Idx = u16;
    type Label = store::labels::DefaultLabelIdentifier;

    type T = store::nodes::legion::HashedNodeRef<'store, Self::IdN>;

    type NS = store::nodes::legion::NodeStore;

    fn node_store(&self) -> &Self::NS {
        &self.0.node_store
    }

    type LS = store::labels::LabelStore;

    fn label_store(&self) -> &Self::LS {
        &self.0.label_store
    }

    type TS = TS;

    fn type_store(&self) -> &Self::TS {
        &self.0.type_store
    }
}

impl<
        'store,
        'a,
        HAST: types::TypedHyperAST<'store, TIdN>,
        TIdN: hyper_ast::types::TypedNodeId<IdN = HAST::IdN>,
    > TreeToQuery<'store, HAST, TIdN>
where
    TIdN::Ty: for<'b> TryFrom<&'b str> + std::fmt::Debug,
    for<'b> <TIdN::Ty as TryFrom<&'b str>>::Error: std::fmt::Debug,
{
    pub fn new(stores: &'store HAST, root: HAST::IdN) -> TreeToQuery<'store, HAST, TIdN> {
        Self::with_pred(stores, root, "")
    }
    pub fn with_pred(
        stores: &'store HAST,
        root: HAST::IdN,
        matcher: &str,
    ) -> TreeToQuery<'store, HAST, TIdN> {
        use std::ops::Deref;
        let query_store = Q_STORE.deref();

        let query =
            crate::search::ts_query2(&mut query_store.0.write().unwrap(), matcher.as_bytes());
        let matcher = {
            let (root_types, patterns) = crate::search::PreparedMatcher::<TIdN::Ty>::new_aux(
                &query_store.0.read().unwrap(),
                query,
            );
            crate::search::PreparedMatcher::<TIdN::Ty>::with_patterns(root_types, patterns)
        };
        Self {
            stores,
            root,
            matcher,
            phantom: PhantomData,
        }
    }
}

impl<
        'store,
        HAST: types::TypedHyperAST<'store, TIdN>,
        TIdN: hyper_ast::types::TypedNodeId<IdN = HAST::IdN> + 'static,
        const PP: bool,
    > Display for TreeToQuery<'store, HAST, TIdN, PP>
where
    HAST::IdN: Debug + Copy,
    TIdN::Ty: for<'b> TryFrom<&'b str> + std::fmt::Debug,
    for<'b> <TIdN::Ty as TryFrom<&'b str>>::Error: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.serialize(&self.root, &mut 0, 0, f).map(|_| ())
    }
}

impl<
        'store,
        HAST: types::TypedHyperAST<'store, TIdN>,
        TIdN: hyper_ast::types::TypedNodeId<IdN = HAST::IdN> + 'static,
        const PP: bool,
    > TreeToQuery<'store, HAST, TIdN, PP>
where
    HAST::IdN: Debug + Copy,
    TIdN::Ty: for<'b> TryFrom<&'b str> + std::fmt::Debug + Eq + Copy,
    for<'b> <TIdN::Ty as TryFrom<&'b str>>::Error: std::fmt::Debug,
{
    // pub fn tree_syntax_with_ids(
    fn serialize(
        &self,
        id: &HAST::IdN,
        count: &mut usize,
        ind: usize,
        out: &mut std::fmt::Formatter<'_>,
    ) -> Result<(), std::fmt::Error> {
        use types::{LabelStore, Labeled, NodeStore, TypeStore, WithChildren};
        let b = NodeStore::resolve(self.stores.node_store(), id);
        // let kind = (self.stores.type_store(), b);
        let kind = self.stores.type_store().resolve_type(&b);
        let label = b.try_get_label();
        let children = b.children();

        if kind.is_spaces() {
            return Ok(());
        }

        match (label, children) {
            (None, None) => {
                write!(out, "\"{}\"", kind.to_string())?;
            }
            (_, Some(children)) => {
                if !children.is_empty() {
                    let it = children.iter_children();
                    write!(out, "(")?;
                    write!(out, "{}", kind.to_string())?;
                    for id in it {
                        let b = self.stores.node_store().resolve(id);
                        let kind = self.stores.type_store().resolve_type(&b);
                        if !kind.is_spaces() {
                            if PP {
                                write!(out, "\n{}", "  ".repeat(ind + 1))?;
                            } else {
                                write!(out, " ")?;
                            }
                        }
                        self.serialize(&id, count, ind + 1, out)?;
                    }
                    if PP {
                        write!(out, "\n{}", "  ".repeat(ind))?;
                    }
                    write!(out, ")")?;
                }
            }
            (Some(label), None) => {
                write!(out, "(")?;
                write!(out, "{}", kind.to_string())?;
                write!(out, ")")?;
                if self.matcher.is_matching::<_, TIdN>(self.stores, *id) {
                    let s = self.stores.label_store().resolve(label);
                    write!(out, " @id{} (#eq? @id{} \"{}\")", count, count, s)?;
                    *count += 1;
                }
            }
        }
        return Ok(());
    }
}
