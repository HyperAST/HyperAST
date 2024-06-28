use hyper_ast::position::TreePath;
use hyper_ast::store::labels::LabelStore;
use hyper_ast::store::nodes::legion::{HashedNodeRef, NodeIdentifier};
use hyper_ast::types::{
    HyperASTShared, HyperType, LabelStore as _, Labeled, NodeStore, Role, RoleStore, Tree,
    WithRoles,
};
use hyper_ast::{position::TreePathMut, types::TypeStore};
use hyper_ast_tsquery::{Cursor, Node as _, Status, Symbol, TreeCursorStep};
use num::ToPrimitive;
use std::marker::PhantomData;

use crate::types::{TIdN, Type};

pub type TreeCursor<'hast, 'acc, HAST> = Node<'hast, 'acc, HAST>;

pub struct Node<'hast, 'acc, HAST: HyperASTShared> {
    pub stores: &'acc HAST,
    acc: &'acc super::Acc,
    label: &'acc Option<String>,
    offset: Idx,
    pub pos: hyper_ast::position::StructuralPosition<HAST::IdN, HAST::Idx>,
    _p: PhantomData<&'hast ()>,
}

impl<'hast, 'acc, TS> PartialEq for Node<'hast, 'acc, HAST<'hast, 'acc, TS>> {
    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos
    }
}

type IdN = NodeIdentifier;
type Idx = u16;
type T<'a> = HashedNodeRef<'a, TIdN<NodeIdentifier>>;

impl<'hast, 'acc, TS> Node<'hast, 'acc, HAST<'hast, 'acc, TS>> {
    pub fn new(
        stores: &'acc HAST<'hast, 'acc, TS>,
        acc: &'acc super::Acc,
        label: &'acc Option<String>,
        pos: hyper_ast::position::StructuralPosition<IdN, Idx>,
    ) -> Self {
        Self {
            stores,
            acc,
            label,
            offset: 0,
            pos,
            _p: PhantomData,
        }
    }
}

