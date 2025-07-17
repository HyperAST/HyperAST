use crate::decompressed_tree_store::{Shallow, ShallowDecompressedTreeStore};
use crate::matchers::optimal::zs::str_distance_patched::QGram;
use hyperast::nodes::TextSerializer;
use hyperast::store::nodes::compo;
use hyperast::types;
use str_distance::DistanceMetric;
use types::{HyperAST, NodeId, WithMetaData};
use types::{HyperType as _, LabelStore as _, NodeStore as _};

pub mod bottom_up_matcher;
pub mod lazy_bottom_up_matcher;
pub mod lazy_leaves_matcher;
pub mod leaves_matcher;

pub trait Similarity {
    type HAST;
    type IdN;
    fn norm(hyperast: &Self::HAST, p: &[Self::IdN; 2]) -> f64;
    fn dist(hyperast: &Self::HAST, p: &[Self::IdN; 2]) -> usize;
}

pub struct TextSimilarity<HAST>(std::marker::PhantomData<HAST>);

impl<HAST> Similarity for TextSimilarity<HAST>
where
    HAST: HyperAST + Clone,
    HAST::Label: Eq + Copy,
    HAST::IdN: Copy,
    HAST::IdN: NodeId<IdN = HAST::IdN>,
{
    type HAST = HAST;
    type IdN = HAST::IdN;

    fn norm(hyperast: &Self::HAST, p: &[Self::IdN; 2]) -> f64 {
        if p[0] == p[1] {
            return 1.0;
        }
        let l = p.each_ref().map(|x| try_label(hyperast, *x));
        if l[0] == l[1] && !l[0].is_none() {
            return 1.0;
        }
        let l = p.each_ref().map(|x| retrieve_text(hyperast, *x));
        // let [src_l, dst_l] = l.each_ref().map(|x| x.as_bytes().into_iter());
        // 1.0_f64 - QGram::new(3).normalized(src_l, dst_l)
        let [src_l, dst_l] = l.each_ref().map(|x| x.as_bytes());
        crate::matchers::optimal::zs::qgrams::qgram_distance_hash_opti(src_l, dst_l)
    }

    fn dist(hyperast: &Self::HAST, p: &[Self::IdN; 2]) -> usize {
        if p[0] == p[1] {
            return 0;
        }
        let l = p.each_ref().map(|x| try_label(hyperast, *x));
        if l[0] == l[1] && !l[0].is_none() {
            return 0;
        }
        let l = p.each_ref().map(|x| retrieve_text(hyperast, *x));
        let [src_l, dst_l] = l.each_ref().map(|x| x.as_bytes().into_iter());
        QGram::new(3).distance(src_l, dst_l)
    }
}

fn try_label<HAST>(hyperast: &HAST, x: HAST::IdN) -> Option<HAST::Label>
where
    HAST: HyperAST + Clone,
    HAST::Label: Clone,
{
    use types::Labeled;
    let n = hyperast.node_store().resolve(&x);
    n.try_get_label().cloned()
}

fn retrieve_text<HAST>(hyperast: &HAST, x: HAST::IdN) -> std::borrow::Cow<'_, str>
where
    HAST: HyperAST + Clone,
    HAST::IdN: NodeId<IdN = HAST::IdN>,
{
    use types::Labeled;
    let n = hyperast.node_store().resolve(&x);
    let l = n.try_get_label();
    if let Some(l) = l {
        std::borrow::Cow::Borrowed(hyperast.label_store().resolve(l))
    } else {
        std::borrow::Cow::Owned(TextSerializer::new(hyperast, x).to_string())
    }
}

pub struct LabelSimilarity<HAST>(std::marker::PhantomData<HAST>);

