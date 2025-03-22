use hyperast::store;
use hyperast::types::{self, Childrn, HyperASTShared, HyperType};
use std::fmt::{Debug, Display};
use std::marker::PhantomData;
use std::sync::LazyLock;

pub struct TreeToQuery<
    'a,
    HAST: types::HyperAST,
    TIdN: hyperast::types::TypedNodeId,
    C: Converter,
    const PP: bool = true,
> {
    stores: &'a HAST,
    root: HAST::IdN,
    matcher: crate::search::PreparedMatcher<TIdN::Ty, C>,
    converter: C,
    phantom: PhantomData<TIdN>,
}

static Q_STORE: LazyLock<QStore<crate::types::TStore>> = LazyLock::new(|| Default::default());

struct QStore<
    TS,
    NS = hyperast::store::nodes::DefaultNodeStore,
    LS = hyperast::store::labels::LabelStore,
>(std::sync::RwLock<hyperast::store::SimpleStores<TS, NS, LS>>);

impl<TS: Default> Default for QStore<TS> {
    fn default() -> Self {
        let stores = hyperast::store::SimpleStores::default();
        Self(std::sync::RwLock::new(stores))
    }
}
pub(crate) struct QStoreRef<
    'a,
    TS,
    NS = hyperast::store::nodes::DefaultNodeStore,
    LS = hyperast::store::labels::LabelStore,
>(std::sync::RwLockReadGuard<'a, hyperast::store::SimpleStores<TS, NS, LS>>);

impl<'a, TS, NS, LS> std::ops::Deref for QStoreRef<'a, TS, NS, LS> {
    type Target = store::SimpleStores<TS, NS, LS>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, TS> types::HyperASTShared for QStoreRef<'a, TS, store::nodes::DefaultNodeStore> {
    type IdN = store::nodes::DefaultNodeIdentifier;
    type Idx = u16;
    type Label = store::labels::DefaultLabelIdentifier;
}

impl<'a, 'b, TS> hyperast::types::NLending<'a, <Self as HyperASTShared>::IdN>
    for QStoreRef<'b, TS, store::nodes::DefaultNodeStore>
{
    type N = <store::nodes::DefaultNodeStore as hyperast::types::NLending<
        'a,
        <Self as HyperASTShared>::IdN,
    >>::N;
}

impl<'a, 'b, TS> hyperast::types::AstLending<'a>
    for QStoreRef<'b, TS, store::nodes::DefaultNodeStore>
where
    TS: types::TypeStore<Ty = types::AnyType>,
{
    type RT = <store::nodes::DefaultNodeStore as hyperast::types::NLending<'a, Self::IdN>>::N;
}

impl<'a, TS> types::HyperAST for QStoreRef<'a, TS, store::nodes::DefaultNodeStore>
where
    TS: types::TypeStore<Ty = types::AnyType>,
{
    type NS = store::nodes::legion::NodeStore;

    fn node_store(&self) -> &Self::NS {
        &self.0.node_store
    }

    type LS = store::labels::LabelStore;

    fn label_store(&self) -> &Self::LS {
        &self.0.label_store
    }

    type TS = TS;
}

pub trait Converter: Default {
    type Ty;
    fn conv(s: &str) -> Option<Self::Ty>;
}

pub struct Conv<Ty>(PhantomData<Ty>);

impl<Ty> Default for Conv<Ty> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<Ty> Converter for Conv<Ty>
where
    Ty: for<'b> TryFrom<&'b str> + std::fmt::Debug,
    for<'b> <Ty as TryFrom<&'b str>>::Error: std::fmt::Debug,
{
    type Ty = Ty;

    fn conv(s: &str) -> Option<Self::Ty> {
        s.try_into().ok()
    }
}

impl<'store, 'a, HAST: types::TypedHyperAST<TIdN>, TIdN: hyperast::types::TypedNodeId>
    TreeToQuery<'store, HAST, TIdN, Conv<TIdN::Ty>>
where
    TIdN::Ty: for<'b> TryFrom<&'b str> + std::fmt::Debug,
    for<'b> <TIdN::Ty as TryFrom<&'b str>>::Error: std::fmt::Debug,
{
    pub fn new(
        stores: &'store HAST,
        root: HAST::IdN,
    ) -> TreeToQuery<'store, HAST, TIdN, Conv<TIdN::Ty>> {
        Self::with_pred(stores, root, "")
    }
}

impl<
        'store,
        'a,
        HAST: types::TypedHyperAST<TIdN>,
        TIdN: hyperast::types::TypedNodeId,
        C: Converter<Ty = TIdN::Ty>,
    > TreeToQuery<'store, HAST, TIdN, C>
{
    pub fn with_pred(
        stores: &'store HAST,
        root: HAST::IdN,
        matcher: &str,
    ) -> TreeToQuery<'store, HAST, TIdN, C> {
        use std::ops::Deref;
        let query_store = Q_STORE.deref();

        let query =
            crate::search::ts_query2(&mut query_store.0.write().unwrap(), matcher.as_bytes());
        let matcher = {
            let preparing = crate::search::PreparedMatcher::<TIdN::Ty, C>::new_aux(
                query_store.0.read().unwrap().with_ts(),
                query,
            );
            preparing.into()
        };
        Self {
            stores,
            root,
            matcher,
            converter: Default::default(),
            phantom: PhantomData,
        }
    }
}

impl<
        'store,
        HAST: types::TypedHyperAST<TIdN>,
        TIdN: hyperast::types::TypedNodeId + 'static,
        F: Converter<Ty = TIdN::Ty>,
        const PP: bool,
    > Display for TreeToQuery<'store, HAST, TIdN, F, PP>
where
    HAST::IdN: Debug + Copy,
    TIdN::Ty: types::TypeTrait,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.serialize(&self.root, &mut 0, 0, f).map(|_| ())
    }
}

impl<
        'store,
        HAST: types::TypedHyperAST<TIdN>,
        TIdN: hyperast::types::TypedNodeId + 'static,
        F: Converter<Ty = TIdN::Ty>,
        const PP: bool,
    > TreeToQuery<'store, HAST, TIdN, F, PP>
where
    HAST::IdN: Debug + Copy,
    TIdN::Ty: types::TypeTrait,
{
    // pub fn tree_syntax_with_ids(
    fn serialize(
        &self,
        id: &HAST::IdN,
        count: &mut usize,
        ind: usize,
        out: &mut std::fmt::Formatter<'_>,
    ) -> Result<(), std::fmt::Error> {
        use types::{LabelStore, Labeled, NodeStore, WithChildren};
        let b = self.stores.node_store().resolve(id);
        // let kind = (self.stores.type_store(), b);
        let kind = self.stores.resolve_type(id);
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
                        let kind = self.stores.resolve_type(&id);
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