impl<'hast, 'acc, TS> Clone for Node<'hast, 'acc, HAST<'hast, 'acc, TS>> {
    fn clone(&self) -> Self {
        Self {
            stores: self.stores,
            acc: self.acc,
            label: self.label,
            offset: self.offset,
            pos: self.pos.clone(),
            _p: PhantomData,
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

type HAST<'hast, 'acc, TS> = super::SimpleStores<&'acc TS, &'hast legion::World, &'acc LabelStore>;

impl<'hast, 'acc, TS> hyper_ast_tsquery::Cursor
    for self::TreeCursor<'hast, 'acc, HAST<'hast, 'acc, TS>>
where
    TS: super::JavaEnabledTypeStore<T<'hast>, Ty = Type>
        + hyper_ast::types::RoleStore<T<'hast>, IdF = IdF, Role = Role>,
{
    type Node = self::Node<'hast, 'acc, HAST<'hast, 'acc, TS>>;
    type NodeRef<'a> = &'a self::Node<'hast, 'acc, HAST<'hast, 'acc, TS>> where Self: 'a;

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
            let n = self.stores.node_store.resolve(p);
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
                        if *o as usize + 1 < self.acc.simple.children.len() {
                            self.offset = o + 1;
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
            let Some(node) = self.acc.simple.children.get(o.to_usize().unwrap()) else {
                return TreeCursorStep::TreeCursorStepNone;
            };
            self.pos.inc(*node);
        } else {
            //dbg!();
            self.offset += 1;
            let o = self.offset;
            let Some(node) = self.acc.simple.children.get(o.to_usize().unwrap()) else {
                return TreeCursorStep::TreeCursorStepNone;
            }; //dbg!(node);
            self.pos.goto(*node, o);
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
            let n = self.stores.node_store.resolve(n);
            use hyper_ast::types::Children;
            use hyper_ast::types::WithChildren;
            let Some(node) = n.children().and_then(|x| x.get(0u16)) else {
                return TreeCursorStep::TreeCursorStepNone;
            };
            self.pos.goto(*node, num::zero());
        } else if let Some(o) = self.pos.offset() {
            // dbg!();
            let Some(node) = self.acc.simple.children.get(o.to_usize().unwrap()) else {
                return TreeCursorStep::TreeCursorStepNone;
            };
            self.pos.inc(*node);
        } else {
            // dbg!();
            let Some(node) = self.acc.simple.children.get(self.offset as usize) else {
                return TreeCursorStep::TreeCursorStepNone;
            };
            self.pos.goto(*node, self.offset);
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
                self.offset = *o + 1;
                // let o = self.pos.offset().unwrap();
                let Some(_) = self.acc.simple.children.get(o.to_usize().unwrap() + 1) else {
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
                use hyper_ast_tsquery::Node;
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

    fn text_provider(&self) -> <Self::Node as hyper_ast_tsquery::Node>::TP<'_> {
        ()
    }

    fn is_visible_at_root(&self) -> bool {
        assert!(self.pos.node().is_none());
        self.is_visible()
    }
}

impl<'hast, 'acc, TS> self::TreeCursor<'hast, 'acc, HAST<'hast, 'acc, TS>>
where
    TS: super::JavaEnabledTypeStore<T<'hast>, Ty = Type>
        + hyper_ast::types::RoleStore<T<'hast>, IdF = IdF, Role = Role>,
{
    fn role(&self) -> Option<Role> {
        if let Some(p) = self.pos.parent() {
            let n = self.stores.node_store.resolve(p);
            n.role_at::<Role>(self.pos.o().unwrap())
        } else {
            let at = self.pos.o().unwrap();
            let ro = &self.acc.role_offsets;
            let r = &self.acc.roles;
            let mut i = 0;
            for &ro in ro {
                if ro as u16 > at {
                    return None;
                } else if ro as u16 == at {
                    return Some(r[i]);
                }
                i += 1;
            }
            None
        }
    }

    fn super_types(mut self) -> Vec<Symbol> {
        // TODO Might create efficiency issues, is it compiling well ?
        let mut result = vec![];
        loop {
            use hyper_ast_tsquery::Node;
            if self.pos.pop().is_none() {
                return result;
            }
            let Some((_, o)) = &self.pos.pop() else {
                return result;
            };
            if self.pos.node().is_none() {
                // at root of subtree
                self.offset = *o + 1;
                let Some(_) = self.acc.simple.children.get(o.to_usize().unwrap() + 1) else {
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
            lang = self.acc.simple.kind.get_lang();
            // self.acc.role
            None // actually should not provide role as it is not part of identifying data
        } else if self.pos.parent().is_none() {
            lang = self.acc.simple.kind.get_lang();
            let o = self.pos.o().unwrap();
            self.acc
                .role_offsets
                .iter()
                .position(|x| *x as u16 == o)
                .and_then(|x| self.acc.roles.get(x))
                .cloned()
        } else {
            let mut p = self.clone();
            loop {
                let Some((_, o)) = p.pos.pop() else {
                    return (None, Default::default());
                };
                let Some(n) = p.pos.node() else {
                    return (None, Default::default());
                };
                let n = self.stores.node_store.resolve(n);
                // dbg!(p.kind());
                if p.kind().is_supertype() {
                    continue;
                }
                lang = p.kind().get_lang();
                use num::One;
                break n.role_at::<Role>(o - Idx::one());
            }
        };
        let field_id = if let Some(role) = role {
            RoleStore::<HashedNodeRef<TIdN<_>>>::intern_role(self.stores.type_store, lang, role)
        } else {
            Default::default()
        };
        (role, field_id)
    }
}

type IdF = u16;

impl<'hast, 'acc, TS> hyper_ast_tsquery::Node for self::Node<'hast, 'acc, HAST<'hast, 'acc, TS>>
where
    TS: super::JavaEnabledTypeStore<T<'hast>, Ty = Type>
        + hyper_ast::types::RoleStore<T<'hast>, IdF = IdF, Role = Role>,
{
    fn symbol(&self) -> Symbol {
        // TODO make something more efficient
        let id = TypeStore::<T>::type_to_u16(self.stores.type_store, self.kind());
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
        let role =
            RoleStore::<T>::resolve_field(self.stores.type_store, self.kind().get_lang(), field_id);
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
            let n = self.stores.node_store.resolve(id);
            if n.has_children() {
                // dbg!();
                return "".into();
                // let r = hyper_ast::nodes::TextSerializer::new(self.stores, *id).to_string();
                // return r.into();
            }
            if let Some(l) = n.try_get_label() {
                let l = self.stores.label_store.resolve(l);
                return l.into();
            }
            "".into()
        } else if !self.acc.simple.children.is_empty() {
            todo!()
        } else if let Some(label) = self.label {
            label.into()
        } else {
            "".into()
        }
    }
}

impl<'hast, 'acc, TS> Node<'hast, 'acc, HAST<'hast, 'acc, TS>>
where
    TS: super::JavaEnabledTypeStore<T<'hast>, Ty = Type>
        + hyper_ast::types::RoleStore<T<'hast>, IdF = IdF, Role = Role>,
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

impl<'hast, 'acc, TS: super::JavaEnabledTypeStore<T<'hast>, Ty = Type>>
    Node<'hast, 'acc, HAST<'hast, 'acc, TS>>
where
//     HAST::IdN: std::fmt::Debug + Copy,
//     HAST::TS: TypeStore<HashedNodeRef<'hast, TIdN<NodeIdentifier>>, Ty = crate::types::Type>,
//     HAST: HyperAST<
//         'hast,
//         IdN = NodeIdentifier,
//         // TS = crate::types::TStore,
//         T = HashedNodeRef<'hast, TIdN<NodeIdentifier>>,
//     >,
{
    fn kind(&self) -> crate::types::Type {
        if let Some(n) = self.pos.node() {
            self.resolve_type(n)
        } else {
            self.acc.simple.kind
        }
    }

    fn resolve_type(&self, n: &legion::Entity) -> Type {
        let node =
            hyper_ast::store::nodes::legion::_resolve::<TIdN<IdN>>(self.stores.node_store, n)
                .unwrap();
        self.stores.type_store.resolve_type(&node)
    }
}

impl<'hast, 'acc, TS: super::JavaEnabledTypeStore<T<'hast>, Ty = Type>>
    Node<'hast, 'acc, HAST<'hast, 'acc, TS>>
where
// HAST::IdN: std::fmt::Debug + Copy,
// HAST::TS: TypeStore<HashedNodeRef<'hast, TIdN<NodeIdentifier>>, Ty = crate::types::Type>,
// HAST: HyperAST<
//     'hast,
//     IdN = NodeIdentifier,
//     // TS = crate::types::TStore,
//     T = HashedNodeRef<'hast, TIdN<NodeIdentifier>>,
// >,
{
    fn is_visible(&self) -> bool {
        !self.kind().is_hidden()
    }
}
