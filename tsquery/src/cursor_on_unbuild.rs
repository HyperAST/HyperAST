//! TODO more difficult: make it backend agnostic, e.g., no ref to legion stuff

use crate::{Cursor, Node as _, Status, Symbol, TreeCursorStep};
use hyper_ast::position::TreePath;
use hyper_ast::tree_gen;
use hyper_ast::types::{
    self, HyperAST, HyperASTShared, HyperType as _, LabelStore as _, Labeled, Role, Tree, WithRoles,
};
use hyper_ast::types::{RoleStore as _, Stored};
use hyper_ast::{position::TreePathMut, types::TypeStore};
use num::ToPrimitive;

use types::ETypeStore as EnabledTypeStore;

pub type TreeCursor<HAST, Acc> = Node<HAST, Acc>;

pub struct Node<
    HAST: HyperASTShared,
    Acc: hyper_ast::tree_gen::WithLabel,
    Idx = <HAST as HyperASTShared>::Idx,
    P = hyper_ast::position::StructuralPosition<<HAST as HyperASTShared>::IdN, Idx>,
    L = <Acc as hyper_ast::tree_gen::WithLabel>::L,
> {
    pub stores: HAST,
    acc: Acc,
    label: Option<L>,
    offset: Idx,
    pub pos: P,
}

impl<HAST: HyperASTShared, Acc: hyper_ast::tree_gen::WithLabel> PartialEq for Node<HAST, Acc> {
    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos
    }
}

impl<HAST: HyperASTShared, Acc: hyper_ast::tree_gen::WithLabel> Node<HAST, Acc> {
    pub fn new(
        stores: HAST,
        acc: Acc,
        label: Option<Acc::L>,
        pos: hyper_ast::position::StructuralPosition<HAST::IdN, HAST::Idx>,
    ) -> Self {
        Self {
            stores,
            acc,
            label,
            offset: num::zero(),
            pos,
        }
    }
}

impl<'acc, 'l, HAST: HyperASTShared + Clone, Acc> Clone for Node<HAST, &'acc Acc>
where
    &'acc Acc: hyper_ast::tree_gen::WithLabel,
{
    fn clone(&self) -> Self {
        Self {
            stores: self.stores.clone(),
            acc: self.acc,
            label: self.label.clone(),
            offset: self.offset,
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

impl<'hast, 'acc, 'l, HAST, Acc> crate::Cursor for self::TreeCursor<HAST, &'acc Acc>
where
    HAST: HyperAST<'hast> + Clone,
    HAST::TS:
        EnabledTypeStore<Ty2 = Acc::Type> + hyper_ast::types::RoleStore<IdF = IdF, Role = Role>,
    HAST::T: hyper_ast::types::WithRoles,
    HAST::IdN: Copy,
    Acc: tree_gen::WithChildren<<HAST::T as Stored>::TreeId>
        + tree_gen::WithRole<Role>
        + types::Typed,
    &'acc Acc: hyper_ast::tree_gen::WithLabel,
{
    type Node = self::Node<HAST, &'acc Acc>;
    type NodeRef<'a>
        = &'a self::Node<HAST, &'acc Acc>
    where
        Self: 'a;

    fn goto_next_sibling_internal(&mut self) -> TreeCursorStep {
        // log::trace!(
        //     "{} {:?} {} {} {:?}",
        //     self.kind(),
        //     &self.pos,
        //     &self.offset,
        //     self.acc.simple.children.len(),
        //     self.acc.simple.children
        // );
        if let Some(p) = self.pos.parent() {
            //dbg!();
            let n = self.stores.resolve(p);
            use hyper_ast::types::Children;
            use hyper_ast::types::WithChildren;
            let Some(node) = n
                .children()
                .and_then(|x| x.get(*self.pos.offset().unwrap()))
            else {
                if self.resolve_type(p).is_hidden() {
                    let Some((_, o)) = &self.pos.pop() else {
                        panic!()
                        // if (self.offset as usize) < self.acc.simple.children.len() {
                        //     self.offset += 1;
                        //     return self.goto_next_sibling_internal();
                        // } else {
                        //     return TreeCursorStep::TreeCursorStepNone;
                        // }
                    };
                    if self.pos.node().is_none() {
                        if o.to_usize().unwrap() + 1 < self.acc.child_count() {
                            self.offset = *o + num::one();
                        } else {
                            return TreeCursorStep::TreeCursorStepNone;
                        }
                    }
                    // dbg!();
                    return self.goto_next_sibling_internal();
                } else {
                    return TreeCursorStep::TreeCursorStepNone;
                }
            };
            self.pos.inc(*node);
        } else if let Some(o) = self.pos.offset() {
            //dbg!();
            let Some(node) = self.acc.child((*o).to_usize().unwrap()) else {
                return TreeCursorStep::TreeCursorStepNone;
            };
            self.pos.inc(node);
        } else {
            //dbg!();
            self.offset += num::one();
            let o = self.offset;
            let Some(node) = self.acc.child(o.to_usize().unwrap()) else {
                return TreeCursorStep::TreeCursorStepNone;
            }; //dbg!(node);
            self.pos.goto(node, o);
        }
        if self.kind().is_spaces() {
            //dbg!();
            return self.goto_next_sibling_internal();
        }
        if self.is_visible() {
            TreeCursorStep::TreeCursorStepVisible
        } else {
            // log::trace!(
            //     "{} {:?} {} {}",
            //     self.kind(),
            //     &self.pos,
            //     &self.offset,
            //     self.acc.simple.children.len()
            // );
            TreeCursorStep::TreeCursorStepHidden
        }
    }

    fn goto_first_child_internal(&mut self) -> TreeCursorStep {
        if let Some(n) = self.pos.node() {
            // dbg!();
            let n = self.stores.resolve(n);
            use hyper_ast::types::Children;
            use hyper_ast::types::WithChildren;
            let Some(node) = n.children().and_then(|x| x.get(num::zero())) else {
                return TreeCursorStep::TreeCursorStepNone;
            };
            self.pos.goto(*node, num::zero());
        } else if let Some(o) = self.pos.offset() {
            // dbg!();
            let Some(node) = self.acc.child((*o).to_usize().unwrap()) else {
                return TreeCursorStep::TreeCursorStepNone;
            };
            self.pos.inc(node);
        } else {
            // dbg!();
            let Some(node) = self.acc.child(self.offset.to_usize().unwrap()) else {
                return TreeCursorStep::TreeCursorStepNone;
            };
            self.pos.goto(node, self.offset);
        }
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
            let Some((_, o)) = &self.pos.pop() else {
                return false;
            };
            if self.pos.node().is_none() {
                // at root of subtree
                self.offset = *o + num::one();
                // let o = self.pos.offset().unwrap();
                let Some(_) = self.acc.child((*o + num::one()).to_usize().unwrap()) else {
                    return false;
                };
                if self.is_visible() {
                    return true;
                }
                return false;
            }
            if self.is_visible() {
                return true;
            }
        }
    }

    fn current_node(&self) -> Self::NodeRef<'_> {
        self
    }

    fn parent_is_error(&self) -> bool {
        // NOTE: maybe more efficient impl
        let mut s = self.clone();
        if !s.goto_parent() {
            return false;
        }
        s.symbol().is_error()
    }

    fn has_parent(&self) -> bool {
        let mut node = self.clone();
        node.goto_parent()
    }

    fn persist(&mut self) -> Self::Node {
        self.clone()
    }

    fn persist_parent(&mut self) -> Option<Self::Node> {
        let mut node = self.clone();
        node.goto_parent();
        Some(node)
    }

    type Status = CursorStatus<IdF>;

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
                use crate::Node;
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

    fn text_provider(&self) -> <Self::Node as crate::Node>::TP<'_> {
        ()
    }

    fn is_visible_at_root(&self) -> bool {
        assert!(self.pos.node().is_none());
        self.is_visible()
    }
}

