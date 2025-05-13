use super::{Cursor, Status, Symbol, TextLending, TreeCursorStep};
use hyperast::position::TreePath;
use hyperast::types::{HyperASTShared, HyperType, LabelStore, Labeled, NodeId, RoleStore, Tree, WithRoles};
use hyperast::{
    position::TreePathMut,
    types::{HyperAST, TypeStore},
};
pub type TreeCursor<'hast, HAST> = Node<'hast, HAST>;

pub struct Node<'hast, HAST: HyperASTShared> {
    pub stores: &'hast HAST,
    pub pos: hyperast::position::StructuralPosition<HAST::IdN, HAST::Idx>,
}

impl<'hast, HAST: HyperAST> PartialEq for Node<'hast, HAST> {
    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos
    }
}

impl<'hast, HAST: HyperAST> Node<'hast, HAST> {
    pub fn new(
        stores: &'hast HAST,
        pos: hyperast::position::StructuralPosition<HAST::IdN, HAST::Idx>,
    ) -> Self {
        Self { stores, pos }
    }
}

impl<'hast, HAST: HyperAST> Clone for Node<'hast, HAST> {
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

impl<'hast, HAST: HyperAST> super::Cursor for self::TreeCursor<'hast, HAST>
where
    HAST::IdN: std::fmt::Debug + Copy,
    HAST::IdN: NodeId<IdN = HAST::IdN>,
    HAST::TS: RoleStore,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithRoles,
{
    type Node = self::Node<'hast, HAST>;

    fn goto_next_sibling_internal(&mut self) -> TreeCursorStep {
        use hyperast::types::NodeStore;
        let Some(p) = self.pos.parent() else {
            return TreeCursorStep::TreeCursorStepNone;
        };
        let n = self.stores.node_store().resolve(p);
        use hyperast::types::Children;
        use hyperast::types::WithChildren;
        let Some(node) = n.child(self.pos.offset().unwrap()) else {
            if self.stores.resolve_type(p).is_hidden() {
                self.pos.pop();
                return self.goto_next_sibling_internal();
            } else {
                return TreeCursorStep::TreeCursorStepNone;
            }
        };
        self.pos.inc(node);
        if self.kind().is_spaces() {
            return self.goto_next_sibling_internal();
        }
        if self.is_visible() {
            TreeCursorStep::TreeCursorStepVisible
        } else {
            TreeCursorStep::TreeCursorStepHidden
        }
    }

    fn goto_first_child_internal(&mut self) -> TreeCursorStep {
        use hyperast::types::NodeStore;
        let n = self.stores.node_store().resolve(self.pos.node().unwrap());
        use hyperast::types::WithChildren;
        let Some(node) = n.child(&num::zero()) else {
            return TreeCursorStep::TreeCursorStepNone;
        };
        self.pos.goto(node, num::zero());
        if self.kind().is_spaces() {
            return self.goto_next_sibling_internal();
        }
        if self.is_visible() {
            TreeCursorStep::TreeCursorStepVisible
        } else {
            TreeCursorStep::TreeCursorStepHidden
        }
    }

    fn goto_parent(&mut self) -> bool {
        loop {
            if self.pos.pop().is_none() {
                return false;
            }
            if self.pos.node().is_none() {
                return false;
            }
            if self.is_visible() {
                return true;
            }
        }
    }

    fn current_node(&self) -> Self::Node {
        self.clone()
    }

    fn parent_node(&self) -> Option<Self::Node> {
        // NOTE: maybe more efficient impl
        let mut s = self.clone();
        s.goto_parent().then_some(s.current_node())
    }

    type Status = CursorStatus<<<HAST as HyperAST>::TS as RoleStore>::IdF>;

    #[inline]
    fn current_status(&self) -> Self::Status {
        let (role, field_id) = self.compute_current_role();
        let mut has_later_siblings = false;
        let mut has_later_named_siblings = false;
        let mut can_have_later_siblings_with_this_field = false;
        let mut s = self.clone();
        loop {
            if let TreeCursorStep::TreeCursorStepNone = s.goto_next_sibling_internal() {
                break;
            }
            // dbg!(s.str_symbol());
            if role.is_some() && s.role() == role {
                can_have_later_siblings_with_this_field = true;
            }
            has_later_siblings = true;
            if s.kind().is_supertype() {
                has_later_named_siblings = true;
            }
            if s.is_visible() {
                has_later_siblings = true;
                use super::Node;
                if s.is_named() {
                    has_later_named_siblings = true;
                    break;
                }
            }
        }
        CursorStatus {
            has_later_siblings,
            has_later_named_siblings,
            can_have_later_siblings_with_this_field,
            field_id,
            supertypes: self.clone().super_types(),
        }
    }

    fn text_provider(&self) -> <Self::Node as TextLending<'_>>::TP {
        ()
    }
}

