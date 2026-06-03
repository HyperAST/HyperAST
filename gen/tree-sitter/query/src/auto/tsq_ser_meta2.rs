use std::fmt::Debug;
use std::fmt::Display;

use hyperast::position::structural_pos::BBB;
use hyperast::types;
use hyperast::types::Childrn as _;
use hyperast::types::HyperAST;
use hyperast::types::HyperType as _;
use hyperast::types::WithPrecompQueries;
use hyperast::types::WithRoles;

pub struct TreeToQuery<
    'hast,
    HAST: HyperAST,
    TIdN: hyperast::types::TypedNodeId,
    // vanilla tsq syntax
    const V: bool = false,
    // pretty print
    const PP: bool = true,
> {
    stores: &'hast HAST,
    root: HAST::IdN,
    meta: hyperast_tsquery::Query,
    phantom: std::marker::PhantomData<TIdN>,
}
impl<'store, 'a, HAST: types::TypedHyperAST<TIdN>, TIdN: hyperast::types::TypedNodeId>
    TreeToQuery<'store, HAST, TIdN>
{
    pub fn new(
        stores: &'store HAST,
        root: HAST::IdN,
        meta: hyperast_tsquery::Query,
    ) -> TreeToQuery<'store, HAST, TIdN> {
        Self {
            stores,
            root,
            meta,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<
    'hast,
    HAST: types::TypedHyperAST<TIdN>,
    TIdN: hyperast::types::TypedNodeId + 'static,
    const V: bool,
    const PP: bool,
> Display for TreeToQuery<'hast, HAST, TIdN, V, PP>
where
    HAST::IdN: Debug + Copy,
    HAST::TS: hyperast::types::RoleStore,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithRoles,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithPrecompQueries,
    <HAST::TS as hyperast::types::RoleStore>::IdF: Into<u16> + From<u16>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let pos = hyperast::position::structural_pos::CursorWithPersistance::new(self.root);
        let mut cursor = hyperast_tsquery::hyperast_opt::TreeCursor::new(self.stores, pos);
        self.serialize(&mut cursor, &mut 0, 0, f).map(|_| ())
    }
}

impl<
    'hast,
    HAST: types::TypedHyperAST<TIdN>,
    TIdN: hyperast::types::TypedNodeId + 'static,
    const V: bool,
    const PP: bool,