impl<HAST> Similarity for LabelSimilarity<HAST>
where
    HAST: HyperAST + Clone,
    HAST::Label: Eq + Copy,
    HAST::IdN: Copy,
    HAST::IdN: NodeId<IdN = HAST::IdN>,
{
    type HAST = HAST;
    type IdN = HAST::IdN;

    fn norm(hyperast: &Self::HAST, p: &[Self::IdN; 2]) -> f64 {
        if p[0] == p[1] {
            return 1.0;
        }
        let l = p.each_ref().map(|x| try_label(hyperast, *x));
        if l[0] == l[1] && !l[0].is_none() {
            return 1.0;
        }
        if l[0].is_none() || l[1].is_none() {
            return 0.0;
        }
        let l = l.map(|x| x.unwrap());
        let l = l.map(|x| hyperast.label_store().resolve(&x));
        // let [src_l, dst_l] = l.each_ref().map(|x| x.as_bytes().into_iter());
        // 1.0_f64 - QGram::new(3).normalized(src_l, dst_l)
        let [src_l, dst_l] = l.each_ref().map(|x| x.as_bytes());
        crate::matchers::optimal::zs::qgrams::qgram_distance_hash_opti(src_l, dst_l)
    }

    fn dist(hyperast: &Self::HAST, p: &[Self::IdN; 2]) -> usize {
        if p[0] == p[1] {
            return 0;
        }
        let l = p.each_ref().map(|x| try_label(hyperast, *x));
        if l[0] == l[1] && !l[0].is_none() {
            return 0;
        }
        if l[0].is_none() || l[1].is_none() {
            return usize::MAX;
        }
        let l = l.map(|x| x.unwrap());
        let l = l.map(|x| hyperast.label_store().resolve(&x));
        let [src_l, dst_l] = l.each_ref().map(|x| x.as_bytes().into_iter());
        QGram::new(3).distance(src_l, dst_l)
    }
}

fn is_leaf_file<HAST, D, IdS, IdD>(stores: HAST, arena: &D, idd: IdD) -> bool
where
    HAST: HyperAST + Copy,
    D: ShallowDecompressedTreeStore<HAST, IdD, IdS>,
{
    let id = arena.original(&idd);
    let t = stores.resolve_type(&id);
    t.is_file()
}

fn is_leaf_sub_file<HAST, D, IdS, IdD>(stores: HAST, arena: &D, idd: IdD) -> bool
where
    HAST: HyperAST + Copy,
    D: ShallowDecompressedTreeStore<HAST, IdD, IdS>,
    for<'t> <HAST as types::AstLending<'t>>::RT: WithMetaData<compo::MemberImportCount>,
{
    let id = arena.original(&idd);
    let n = stores.node_store().resolve(&id);
    n.get_metadata().map_or(false, |x| x.0 == 1)
}

fn is_leaf_stmt<HAST, D, IdS, IdD>(stores: HAST, arena: &D, idd: IdD) -> bool
where
    HAST: HyperAST + Copy,
    for<'t> <HAST as types::AstLending<'t>>::RT: WithMetaData<compo::StmtCount>,
    D: ShallowDecompressedTreeStore<HAST, IdD, IdS>,
{
    let id = arena.original(&idd);
    let n = stores.node_store().resolve(&id);
    n.get_metadata().map_or(false, |x| x.0 == 1)
}

fn is_leaf<HAST, D, IdD, IdS>(stores: HAST, arena: &D, idd: IdD) -> bool
where
    HAST: HyperAST + Copy,
    IdS: Eq,
    IdD: Shallow<IdS>,
    D: ShallowDecompressedTreeStore<HAST, IdD, IdS>,
{
    use types::WithChildren;
    let o = arena.original(&idd);
    stores.node_store().resolve(&o).child_count() == num_traits::zero()
}

// it's an approximation because of the layered nature of the efficient variant of the leaf matching
fn leaf_count<HAST>(hyperast: HAST, x: HAST::IdN) -> usize
where
    HAST: HyperAST + Copy,
    for<'t> <HAST as types::AstLending<'t>>::RT: WithMetaData<compo::StmtCount>,
    for<'t> <HAST as types::AstLending<'t>>::RT: WithMetaData<compo::MemberImportCount>,
{
    let n = hyperast.node_store().resolve(&x);
    let t = hyperast.resolve_type(&x);
    if t.is_file() || t.is_directory() {
        return 10; // just use the less precise similarity threshold
    }
    let r = WithMetaData::<compo::MemberImportCount>::get_metadata(&n).map_or(0, |x| x.0 as usize);
    if r != 0 {
        return r;
    }
    WithMetaData::<compo::StmtCount>::get_metadata(&n).map_or(0, |x| x.0 as usize)
}
