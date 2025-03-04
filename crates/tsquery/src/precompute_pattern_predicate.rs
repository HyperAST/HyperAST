use hyperast::{
    tree_gen,
    types::{self, HyperAST},
};

#[derive(Default)]
pub struct PreparedQuerying<Q, HAST, Acc>(Q, std::marker::PhantomData<(HAST, Acc)>);

impl<'a, HAST, Acc> From<&'a crate::Query> for PreparedQuerying<&'a crate::Query, HAST, Acc> {
    fn from(value: &'a crate::Query) -> Self {
        Self(value, Default::default())
    }
}

impl<Q, HAST, Acc> std::ops::Deref for PreparedQuerying<Q, HAST, &Acc> {
    type Target = Q;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, HAST, Acc> tree_gen::Prepro<T> for PreparedQuerying<&crate::Query, HAST, Acc> {
    const USING: bool = false;

    fn preprocessing(&self, ty: T) -> Result<hyperast::scripting::Acc, String> {
        unimplemented!()
    }
}

impl<TS, T, Acc> tree_gen::More for PreparedQuerying<&crate::Query, (TS, T), Acc>
where
    TS: 'static
        + Clone
        + types::ETypeStore<Ty2 = Acc::Type>
        + types::RoleStore<IdF = u16, Role = types::Role>,
    T: types::WithRoles,
    T: types::Tree,
    T::TreeId: Copy,
    Acc: types::Typed + tree_gen::WithRole<types::Role> + tree_gen::WithChildren<T::TreeId>,
    for<'c> &'c Acc: tree_gen::WithLabel<L = &'c str>,
{
    type Acc = Acc;
    type T = T;
    type TS = TS;
    const ENABLED: bool = true;
    fn match_precomp_queries<
        HAST: HyperAST<IdN = <Self::T as types::Stored>::TreeId, TS = Self::TS, RT = Self::T>
            + std::clone::Clone,
    >(
        &self,
        stores: HAST,
        acc: &Acc,
        label: Option<&str>,
    ) -> tree_gen::PrecompQueries {
        if self.0.enabled_pattern_count() == 0 {
            return Default::default();
        }
        let pos = hyperast::position::StructuralPosition::empty();

        let cursor = crate::cursor_on_unbuild::TreeCursor::new(stores, acc, label, pos);
        let qcursor = self.0.matches_immediate(cursor); // TODO filter on height (and visibility?)
        let mut r = Default::default();
        for m in qcursor {
            assert!(m.pattern_index.to_usize() < 16);
            r |= 1 << m.pattern_index.to_usize() as u16;
        }
        r
    }
}

impl<TS, T, Acc> tree_gen::PreproTSG for PreparedQuerying<&crate::Query, (TS, T), Acc>
where
    TS: 'static
        + Clone
        + types::ETypeStore<Ty2 = Acc::Type>
        + types::RoleStore<IdF = u16, Role = types::Role>,
    T: types::WithRoles,
    T: types::Tree,
    T::TreeId: Copy,
    Acc: types::Typed + tree_gen::WithRole<types::Role> + tree_gen::WithChildren<T::TreeId>,
    for<'c> &'c Acc: tree_gen::WithLabel<L = &'c str>,
{
    const GRAPHING: bool = false;
    fn compute_tsg<
        HAST: HyperAST<IdN = <Self::T as types::Stored>::TreeId, TS = Self::TS, RT = Self::T>
            + std::clone::Clone,
    >(
        &self,
        _stores: HAST,
        _acc: &Acc,
        _label: Option<&str>,
    ) -> Result<usize, String> {
        Ok(0)
    }
}
