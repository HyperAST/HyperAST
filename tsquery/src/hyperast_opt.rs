use super::{Status, Symbol, TreeCursorStep};
use hyper_ast::position::structural_pos::{self, AAA, BBB};
use hyper_ast::types::{HyperAST, TypeStore};
use hyper_ast::types::{
    HyperASTShared, HyperType, LabelStore, Labeled, RoleStore, Tree, WithPrecompQueries, WithRoles,
};

pub struct TreeCursor<'hast, HAST: HyperASTShared> {
    pub stores: &'hast HAST,
    pub pos: structural_pos::CursorWithPersistance<HAST::IdN, HAST::Idx>,
}

pub struct Node<'hast, HAST: HyperASTShared> {
    pub stores: &'hast HAST,
    pub pos: structural_pos::PersistedNode<HAST::IdN, HAST::Idx>,
}

pub struct NodeRef<'a, 'hast, HAST: HyperASTShared> {
    pub stores: &'hast HAST,
    pub pos: structural_pos::RefNode<'a, HAST::IdN, HAST::Idx>,
}

struct ExtNodeRef<'a, 'hast, HAST: HyperASTShared> {
    pub stores: &'hast HAST,
    pub pos: structural_pos::ExtRefNode<'a, HAST::IdN, HAST::Idx>,
}

// impl<'hast, HAST: HyperAST<'hast>> PartialEq for Node<'hast, HAST> {
//     fn eq(&self, other: &Self) -> bool {
//         self.pos == other.pos
//     }
// }

impl<'hast, HAST: HyperAST<'hast>> TreeCursor<'hast, HAST> {
    pub fn new(
        stores: &'hast HAST,
        pos: structural_pos::CursorWithPersistance<HAST::IdN, HAST::Idx>,
    ) -> Self {
        Self { stores, pos }
    }
}

impl<'hast, HAST: HyperAST<'hast>> Clone for Node<'hast, HAST> {
    fn clone(&self) -> Self {
        Self {
            stores: self.stores,
            pos: self.pos.clone(),
        }
    }
}

pub struct CursorStatus<IdF> {
    pub has_later_siblings: bool,
    pub has_later_named_siblings: bool,
    pub can_have_later_siblings_with_this_field: bool,
    pub field_id: IdF,
    pub supertypes: Vec<Symbol>,
}

impl<IdF: Copy> Status for CursorStatus<IdF> {
    type IdF = IdF;

    fn has_later_siblings(&self) -> bool {
        self.has_later_siblings
    }

    fn has_later_named_siblings(&self) -> bool {
        self.has_later_named_siblings
    }

    fn can_have_later_siblings_with_this_field(&self) -> bool {
        self.can_have_later_siblings_with_this_field
    }

    fn field_id(&self) -> Self::IdF {
        self.field_id
    }

    fn has_supertypes(&self) -> bool {
        !self.supertypes.is_empty()
    }

    fn contains_supertype(&self, sym: Symbol) -> bool {
        self.supertypes.contains(&sym)
    }
}

