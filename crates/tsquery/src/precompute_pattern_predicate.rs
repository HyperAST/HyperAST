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

impl<HAST, Acc> tree_gen::More for PreparedQuerying<&crate::Query, HAST, Acc>
where
    HAST: types::HyperAST,
    HAST::TS: 'static + Clone + types::ETypeStore + types::RoleStore<IdF = u16, Role = types::Role>,
    HAST::IdN: Copy,
    Acc: types::Typed<Type = <HAST::TS as types::ETypeStore>::Ty2>
        + tree_gen::WithRole<types::Role>
        + tree_gen::WithChildren<HAST::IdN>,
    for<'c> &'c Acc: tree_gen::WithLabel<L = &'c str>,
    // T: for<'t> types::AstLending<'t>,
    // for<'t> types::LendT<'t, T>: types::WithChildren + types::WithRoles,
    for<'t> types::LendT<'t, HAST>: types::WithRoles,
    HAST::TM:
        hyperast::types::MarkedT<TreeId = HAST::IdN, Label = HAST::Label, ChildIdx = HAST::Idx>,
{
    type Acc = Acc;
    type T = HAST::TM;
    type TS = HAST::TS;
    const ENABLED: bool = true;
    fn match_precomp_queries<
        HAST2: types::HyperASTShared<IdN = HAST::IdN, Label = HAST::Label, Idx = HAST::Idx>
            + for<'t> types::AstLending<'t, RT = types::LendN<'t, Self::T, HAST::IdN>>
            + HyperAST<
                // TM = HAST::TM,
                TS = HAST::TS,
            > + std::clone::Clone,
    >(
        &self,
        stores: HAST2,
        acc: &Acc,
        label: Option<&str>,
    ) -> tree_gen::PrecompQueries {
        if self.0.enabled_pattern_count() == 0 {
            return Default::default();
        }
        let pos = hyperast::position::StructuralPosition::empty();

        let cursor = crate::cursor_on_unbuild::TreeCursor::new(stores, acc, label, pos);
        let mut qcursor = self.0.matches_immediate(cursor); // TODO filter on height (and visibility?)
        let mut r = Default::default();
        while let Some(m) = qcursor.next() {
            assert!(m.pattern_index.to_usize() < 16);
            r |= 1 << m.pattern_index.to_usize() as u16;
        }
        r
    }
}

impl<HAST, Acc> tree_gen::PreproTSG for PreparedQuerying<&crate::Query, HAST, Acc>
where
    HAST: types::HyperAST,
    HAST::TS: 'static + Clone + types::ETypeStore + types::RoleStore<IdF = u16, Role = types::Role>,
    HAST::IdN: Copy,
    Acc: types::Typed<Type = <HAST::TS as types::ETypeStore>::Ty2>
        + tree_gen::WithRole<types::Role>
        + tree_gen::WithChildren<HAST::IdN>,
    for<'c> &'c Acc: tree_gen::WithLabel<L = &'c str>,
    // T: for<'t> types::AstLending<'t>,
    // for<'t> types::LendT<'t, T>: types::WithChildren + types::WithRoles,
    for<'t> types::LendT<'t, HAST>: types::WithRoles,
    HAST::TM:
        hyperast::types::MarkedT<TreeId = HAST::IdN, Label = HAST::Label, ChildIdx = HAST::Idx>,
{
    const GRAPHING: bool = false;
    fn compute_tsg<
        HAST2: HyperAST<IdN = <Self::T as types::Stored>::TreeId, TS = Self::TS> + std::clone::Clone,
    >(
        &self,
        _stores: HAST2,
        _acc: &Acc,
        _label: Option<&str>,
    ) -> Result<usize, String> {
        Ok(0)
    }
}
