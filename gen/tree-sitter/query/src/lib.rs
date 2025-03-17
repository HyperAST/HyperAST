pub mod types;

#[cfg(feature = "impl")]
pub mod legion;

#[cfg(feature = "impl")]
#[cfg(test)]
pub mod tests;

#[cfg(feature = "legion")]
mod tnode {
    pub use hyperast::tree_gen::utils_ts::TNode;
}

use auto::tsq_ser_meta::Conv;
use hyperast::types::{HyperType, TypeTrait};
use search::Captured;
#[cfg(feature = "legion")]
pub use tnode::TNode;

pub mod search;

#[cfg(feature = "legion")]
pub mod iter;

pub mod auto;

pub fn prepare_matcher<Ty>(query: &str) -> crate::search::PreparedMatcher<Ty, Conv<Ty>>
where
    Ty: std::fmt::Debug,
    Ty: for<'a> TryFrom<&'a str>,
    for<'a> <Ty as TryFrom<&'a str>>::Error: std::fmt::Debug,
{
    let (query_store, query) = crate::search::ts_query(query.as_bytes());
    let prepared_matcher =
        crate::search::PreparedMatcher::<Ty, Conv<Ty>>::new(query_store.with_ts(), query);
    prepared_matcher
}

pub struct IterMatched<M, HAST, It, TIdN> {
    iter: It,
    matcher: M,
    pub hast: HAST,
    _phantom: std::marker::PhantomData<TIdN>,
}

impl<'a, HAST, It: Iterator, TIdN> Iterator
    for IterMatched<&crate::search::PreparedMatcher<TIdN::Ty, Conv<TIdN::Ty>>, &'a HAST, It, TIdN>
where
    HAST: hyperast::types::HyperAST + hyperast::types::TypedHyperAST<TIdN>,
    TIdN: 'static + hyperast::types::TypedNodeId, //<IdN = <HAST as hyperast::types::HyperASTShared>::IdN>,
    It::Item:
        hyperast::position::TreePath<TIdN::IdN, <HAST as hyperast::types::HyperASTShared>::Idx>,
    for<'b> &'b str: Into<<TIdN as hyperast::types::TypedNodeId>::Ty>,
    TIdN::IdN: Copy,
    TIdN::Ty: TypeTrait,
    // TIdN::IdN: hyperast::types::NodeId<IdN = TIdN::IdN>,
{
    type Item = (It::Item, Captured<HAST::IdN, HAST::Idx>);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(e) = self.iter.next() {
            use hyperast::position::TreePath;
            if let Some(c) = self
                .matcher
                .is_matching_and_capture::<_, TIdN>(self.hast, *e.node().unwrap())
            {
                return Some((e, c));
            }
        }
        None
    }
}

// impl<Ty> crate::search::PreparedMatcher<Ty> {
//     pub fn apply_matcher<'a, HAST, It, TIdN>(
//         &self,
//         hast: &'a HAST,
//         root: TIdN::IdN,
//     ) -> IterMatched<&crate::search::PreparedMatcher<Ty>, &'a HAST, It, TIdN>
//     where
//         HAST: 'a + hyperast::types::HyperAST<'a>,
//         TIdN: hyperast::types::TypedNodeId<Ty = Ty, IdN = HAST::IdN>,
//         It: From<(&'a HAST, It::Item)>,
//         It::Item: From<HAST::IdN>,
//         It: Iterator,
//         It::Item: hyperast::position::TreePathMut<HAST::IdN, HAST::Idx>,
//     {
//         let path = It::Item::from(root);
//         let iter = It::from((hast, path));
//         IterMatched {
//             iter,
//             matcher: self,
//             hast,
//             _phantom: std::marker::PhantomData,
//         }
//     }
// }

impl<Ty: HyperType + std::hash::Hash + Copy + Eq + Send + Sync>
    crate::search::PreparedMatcher<Ty, Conv<Ty>>
{
    // pub fn apply_matcher<'a, HAST, It, TIdN>(
    //     &self,
    //     hast: &'a HAST,
    //     root: HAST::IdN,
    // ) -> IterMatched2<
    //     crate::search::recursive2::MatchingIter<
    //         'a,
    //         HAST,
    //         TIdN,
    //         &crate::search::PreparedMatcher<TIdN::Ty, Conv<Ty>>,
    //     >,
    //     &'a HAST,
    //     It,
    //     TIdN,
    // >
    // where
    //     HAST: hyperast::types::TypedHyperAST<TIdN> + for<'t> hyperast::types::TypedLending<'t, Ty>,
    //     // HAST::TS: hyperast::types::TypeStore<Ty = Ty>,
    //     TIdN: hyperast::types::TypedNodeId,//<Ty = Ty>,
    //     It: From<(&'a HAST, It::Item)>,
    //     It::Item: From<HAST::IdN>,
    //     It: Iterator,
    //     It::Item: hyperast::position::TreePathMut<HAST::IdN, HAST::Idx>,
    // {
    //     let path = It::Item::from(root.clone());
    //     let mut iter = It::from((hast, path));
    //     let cur = iter.next().unwrap();
    //     IterMatched2 {
    //         iter,
    //         cur,
    //         matcher: crate::search::recursive2::MatchingIter::new(self, hast, root),
    //         hast,
    //         _phantom: std::marker::PhantomData,
    //     }
    // }
}

pub struct IterMatched2<M, HAST, It: Iterator, TIdN> {
    iter: It,
    cur: It::Item,
    matcher: M,
    pub hast: HAST,
    _phantom: std::marker::PhantomData<TIdN>,
}

impl<'a, HAST, It: Iterator, TIdN> Iterator
    for IterMatched2<
        crate::search::recursive2::MatchingIter<
            'a,
            HAST,
            TIdN,
            &crate::search::PreparedMatcher<TIdN::Ty, Conv<TIdN::Ty>>,
        >,
        &'a HAST,
        It,
        TIdN,
    >
where
    HAST: hyperast::types::HyperAST + hyperast::types::TypedHyperAST<TIdN>,
    TIdN: 'static + hyperast::types::TypedNodeId, //<IdN = <HAST as hyperast::types::HyperASTShared>::IdN>,
    It::Item: hyperast::position::TreePath<TIdN::IdN, <HAST as hyperast::types::HyperASTShared>::Idx>
        + Clone,
    for<'b> &'b str: Into<<TIdN as hyperast::types::TypedNodeId>::Ty>,
    TIdN::IdN: Copy + std::fmt::Debug,
{
    type Item = (It::Item, Captured<HAST::IdN, HAST::Idx>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(c) = self.matcher.next() {
            return Some((self.cur.clone(), c));
        } else if let Some(e) = self.iter.next() {
            use hyperast::position::TreePath;
            self.cur = e;
            self.matcher.repurpose(*self.cur.node().unwrap());
            return self.next();
        }
        None
    }
}

#[cfg(feature = "impl")]
pub fn language() -> tree_sitter::Language {
    tree_sitter_query::language()
}

#[cfg(feature = "impl")]
pub fn node_types() -> &'static str {
    tree_sitter_query::NODE_TYPES
}