impl<'hast, HAST: HyperAST<'hast>> super::Cursor for self::TreeCursor<'hast, HAST>
where
    HAST::IdN: std::fmt::Debug + Copy,
    HAST::TS: RoleStore<HAST::T>,
    HAST::T: WithRoles,
    HAST::T: WithPrecompQueries,
{
    type Node = self::Node<'hast, HAST>;
    type NodeRef<'a> = self::NodeRef<'a, 'hast, HAST> where Self: 'a;

    fn goto_next_sibling_internal(&mut self) -> TreeCursorStep {
        goto_next_sibling_internal(self.stores, &mut self.pos)
    }

    fn goto_first_child_internal(&mut self) -> TreeCursorStep {
        goto_first_child_internal(self.stores, &mut self.pos)
    }

    fn goto_parent(&mut self) -> bool {
        goto_parent(self.stores, &mut self.pos)
    }

    fn current_node(&self) -> Self::NodeRef<'_> {
        NodeRef {
            stores: &self.stores,
            pos: self.pos.ref_node(),
        }
    }

    fn parent_is_error(&self) -> bool {
        // NOTE: maybe more efficient impl
        let mut s = self.pos.ref_node();
        if !goto_parent(self.stores, &mut s) {
            return false;
        }
        symbol(self.stores, &s) == Symbol::ERROR
    }

    fn has_parent(&self) -> bool {
        let mut s = self.pos.ref_node();
        goto_parent(self.stores, &mut s)
    }

    fn persist(&mut self) -> Self::Node {
        Node {
            stores: &self.stores,
            pos: self.pos.persist(),
        }
    }

    fn persist_parent(&mut self) -> Option<Self::Node> {
        Some(Node {
            stores: &self.stores,
            pos: self.pos.persist_parent()?,
        })
    }

    type Status = CursorStatus<<<HAST as HyperAST<'hast>>::TS as RoleStore<HAST::T>>::IdF>;

    #[inline]
    fn current_status(&self) -> Self::Status {
        let (_role, field_id) = self.current_node().compute_current_role();
        let mut has_later_siblings = false;
        let mut has_later_named_siblings = false;
        let mut can_have_later_siblings_with_this_field = false;
        let mut s = ExtNodeRef {
            stores: self.stores,
            pos: self.pos.ext(),
        };
        loop {
            if let TreeCursorStep::TreeCursorStepNone =
                goto_next_sibling_internal(s.stores, &mut s.pos)
            {
                break;
            }
            if _role.is_some() && role(s.stores, &mut s.pos.clone()) == _role {
                can_have_later_siblings_with_this_field = true;
            }
            has_later_siblings = true;
            if kind(s.stores, &s.pos).is_supertype() {
                has_later_named_siblings = true;
            }
            if is_visible(s.stores, &s.pos) {
                has_later_siblings = true;
                if kind(s.stores, &s.pos).is_named() {
                    has_later_named_siblings = true;
                    break;
                }
            }
        }
        let supertypes = SuperTypeIter {
            stores: self.stores,
            pos: self.pos.ref_node(),
        }
        .collect();
        CursorStatus {
            has_later_siblings,
            has_later_named_siblings,
            can_have_later_siblings_with_this_field,
            field_id,
            supertypes,
        }
    }

    fn text_provider(&self) -> <Self::Node as super::Node>::TP<'_> {
        ()
    }

    fn is_visible_at_root(&self) -> bool {
        assert!(self.pos.ref_parent().is_none());
        is_visible(self.stores, &self.pos)
    }

    fn wont_match(&self, needed: u8) -> bool {
        if needed == 0 {
            return false;
        }
        use hyper_ast::types::NodeStore;
        let id = self.pos.node();
        let n = self.stores.node_store().resolve(&id);
        n.wont_match_given_precomputed_queries(needed)
    }
}

impl<'a, 'hast, HAST: HyperAST<'hast>> self::NodeRef<'a, 'hast, HAST>
where
    HAST::IdN: std::fmt::Debug + Copy,
    HAST::TS: RoleStore<HAST::T>,
    HAST::T: WithRoles,
    HAST::T: WithPrecompQueries,
{
    fn compute_current_role(
        mut self,
    ) -> (
        Option<<<HAST as HyperAST<'hast>>::TS as RoleStore<HAST::T>>::Role>,
        <<HAST as HyperAST<'hast>>::TS as RoleStore<HAST::T>>::IdF,
    ) {
        let lang;
        let role = loop {
            let o = self.pos.offset();
            if !self.pos.up() {
                return (None, Default::default());
            };
            let n = resolve(self.stores, &self.pos);
            // dbg!(self.kind());
            if kind(self.stores, &self.pos).is_supertype() {
                continue;
            }
            lang = kind(self.stores, &self.pos).get_lang();
            break n.role_at::<<HAST::TS as RoleStore<HAST::T>>::Role>(o);
        };
        let field_id = if let Some(role) = role {
            self.stores.type_store().intern_role(lang, role)
        } else {
            Default::default()
        };
        (role, field_id)
    }
}