impl<'hast, 'acc, 'l, HAST, Acc> self::TreeCursor<HAST, &'acc Acc>
where
    HAST: HyperAST<'hast> + Clone,
    HAST::TS:
        EnabledTypeStore<Ty2 = Acc::Type> + hyper_ast::types::RoleStore<IdF = IdF, Role = Role>,
    HAST::T: hyper_ast::types::WithRoles,
    HAST::IdN: Copy,
    Acc: tree_gen::WithChildren<HAST::IdN> + tree_gen::WithRole<Role> + types::Typed,
    &'acc Acc: hyper_ast::tree_gen::WithLabel,
{
    fn role(&self) -> Option<Role> {
        if let Some(p) = self.pos.parent() {
            let n = self.stores.resolve(p);
            n.role_at::<Role>(self.pos.o().unwrap())
        } else {
            let idx = self.pos.o().unwrap();
            self.acc.role_at(idx.to_usize().unwrap())
        }
    }

    fn super_types(mut self) -> Vec<Symbol> {
        // TODO Might create efficiency issues, is it compiling well ?
        let mut result = vec![];
        loop {
            use crate::Node;
            if self.pos.pop().is_none() {
                return result;
            }
            let Some((_, o)) = &self.pos.pop() else {
                return result;
            };
            if self.pos.node().is_none() {
                // at root of subtree
                self.offset = *o + num::one();
                let Some(_) = self.acc.child((*o + num::one()).to_usize().unwrap()) else {
                    return result;
                };
                if self.is_visible() {
                    return result;
                }
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

    fn compute_current_role(&self) -> (Option<Role>, IdF) {
        let lang;
        let role = if self.pos.node().is_none() {
            lang = HAST::TS::intern(self.acc.get_type()).get_lang();
            // self.acc.role
            None // actually should not provide role as it is not part of identifying data
        } else if self.pos.parent().is_none() {
            lang = HAST::TS::intern(self.acc.get_type()).get_lang();
            let o = self.pos.o().unwrap();
            self.acc.role_at(o.to_usize().unwrap())
        } else {
            let mut p = self.clone();
            loop {
                let Some((_, o)) = p.pos.pop() else {
                    return (None, Default::default());
                };
                let Some(n) = p.pos.node() else {
                    return (None, Default::default());
                };
                let n = self.stores.resolve(n);
                // dbg!(p.kind());
                if p.kind().is_supertype() {
                    continue;
                }
                lang = p.kind().get_lang();
                break n.role_at::<Role>(o - num::one());
            }
        };
        let field_id = if let Some(role) = role {
            HAST::TS::intern_role(lang, role)
        } else {
            Default::default()
        };
        (role, field_id)
    }
}

type IdF = u16;

impl<'hast, 'acc, 'l, HAST, Acc> crate::Node for self::Node<HAST, &'acc Acc>
where
    HAST: HyperAST<'hast> + Clone,
    HAST::TS:
        EnabledTypeStore<Ty2 = Acc::Type> + hyper_ast::types::RoleStore<IdF = IdF, Role = Role>,
    HAST::T: hyper_ast::types::WithRoles,
    HAST::IdN: Copy,
    Acc: tree_gen::WithChildren<HAST::IdN> + tree_gen::WithRole<Role> + types::Typed,
    &'acc Acc: hyper_ast::tree_gen::WithLabel,
{
    fn symbol(&self) -> Symbol {
        let id = HAST::TS::ts_symbol(self.kind());
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

    type IdF = IdF;

    // fn child_by_field_id(&self, field_id: FieldId) -> Option<Self> {
    //     if field_id == 0 {
    //         return None;
    //     }
    //     let role = self.stores.type_store().resolve_field(field_id);
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
        if field_id == IdF::default() {
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
    type TP<'a> = ();
    fn text(&self, _tp: ()) -> std::borrow::Cow<str> {
        if let Some(id) = self.pos.node() {
            let n = self.stores.resolve(id);
            if n.has_children() {
                // dbg!();
                return "".into();
                // let r = hyper_ast::nodes::TextSerializer::new(self.stores, *id).to_string();
                // return r.into();
            }
            if let Some(l) = n.try_get_label() {
                let ls = self.stores.label_store();
                let l = ls.resolve(l);
                return l.to_string().into();
            }
            "".into()
        } else if !self.acc.child_count() == 0 {
            todo!()
        } else if let Some(label) = &self.label {
            label.as_ref().into()
        } else {
            "".into()
        }
    }
}

impl<'hast, 'acc, 'l, HAST, Acc> Node<HAST, &'acc Acc>
where
    HAST: HyperAST<'hast> + Clone,
    HAST::TS:
        EnabledTypeStore<Ty2 = Acc::Type> + hyper_ast::types::RoleStore<IdF = IdF, Role = Role>,
    HAST::T: hyper_ast::types::WithRoles,
    HAST::IdN: Copy,
    Acc: tree_gen::WithChildren<HAST::IdN> + tree_gen::WithRole<Role> + types::Typed,
    &'acc Acc: hyper_ast::tree_gen::WithLabel,
{
    fn child_by_role(&mut self, role: Role) -> Option<()> {
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

impl<'hast, 'acc, 'l, HAST, Acc> Node<HAST, &'acc Acc>
where
    HAST: HyperAST<'hast> + Clone,
    HAST::TS: EnabledTypeStore<Ty2 = Acc::Type>,
    HAST::IdN: Copy,
    Acc: tree_gen::WithChildren<HAST::IdN> + tree_gen::WithRole<Role> + types::Typed,
    &'acc Acc: hyper_ast::tree_gen::WithLabel,
{
    pub fn kind(&self) -> <HAST::TS as TypeStore>::Ty {
        if let Some(n) = self.pos.node() {
            self.resolve_type(n)
        } else {
            HAST::TS::intern(self.acc.get_type())
        }
    }

    fn resolve_type(&self, n: &HAST::IdN) -> <HAST::TS as TypeStore>::Ty {
        // TODO une a more generic accessor
        // TODO do not use the raw world, wrap it with the max fields, dissalowing just insertion
        // WARN migth have issues if using compressed components
        // dbg!(self.stores.node_store.resolve(n).get_component::<hyper_ast::types::Type>());
        self.stores.resolve_type(n)
    }
}

impl<'hast, 'acc, 'l, HAST, Acc> Node<HAST, &'acc Acc>
where
    HAST: HyperAST<'hast> + Clone,
    HAST::TS: EnabledTypeStore<Ty2 = Acc::Type>,
    HAST::IdN: Copy,
    Acc: tree_gen::WithChildren<HAST::IdN> + tree_gen::WithRole<Role> + types::Typed,
    &'acc Acc: hyper_ast::tree_gen::WithLabel,
{
    fn is_visible(&self) -> bool {
        !self.kind().is_hidden()
    }
}