impl<'hast, HAST: HyperAST> self::TreeCursor<'hast, HAST>
where
    HAST::IdN: std::fmt::Debug + Copy,
    HAST::IdN: NodeId<IdN = HAST::IdN>,
    HAST::TS: RoleStore,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithRoles,
{
    fn role(&self) -> Option<<HAST::TS as RoleStore>::Role> {
        use hyperast::types::NodeStore;
        let n = self.stores.node_store().resolve(self.pos.parent().unwrap());
        n.role_at::<<HAST::TS as RoleStore>::Role>(self.pos.o().unwrap())
    }

    fn super_types(mut self) -> Vec<Symbol> {
        // TODO Might create efficiency issues, is it compiling well ?
        let mut result = vec![];
        loop {
            use super::Node;
            self.pos.pop();
            if self.pos.offset().is_none() {
                return result;
            }
            if self.is_visible() {
                return result;
            }
            if self.kind().is_supertype() {
                result.push(self.symbol())
            }
        }
    }

    fn compute_current_role(
        &self,
    ) -> (
        Option<<<HAST as HyperAST>::TS as RoleStore>::Role>,
        <<HAST as HyperAST>::TS as RoleStore>::IdF,
    ) {
        use hyperast::types::NodeStore;
        let mut p = self.clone();
        let lang;
        let role = loop {
            let Some((_, o)) = p.pos.pop() else {
                return (None, Default::default());
            };
            let Some(n) = p.pos.node() else {
                return (None, Default::default());
            };
            let n = self.stores.node_store().resolve(n);
            // dbg!(p.kind());
            if p.kind().is_supertype() {
                continue;
            }
            lang = p.kind().get_lang();
            break n.role_at::<<HAST::TS as RoleStore>::Role>(o - num::one());
        };
        let field_id = if let Some(role) = role {
            HAST::TS::intern_role(lang, role)
        } else {
            Default::default()
        };
        (role, field_id)
    }
}

impl<'a, 'hast, HAST: HyperAST> super::TextLending<'a> for self::Node<'hast, HAST> {
    type TP = ();
}

impl<'hast, HAST: HyperAST> super::Node for self::Node<'hast, HAST>
where
    HAST::IdN: std::fmt::Debug + Copy,
    HAST::IdN: NodeId<IdN = HAST::IdN>,
    HAST::TS: RoleStore,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithRoles,
{
    fn symbol(&self) -> Symbol {
        // TODO make something more efficient
        let id = HAST::TS::type_to_u16(self.kind());
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

    type IdF = <HAST::TS as RoleStore>::IdF;

    // fn child_by_field_id(&self, field_id: FieldId) -> Option<Self> {
    //     if field_id == 0 {
    //         return None;
    //     }
    //     let role = HAST::TS::resolve_field(field_id);
    //     let mut slf = self.clone();
    //     loop {
    //         if slf.kind().is_supertype() {
    //             match slf.goto_first_child_internal() {
    //                 TreeCursorStep::TreeCursorStepNone => panic!(),
    //                 TreeCursorStep::TreeCursorStepHidden => (),
    //                 TreeCursorStep::TreeCursorStepVisible => break,
    //             }
    //         } else {
    //             break;
    //         }
    //     }
    //     slf.child_by_role(role).and_then(|_| Some(slf))
    // }

    fn has_child_with_field_id(&self, field_id: Self::IdF) -> bool {
        if field_id == Default::default() {
            return false;
        }
        let role = HAST::TS::resolve_field(self.kind().get_lang(), field_id);
        let mut slf = self.clone();
        loop {
            if slf.kind().is_supertype() {
                match slf.goto_first_child_internal() {
                    TreeCursorStep::TreeCursorStepNone => panic!(),
                    TreeCursorStep::TreeCursorStepHidden => (),
                    TreeCursorStep::TreeCursorStepVisible => break,
                }
            } else {
                break;
            }
        }
        slf.child_by_role(role).is_some()
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
    fn text<'s, 'l>(
        &'s self,
        text_provider: <Self as super::TextLending<'l>>::TP,
    ) -> super::BiCow<'s, 'l, str> {
        let id = self.pos.node().unwrap();
        use hyperast::types::NodeStore;
        let n = self.stores.node_store().resolve(id);
        if n.has_children() {
            let r = hyperast::nodes::TextSerializer::new(self.stores, *id).to_string();
            return super::BiCow::Owned(r.into());
        }
        if let Some(l) = n.try_get_label() {
            let l = self.stores.label_store().resolve(l);
            return super::BiCow::A(l); // TODO use text_provider and ref label store in TP
        }
        super::BiCow::A("")
    }
}

impl<'hast, HAST: HyperAST> Node<'hast, HAST>
where
    HAST::IdN: std::fmt::Debug + Copy,
    HAST::IdN: NodeId<IdN = HAST::IdN>,
    HAST::TS: RoleStore,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithRoles,
{
    fn child_by_role(&mut self, role: <HAST::TS as RoleStore>::Role) -> Option<()> {
        // TODO what about multiple children with same role?
        // NOTE treesitter uses a bin tree for repeats
        let visible = self.is_visible();
        if let TreeCursorStep::TreeCursorStepNone = self.goto_first_child_internal() {
            return None;
        }
        loop {
            if let Some(r) = self.role() {
                if r == role {
                    return Some(());
                } else {
                    if let TreeCursorStep::TreeCursorStepNone = self.goto_next_sibling_internal() {
                        return None;
                    }
                    continue;
                }
            }
            // do not go down
            if visible {
                if let TreeCursorStep::TreeCursorStepNone = self.goto_next_sibling_internal() {
                    return None;
                }
            }
            // hidden node so can explore
            else {
                if self.child_by_role(role).is_some() {
                    return Some(());
                }
                if let TreeCursorStep::TreeCursorStepNone = self.goto_next_sibling_internal() {
                    return None;
                }
            }
        }
    }
}

impl<'hast, HAST: HyperAST> Node<'hast, HAST>
where
    HAST::IdN: std::fmt::Debug + Copy,
{
    fn kind(&self) -> <HAST::TS as TypeStore>::Ty {
        self.stores.resolve_type(self.pos.node().unwrap())
    }
}

impl<'hast, HAST: HyperAST> Node<'hast, HAST>
where
    HAST::IdN: std::fmt::Debug + Copy,
{
    fn is_visible(&self) -> bool {
        !self.kind().is_hidden()
    }
}