impl<'hast, HAST: HyperAST<'hast>> super::Node for self::Node<'hast, HAST>
where
    HAST::IdN: std::fmt::Debug + Copy,
    HAST::TS: RoleStore<HAST::T>,
    HAST::T: WithRoles,
    HAST::T: WithPrecompQueries,
{
    fn symbol(&self) -> Symbol {
        // TODO make something more efficient
        let id = self.stores.type_store().type_to_u16(self.kind());
        id.into()
    }

    fn is_named(&self) -> bool {
        self.kind().is_named()
    }

    fn str_symbol(&self) -> &str {
        self.kind().as_static_str()
    }

    fn start_point(&self) -> tree_sitter::Point {
        // TODO
        tree_sitter::Point { row: 0, column: 0 }
    }

    type IdF = <HAST::TS as RoleStore<HAST::T>>::IdF;

    fn has_child_with_field_id(&self, field_id: Self::IdF) -> bool {
        if field_id == Default::default() {
            return false;
        }
        let role = self
            .stores
            .type_store()
            .resolve_field(self.kind().get_lang(), field_id);
        let mut slf = ExtNodeRef {
            stores: self.stores,
            pos: self.pos.ext(),
        };
        loop {
            if !kind(slf.stores, &slf.pos).is_supertype() {
                break;
            }
            match goto_first_child_internal(slf.stores, &mut slf.pos) {
                TreeCursorStep::TreeCursorStepNone => panic!(),
                TreeCursorStep::TreeCursorStepHidden => (),
                TreeCursorStep::TreeCursorStepVisible => break,
            }
        }
        child_by_role(self.stores, &mut slf.pos, role).is_some()
    }

    fn equal(&self, other: &Self) -> bool {
        &self.pos == &other.pos
    }

    fn compare(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering::*;
        let left = self;
        let right = other;
        if !left.equal(right) {
            return self.pos.cmp(&other.pos);
        }
        Equal
    }
    type TP<'a> = ();
    fn text(&self, _tp: ()) -> std::borrow::Cow<str> {
        text(self.stores, &self.pos)
    }
}

impl<'a, 'hast, HAST: HyperAST<'hast>> super::Node for self::NodeRef<'a, 'hast, HAST>
where
    HAST::IdN: std::fmt::Debug + Copy,
    HAST::TS: RoleStore<HAST::T>,
    HAST::T: WithRoles,
    HAST::T: WithPrecompQueries,
{
    fn symbol(&self) -> Symbol {
        // TODO make something more efficient
        let t = kind(self.stores, &self.pos);
        let id = self.stores.type_store().type_to_u16(t);
        id.into()
    }

    fn is_named(&self) -> bool {
        kind(self.stores, &self.pos).is_named()
    }

    fn str_symbol(&self) -> &str {
        kind(self.stores, &self.pos).as_static_str()
    }

    fn start_point(&self) -> tree_sitter::Point {
        // TODO
        tree_sitter::Point { row: 0, column: 0 }
    }

    type IdF = <HAST::TS as RoleStore<HAST::T>>::IdF;

    fn has_child_with_field_id(&self, field_id: Self::IdF) -> bool {
        if field_id == Default::default() {
            return false;
        }
        let role = self
            .stores
            .type_store()
            .resolve_field(kind(self.stores, &self.pos).get_lang(), field_id);
        let mut slf = ExtNodeRef {
            stores: self.stores,
            pos: self.pos.ext(),
        };
        loop {
            if !kind(slf.stores, &slf.pos).is_supertype() {
                break;
            }
            match goto_first_child_internal(slf.stores, &mut slf.pos) {
                TreeCursorStep::TreeCursorStepNone => panic!(),
                TreeCursorStep::TreeCursorStepHidden => (),
                TreeCursorStep::TreeCursorStepVisible => break,
            }
        }
        child_by_role(self.stores, &mut slf.pos, role).is_some()
    }

    fn equal(&self, other: &Self) -> bool {
        &self.pos == &other.pos
    }

    fn compare(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering::*;
        let left = self;
        let right = other;
        if !left.equal(right) {
            return self.pos.cmp(&other.pos);
        }
        Equal
    }
    type TP<'b> = ();
    fn text(&self, _tp: ()) -> std::borrow::Cow<str> {
        text(self.stores, &self.pos)
    }
}