> TreeToQuery<'hast, HAST, TIdN, V, PP>
where
    HAST::IdN: Debug + Copy,
    HAST::TS: hyperast::types::RoleStore,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithRoles,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithPrecompQueries,
    <HAST::TS as hyperast::types::RoleStore>::IdF: Into<u16> + From<u16>,
{
    fn serialize(
        &self,
        cursor: &mut hyperast_tsquery::hyperast_opt::TreeCursor<HAST>,
        count: &mut usize,
        ind: usize,
        out: &mut std::fmt::Formatter<'_>,
    ) -> Result<(), std::fmt::Error> {
        use hyperast::position::structural_pos::AAA;
        use types::{LabelStore, Labeled, NodeStore, WithChildren};
        let id = &cursor.pos.node();
        let b = self.stores.node_store().resolve(&cursor.pos.node());
        // let kind = (self.stores.type_store(), b);
        let kind = self.stores.resolve_type(&cursor.pos.node());
        let label = b.try_get_label();
        let children = b.children();

        if kind.is_spaces() {
            return Ok(());
        }

        debug_assert_eq!(*id, cursor.pos.node());
        if self.should_skip(id) {
            return Ok(());
        }

        let kind2 = if self.should_abstract(cursor) {
            let p = cursor.pos.parent().unwrap();
            Some(self.stores.resolve_type(&p))
        } else {
            None
        };

        match (label, children) {
            (None, None) => {
                if self.should_ignore(id) {
                    return Ok(());
                }
                write!(out, "\"{}\"", kind)?;
            }
            (_, Some(children)) => {
                if kind.is_hidden() {
                    let it = children.iter_children();
                    let mut f = false;
                    let mut first = true;
                    for id in it {
                        if first {
                            cursor.pos.down(id, num::zero());
                            first = false;
                        } else {
                            cursor.pos.inc(id);
                        }
                        if self.should_skip(&id) {
                            assert_eq!(id, cursor.pos.node());
                            continue;
                        }
                        let kind = self.stores.resolve_type(&id);
                        if !kind.is_spaces() && !kind.is_hidden() {
                            if PP {
                                if f {
                                    write!(out, "\n{}", "  ".repeat(ind))?;
                                } else {
                                    f = true;
                                }
                            } else {
                                write!(out, " ")?;
                            }
                        }
                        self.serialize(cursor, count, ind, out)?;
                    }
                    cursor.pos.up();
                    debug_assert_eq!(*id, cursor.pos.node());
                } else if self.should_ignore(id) && !children.is_empty() {
                    let it = children.iter_children();

                    let mut f = false;
                    let mut first = true;
                    for id in it {
                        if first {
                            cursor.pos.down(id, num::zero());
                            first = false;
                        } else {
                            cursor.pos.inc(id);
                        }
                        if self.should_skip(&id) {
                            debug_assert_eq!(id, cursor.pos.node());
                            continue;
                        }
                        let kind = self.stores.resolve_type(&id);
                        if !kind.is_spaces() && !kind.is_hidden() {
                            if PP {
                                if f {
                                    write!(out, "\n{}", "  ".repeat(ind))?;
                                } else {
                                    f = true;
                                }
                            } else {
                                write!(out, " ")?;
                            }
                        }
                        self.serialize(cursor, count, ind, out)?;
                        debug_assert_eq!(id, cursor.pos.node());
                    }
                    cursor.pos.up();
                    debug_assert_eq!(*id, cursor.pos.node());
                } else if !children.is_empty() {
                    let it = children.iter_children();
                    write!(out, "(")?;
                    if let Some(kind2) = kind2 {
                        write!(out, "{kind2}/{kind}")?;
                    } else {
                        write!(out, "{kind}")?;
                    }
                    let mut first = true;
                    for id in it {
                        if first {
                            cursor.pos.down(id, num::zero());
                            first = false;
                        } else {
                            cursor.pos.inc(id);
                        }
                        if self.should_skip(&id) {
                            debug_assert_eq!(id, cursor.pos.node());
                            continue;
                        }

                        let kind = self.stores.resolve_type(&id);
                        if !kind.is_spaces() {
                            if PP {
                                write!(out, "\n{}", "  ".repeat(ind + 1))?;
                            } else {
                                write!(out, " ")?;
                            }
                        }
                        self.serialize(cursor, count, ind + 1, out)?;
                        debug_assert_eq!(id, cursor.pos.node());
                    }
                    if PP {
                        write!(out, "\n{}", "  ".repeat(ind))?;
                    }
                    write!(out, ")")?;
                    cursor.pos.up();
                    debug_assert_eq!(*id, cursor.pos.node());
                }
            }
            (Some(label), None) => {
                if self.should_ignore(id) {
                    return Ok(());
                }
                write!(out, "(")?;
                if let Some(kind2) = kind2 {
                    write!(out, "{kind2}/{kind}")?;
                } else {
                    write!(out, "{kind}")?;
                }
                write!(out, ")")?;

                if self.should_pred_label(id) {
                    let s = self.stores.label_store().resolve(label);
                    if V {
                        write!(out, " @id{} (#eq? @id{} \"{}\")", count, count, s)?;
                        *count += 1;
                    } else {
                        write!(out, " (#EQ? \"{}\")", s)?;
                        *count += 1;
                    }
                }
            }
        }
        Ok(())
    }

    fn should_pred_label(&self, id: &HAST::IdN) -> bool {
        let pos = hyperast::position::structural_pos::CursorWithPersistance::new(*id);
        let cursor = hyperast_tsquery::hyperast_opt::TreeCursor::new(self.stores, pos);
        let mut matches = self.meta.matches_immediate(cursor);
        let Some(m) = matches.next_match() else {
            return false;
        };
        if self.meta.capture_count() == 0 {
            return true;
        }
        let Some(cid) = self.meta.capture_index_for_name("label") else {
            return false;
        };
        if let Some(_) = m.nodes_for_capture_index(cid).next() {
            return true;
        }
        false
    }

    fn should_ignore(&self, id: &HAST::IdN) -> bool {
        let pos = hyperast::position::structural_pos::CursorWithPersistance::new(*id);
        let cursor = hyperast_tsquery::hyperast_opt::TreeCursor::new(self.stores, pos);
        let mut matches = self.meta.matches_immediate(cursor);
        let Some(m) = matches.next_match() else {
            return false;
        };
        let Some(cid) = self.meta.capture_index_for_name("ignore") else {
            return false;
        };
        if let Some(_) = m.nodes_for_capture_index(cid).next() {
            return true;
        }
        false
    }

    fn should_skip(&self, id: &HAST::IdN) -> bool {
        let pos = hyperast::position::structural_pos::CursorWithPersistance::new(*id);
        let cursor = hyperast_tsquery::hyperast_opt::TreeCursor::new(self.stores, pos);
        let mut matches = self.meta.matches_immediate(cursor);
        let Some(cid) = self.meta.capture_index_for_name("skip") else {
            return false;
        };
        while let Some(m) = matches.next_match() {
            if let Some(_) = m.nodes_for_capture_index(cid).next() {
                return true;
            }
        }
        false
    }

    fn should_abstract(&self, c: &mut hyperast_tsquery::hyperast_opt::TreeCursor<HAST>) -> bool {
        let Some(cid) = self.meta.capture_index_for_name("abstract") else {
            return false;
        };
        use hyperast::position::structural_pos::AAA;
        let mut pos = hyperast::position::structural_pos::CursorWithPersistance::new(c.pos.node());
        std::mem::swap(&mut pos, &mut c.pos);
        let cursor = hyperast_tsquery::hyperast_opt::TreeCursor::new(self.stores, pos);
        let mut matches = self.meta.matches_immediate(cursor);
        let r = loop {
            let Some(m) = matches.next_match() else {
                break false;
            };
            if let Some(_) = m.nodes_for_capture_index(cid).next() {
                break true;
            }
        };
        assert_eq!(matches.cursor.pos.node(), c.pos.node());
        std::mem::swap(&mut matches.cursor.pos, &mut c.pos);
        r
    }
}