impl<'hast, HAST: HyperAST<'hast>> Node<'hast, HAST>
where
    HAST::IdN: std::fmt::Debug + Copy,
{
    fn kind(&self) -> <HAST::TS as TypeStore<HAST::T>>::Ty {
        kind(self.stores, &self.pos)
    }
}

fn kind<'hast, HAST: HyperAST<'hast>>(
    stores: &'hast HAST,
    pos: &impl AAA<HAST::IdN, HAST::Idx>,
) -> <HAST::TS as TypeStore<HAST::T>>::Ty {
    stores.resolve_type(&pos.node())
}

fn resolve<'hast, HAST: HyperAST<'hast>>(
    stores: &'hast HAST,
    pos: &impl AAA<HAST::IdN, HAST::Idx>,
) -> HAST::T {
    let n = pos.node();
    use hyper_ast::types::NodeStore;
    let n = stores.node_store().resolve(&n);
    n
}

fn is_visible<'hast, HAST: HyperAST<'hast>>(
    stores: &'hast HAST,
    pos: &impl AAA<HAST::IdN, HAST::Idx>,
) -> bool {
    !kind(stores, pos).is_hidden()
}

fn symbol<'hast, HAST: HyperAST<'hast>>(
    stores: &'hast HAST,
    pos: &impl AAA<HAST::IdN, HAST::Idx>,
) -> Symbol {
    // TODO make something more efficient
    let id = stores.type_store().type_to_u16(kind(stores, pos));
    id.into()
}

fn text<'a, 'hast, HAST: HyperAST<'hast>>(
    stores: &'hast HAST,
    pos: &impl AAA<HAST::IdN, HAST::Idx>,
) -> std::borrow::Cow<'hast, str> {
    let id = pos.node();
    use hyper_ast::types::NodeStore;
    let n = stores.node_store().resolve(&id);
    if n.has_children() {
        let r = hyper_ast::nodes::TextSerializer::new(stores, id).to_string();
        return r.into();
    }
    if let Some(l) = n.try_get_label() {
        let l = stores.label_store().resolve(l);
        return l.into();
    }
    "".into()
}

fn role<'hast, HAST: HyperAST<'hast>>(
    stores: &'hast HAST,
    pos: &mut impl AAA<HAST::IdN, HAST::Idx>,
) -> Option<<HAST::TS as RoleStore<HAST::T>>::Role>
where
    // HAST::IdN: std::fmt::Debug + Copy,
    HAST::TS: RoleStore<HAST::T>,
    HAST::T: WithRoles,
{
    let at = pos.offset();
    if !pos.up() {
        return None;
    }
    let n = resolve(stores, pos);
    n.role_at::<<HAST::TS as RoleStore<HAST::T>>::Role>(at)
}

struct SuperTypeIter<'a, 'hast, HAST: HyperASTShared> {
    pub stores: &'hast HAST,
    pub pos: structural_pos::RefNode<'a, HAST::IdN, HAST::Idx>,
}

impl<'a, 'hast, HAST: HyperAST<'hast>> Iterator for SuperTypeIter<'a, 'hast, HAST>
where
    HAST::IdN: std::fmt::Debug + Copy,
    HAST::TS: RoleStore<HAST::T>,
    HAST::T: WithRoles,
    HAST::T: WithPrecompQueries,
{
    type Item = Symbol;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let p = self.pos.parent()?;
            let k = self.stores.resolve_type(&p);
            if !k.is_hidden() {
                return None;
            }
            if k.is_supertype() {
                let symbol = symbol(self.stores, &self.pos);
                assert!(self.pos.up());
                return Some(symbol);
            }
            assert!(self.pos.up());
        }
    }
}

fn goto_parent<'hast, HAST: HyperAST<'hast>>(
    stores: &'hast HAST,
    pos: &mut impl AAA<HAST::IdN, HAST::Idx>,
) -> bool {
    loop {
        if !pos.up() {
            return false;
        }
        if is_visible(stores, pos) {
            return true;
        }
    }
}

fn goto_next_sibling_internal<'hast, HAST: HyperAST<'hast>>(
    stores: &'hast HAST,
    pos: &mut impl BBB<HAST::IdN, HAST::Idx>,
) -> TreeCursorStep
where
    HAST::IdN: Copy,
{
    use hyper_ast::types::NodeStore;
    let Some(p) = pos.parent() else {
        return TreeCursorStep::TreeCursorStepNone;
    };
    let n = stores.node_store().resolve(&p);
    use hyper_ast::types::Children;
    use hyper_ast::types::WithChildren;
    let Some(node) = n.children().and_then(|x| x.get(pos.offset() + num::one())) else {
        if stores.resolve_type(&p).is_hidden() {
            pos.up();
            return goto_next_sibling_internal(stores, pos);
        } else {
            return TreeCursorStep::TreeCursorStepNone;
        }
    };
    pos.inc(*node);
    if kind(stores, pos).is_spaces() {
        return goto_next_sibling_internal(stores, pos);
    }
    if is_visible(stores, pos) {
        TreeCursorStep::TreeCursorStepVisible
    } else {
        TreeCursorStep::TreeCursorStepHidden
    }
}

fn goto_first_child_internal<'hast, HAST: HyperAST<'hast>>(
    stores: &'hast HAST,
    pos: &mut impl BBB<HAST::IdN, HAST::Idx>,
) -> TreeCursorStep
where
    HAST::IdN: Copy,
{
    use hyper_ast::types::NodeStore;
    let n = stores.node_store().resolve(&pos.node());
    use hyper_ast::types::Children;
    use hyper_ast::types::WithChildren;
    let Some(node) = n.children().and_then(|x| x.get(num::zero())) else {
        return TreeCursorStep::TreeCursorStepNone;
    };
    pos.down(*node, num::zero());
    if kind(stores, pos).is_spaces() {
        return goto_next_sibling_internal(stores, pos);
    }
    if is_visible(stores, pos) {
        TreeCursorStep::TreeCursorStepVisible
    } else {
        TreeCursorStep::TreeCursorStepHidden
    }
}

fn child_by_role<'hast, HAST: HyperAST<'hast>>(
    stores: &'hast HAST,
    pos: &mut (impl BBB<HAST::IdN, HAST::Idx> + Clone),
    _role: <HAST::TS as RoleStore<HAST::T>>::Role,
) -> Option<()>
where
    <HAST as HyperAST<'hast>>::TS: RoleStore<<HAST as HyperAST<'hast>>::T>,
    <HAST as HyperASTShared>::IdN: Copy,
    HAST::TS: RoleStore<HAST::T>,
    HAST::T: WithRoles,
{
    // TODO what about multiple children with same role?
    // NOTE treesitter uses a bin tree for repeats
    let visible = is_visible(stores, pos);
    if let TreeCursorStep::TreeCursorStepNone = goto_first_child_internal(stores, pos) {
        return None;
    }
    loop {
        if let Some(r) = role(stores, &mut pos.clone()) {
            if r == _role {
                return Some(());
            } else {
                if let TreeCursorStep::TreeCursorStepNone = goto_next_sibling_internal(stores, pos)
                {
                    return None;
                }
                continue;
            }
        }
        // do not go down
        if visible {
            if let TreeCursorStep::TreeCursorStepNone = goto_next_sibling_internal(stores, pos) {
                return None;
            }
        }
        // hidden node so can explore
        else {
            if child_by_role(stores, pos, _role).is_some() {
                return Some(());
            }
            if let TreeCursorStep::TreeCursorStepNone = goto_next_sibling_internal(stores, pos) {
                return None;
            }
        }
    }
}
