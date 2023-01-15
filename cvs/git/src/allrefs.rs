use std::{borrow::Borrow, fmt::Display, io::Write, path::Path, time::Instant};

use hyper_ast::{
    position::{Position, Scout, SpHandle, StructuralPosition, StructuralPositionStore, TreePath},
    store::{
        defaults::{LabelIdentifier, NodeIdentifier},
        nodes::legion::HashedNodeRef,
    },
    types::{IterableChildren, LabelStore, Labeled, Type, Typed, WithChildren},
};
use hyper_ast_gen_ts_java::{
    impact::{
        element::{IdentifierFormat, LabelPtr, RefPtr, RefsEnum},
        partial_analysis::PartialAnalysis,
        reference::DisplayRef,
        usage::{self, remake_pkg_ref},
    },
    usage::declarations::IterDeclarations,
};

use crate::{maven::IterMavenModules, preprocessed::child_by_name_with_idx, SimpleStores};

const REFERENCES_SERIALIZATION_SUMMARY: bool = false;

/// TODO before enabling, make sure the recusive reference search works, it is often needed for members eg. chained calls
const SEARCH_MEMBERS: bool = false;

/// Write in [`out`], the JSON formated reprentation of the reference relations at [`root`] in [`prepro`].
pub fn write_referencial_relations<W: Write>(
    stores: &SimpleStores,
    root: NodeIdentifier,
    out: &mut W,
) {
    let modules = IterMavenModules::new(stores, StructuralPosition::new(root), root);
    // let declarations = iter_declarations(stores, modules);

    let mut first = true;

    for module in modules {
        if first {
            first = false;
        } else {
            writeln!(out, ",").unwrap();
        }
        write!(out, r#"{{"module":"#).unwrap();
        write!(
            out,
            "\"{}\"",
            module.make_position(stores).file().to_str().unwrap()
        )
        .unwrap();
        writeln!(out, r#","content": ["#).unwrap();
        let mut writer = Writer::new(out);
        let declarations = iter_declarations(stores, module);
        for (decl, root_folder, of) in declarations {
            let now = Instant::now();
            let references =
                find_declaration_references_position(root, stores, &decl, root_folder, of);
            if let Some((sk, references)) = references {
                let decl = decl.make_position(stores);
                let time = now.elapsed().as_nanos();
                log::info!("time taken for refs search of {} :\t{}", decl, time);
                if REFERENCES_SERIALIZATION_SUMMARY {
                    writer.summary_of_referencial_relations(decl, sk, time, references);
                } else {
                    writer.positions_of_referencial_relations(decl, sk, time, references);
                }
            }
        }
        write!(out, "]}}").unwrap();
    }
}

pub enum SearchKinds {
    TypeDecl,
    LocalDecl,
}

impl Display for SearchKinds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            SearchKinds::TypeDecl => write!(f, "type"),
            SearchKinds::LocalDecl => write!(f, "local"),
        }
    }
}

pub fn find_declaration_references_position(
    root: NodeIdentifier,
    stores: &SimpleStores,
    declaration: &StructuralPosition,
    root_folder: StructuralPosition,
    other_folders: Vec<StructuralPosition>,
) -> Option<(SearchKinds, Vec<Position>)> {
    let mut structural_positions = StructuralPosition::new(root).into();
    let (rk, references) = find_declaration_references(
        stores,
        &mut structural_positions,
        declaration,
        root_folder,
        other_folders,
    )?;
    let references = structural_positions.ends_positions(stores, &references);
    Some((rk, references))
}

fn find_declaration_references(
    stores: &SimpleStores,
    structural_positions: &mut StructuralPositionStore,
    declaration: &StructuralPosition,
    root_folder: StructuralPosition,
    other_folders: Vec<StructuralPosition>,
) -> Option<(SearchKinds, Vec<SpHandle>)> {
    let b = stores
        .node_store
        .resolve(declaration.node().unwrap().to_owned());
    let t = b.get_type();
    let of = other_folders
        .iter()
        .map(|x| (x.make_position(stores).file().to_owned(), x.clone()));
    let p_in_of = find_package_in_other_folders(
        stores,
        declaration.make_position(stores).file(),
        declaration.make_position(stores).file(),
        of,
    );
    let p_in_of: Vec<Scout> = p_in_of.into_iter().map(|x| (x, 0).into()).collect();
    let other_folders: Vec<Scout> = other_folders.into_iter().map(|x| (x, 0).into()).collect();

    if t == Type::ClassDeclaration
        || t == Type::InterfaceDeclaration
        || t == Type::AnnotationTypeDeclaration
    {
        let rs = RefsFinder::new(stores, structural_positions)
            .find_type_declaration_references_unchecked(
                (declaration.clone(), 0).into(),
                root_folder.node().unwrap(),
                &other_folders,
                &p_in_of,
            );
        Some((SearchKinds::TypeDecl, rs))
    } else if SEARCH_MEMBERS && t == Type::FieldDeclaration
    // || t == Type::ConstantDeclaration
    {
        let rs = RefsFinder::new(stores, structural_positions)
            .find_field_declaration_references_unchecked(
                (declaration.clone(), 0).into(),
                root_folder.node().unwrap(),
                &p_in_of,
            );
        Some((SearchKinds::TypeDecl, rs))
    } else if t == Type::ClassBody {
        let rs = RefsFinder::new(stores, structural_positions)
            .find_this_unchecked((declaration.clone(), 0).into());
        Some((SearchKinds::LocalDecl, rs))
    } else if t == Type::LocalVariableDeclaration
        || t == Type::Resource
        || t == Type::EnhancedForVariable
        || t == Type::CatchFormalParameter
        || t == Type::FormalParameter
        || t == Type::SpreadParameter
        || t == Type::Identifier
        || t == Type::TypeParameter
    {
        let rs = RefsFinder::new(stores, structural_positions)
            .find_localvar_declaration_references_unchecked((declaration.clone(), 0).into());
        Some((SearchKinds::LocalDecl, rs))
    } else {
        None
    }
}

struct Writer<'a, W: Write> {
    start: bool,
    out: &'a mut W,
}

impl<'a, W: Write> Writer<'a, W> {
    pub fn new(out: &'a mut W) -> Self {
        Self { start: true, out }
    }
    pub fn positions_of_referencial_relations(
        &mut self,
        decl: Position,
        search_kind: SearchKinds,
        time: u128,
        references: Vec<Position>,
    ) {
        let out = &mut self.out;
        if self.start {
            self.start = false;
        } else {
            writeln!(out, ",").unwrap();
        }
        write!(out, r#"{{"decl":"#).unwrap();
        write!(out, "{}", decl).unwrap();
        write!(out, r#","search":"#).unwrap();
        write!(out, "\"{}\"", search_kind).unwrap();
        write!(out, r#","duration":"#).unwrap();
        write!(out, "{}", time).unwrap();
        write!(out, r#","refs":["#).unwrap();
        let mut first = true;
        for x in references {
            if first {
                first = false;
            } else {
                write!(out, ",").unwrap();
            }
            write!(out, "{}", x).unwrap();
        }
        write!(out, "]}}").unwrap();
    }

    pub fn summary_of_referencial_relations(
        &mut self,
        _decl: Position,
        search_kind: SearchKinds,
        time: u128,
        references: Vec<Position>,
    ) {
        let out = &mut self.out;
        write!(out, r#"{{"search":"#).unwrap();
        write!(out, "\"{}\"", search_kind).unwrap();
        write!(out, r#","duration":"#).unwrap();
        write!(out, "{}", time).unwrap();
        write!(out, r#","refs":"#).unwrap();
        write!(out, "{}", references.len()).unwrap();
        write!(out, "}}").unwrap();
        writeln!(out, ",").unwrap();
    }
}

type ExtendedDeclaration = (
    StructuralPosition,
    StructuralPosition,
    Vec<StructuralPosition>,
);

pub fn modules_iter_declarations<'a>(
    stores: &'a SimpleStores,
    modules: IterMavenModules<'a, StructuralPosition>,
) -> impl Iterator<Item = ExtendedDeclaration> + 'a {
    modules
        .flat_map(|maven_module| maven_module_folders(stores, maven_module))
        .flat_map(|(f, _m, of)| make_decl_iter(stores, f, of))
}

pub fn iter_declarations<'a>(
    stores: &'a SimpleStores,
    maven_module: StructuralPosition,
) -> impl Iterator<Item = ExtendedDeclaration> + 'a {
    maven_module_folders(stores, maven_module)
        .into_iter()
        .flat_map(|(f, _m, of)| make_decl_iter(stores, f, of))
}

fn make_decl_iter(
    stores: &SimpleStores,
    f: StructuralPosition,
    of: Vec<StructuralPosition>,
) -> impl Iterator<Item = ExtendedDeclaration> + '_ {
    let n = *f.node().unwrap();
    IterDeclarations::new(stores, f.clone(), n).map(move |x| (x, f.clone(), of.clone()))
}

fn maven_module_folders(
    stores: &SimpleStores,
    maven_module: StructuralPosition,
) -> Vec<(
    StructuralPosition,
    StructuralPosition,
    Vec<StructuralPosition>,
)> {
    let src = goto_by_name(stores, maven_module.clone(), "src");
    let source_tests = src
        .clone()
        .and_then(|x| goto_by_name(stores, x, "test"))
        .and_then(|x| goto_by_name(stores, x, "java"));
    let source = src
        .and_then(|x| goto_by_name(stores, x, "main"))
        .and_then(|x| goto_by_name(stores, x, "java"));
    let mut r = vec![];
    let mut test_folders = vec![];
    if let Some(source_tests) = source_tests {
        test_folders.push(source_tests.clone());
        r.push((source_tests, maven_module.clone(), vec![]))
    }
    if let Some(source) = source {
        r.push((source, maven_module, test_folders))
    }
    r
}

pub struct RefsFinder<'a> {
    stores: &'a SimpleStores,
    ana: PartialAnalysis,
    structural_positions: &'a mut StructuralPositionStore,
}

struct Cursor {
    scout: Scout,
    prev: Option<NodeIdentifier>,
    curr: Option<NodeIdentifier>,
}

#[derive(Debug)]
enum SearchStopEvent {
    NoMore,
    Blocked,
}

impl<'a> RefsFinder<'a> {
    pub fn new(
        stores: &'a SimpleStores,
        structural_positions: &'a mut StructuralPositionStore,
    ) -> Self {
        Self {
            stores: stores,
            ana: PartialAnalysis::default(),
            structural_positions,
        }
    }

    fn find_type_declaration_references_unchecked(
        self,
        decl: Scout,
        limit: &NodeIdentifier,
        other_folders: &[Scout],
        mirror_packages: &[Scout],
    ) -> Vec<SpHandle> {
        let mut r = vec![];
        // let p = decl.make_position(&self.structural_positions, &self.stores);
        let res = self.find_type_declaration_references(
            &mut r,
            &decl,
            limit,
            other_folders,
            mirror_packages,
        );
        if let Err(SearchStopEvent::NoMore) = res {
            log::error!("search stoped early");
            // log::error!("search of {} ended with {:?}", p, err);
        }
        r
    }

    fn find_type_declaration_references(
        mut self,
        r: &mut Vec<SpHandle>,
        decl: &Scout,
        limit: &NodeIdentifier,
        other_folders: &[Scout],
        mirror_packages: &[Scout],
    ) -> Result<(), SearchStopEvent> {
        let mut scout = decl.clone();
        self.structural_positions.push(&mut scout); // memory footprint ?
        let qual_ref = self.init_type_decl(r, &decl);

        let prev_offset = scout.offset_always(&self.structural_positions) - 1;
        let prev = Some(scout.node_always(&self.structural_positions));
        let curr = scout.up(&self.structural_positions);

        let mut cursor = Cursor { scout, prev, curr };

        log::trace!("go_through_type_declarations");
        let qual_ref = self.go_through_type_declarations(r, prev_offset, &mut cursor, qual_ref)?;
        log::trace!("go_through_program");
        let (package_ref, fq_decl_ref) = self.go_through_program(r, &mut cursor, qual_ref)?;
        {
            let mut scout = decl.clone();
            let prev = Some(scout.node_always(&self.structural_positions));
            let curr = scout.up(&self.structural_positions);

            let mut cursor = Cursor { scout, prev, curr };
            log::trace!("go_through_type_declarations_with_fully_qual_ref");
            self.go_through_type_declarations_with_fully_qual_ref(r, &mut cursor, fq_decl_ref)
                .unwrap_or(());
        }
        log::trace!("go_through_package");
        self.go_through_package(r, &mut cursor, mirror_packages, &package_ref, &fq_decl_ref)?;
        log::trace!("go_through_directories");
        self.go_through_directories(r, &mut cursor, &package_ref, &fq_decl_ref, limit)?;
        log::trace!("go_through_folders");
        self.go_through_folders(r, &mut cursor, &package_ref, &fq_decl_ref, other_folders)?;

        Ok(())
    }

    fn find_this_unchecked(mut self, decl: Scout) -> Vec<SpHandle> {
        let mut r = vec![];
        self.find_this(&mut r, &decl);
        r
    }

    fn find_field_declaration_references_unchecked(
        self,
        decl: Scout,
        limit: &NodeIdentifier,
        mirror_packages: &[Scout],
    ) -> Vec<SpHandle> {
        let mut r = vec![];
        let p = decl.make_position(&self.structural_positions, &self.stores);
        let res = self.find_field_declaration_references(&mut r, &decl, limit, mirror_packages);
        if let Err(err) = res {
            log::error!("search of {} ended with {:?}", p, err);
        }
        r
    }

    fn find_field_declaration_references(
        mut self,
        r: &mut Vec<SpHandle>,
        decl: &Scout,
        limit: &NodeIdentifier,
        mirror_packages: &[Scout],
    ) -> Result<(), SearchStopEvent> {
        let mut scout = decl.clone();
        self.structural_positions.push(&mut scout); // memory footprint ?
        let qual_ref = self.init_field_decl(r, &decl);

        let prev = Some(scout.node_always(&self.structural_positions));
        let curr = scout.up(&self.structural_positions);

        let mut cursor = Cursor { scout, prev, curr };

        let qual_ref = self.go_through_type_declarations_for_field(r, &mut cursor, qual_ref)?;
        // TODO make sure it does not need special handling
        let (package_ref, fq_decl_ref) = self.go_through_program(r, &mut cursor, qual_ref)?;
        self.go_through_package(r, &mut cursor, mirror_packages, &package_ref, &fq_decl_ref)?;
        self.go_through_directories(r, &mut cursor, &package_ref, &fq_decl_ref, limit)?;

        Ok(())
    }

    fn init_field_decl<'b>(&mut self, _r: &mut Vec<SpHandle>, decl: &Scout) -> RefPtr {
        let b = self
            .stores
            .node_store
            .resolve(decl.node_always(&self.structural_positions));
        let t = b.get_type();
        log::info!(
            "now search for {:?} at {:?}",
            &t,
            decl.make_position(&self.structural_positions, &self.stores)
        );
        let name = {
            let mut i = None;
            for xx in b.children().unwrap().iter_children() {
                let bb = self.stores.node_store.resolve(*xx);
                if bb.get_type() == Type::VariableDeclarator {
                    i = self.extract_identifier(&bb);
                    break;
                }
            }
            let i = i.unwrap();
            let name = self.stores.label_store.resolve(&i);
            log::info!("search uses of {:?}", name);
            LabelPtr::new(i, IdentifierFormat::from(name))
        };

        let mm = self.ana.solver.intern(RefsEnum::MaybeMissing);
        self.ana.solver.intern(RefsEnum::ScopedIdentifier(mm, name))
    }

    fn find_localvar_declaration_references_unchecked(self, decl: Scout) -> Vec<SpHandle> {
        let mut r = vec![];
        let p = decl.make_position(&self.structural_positions, &self.stores);
        let res = self.find_localvar_declaration_references(&mut r, &decl);
        if let Err(err) = res {
            log::error!("search of {} ended with {:?}", p, err);
        }
        r
    }

    fn find_localvar_declaration_references(
        mut self,
        r: &mut Vec<SpHandle>,
        decl: &Scout,
    ) -> Result<(), SearchStopEvent> {
        let mut scout = decl.clone();
        self.structural_positions.push(&mut scout); // memory footprint ?
        let qual_ref = self.init_localvar_decl(r, &decl);

        let prev = Some(scout.node_always(&self.structural_positions));
        let curr = scout.up(&self.structural_positions);

        let mut cursor = Cursor { scout, prev, curr };

        self.go_through_block(r, &mut cursor, qual_ref)
    }

    fn init_localvar_decl<'b>(&mut self, _r: &mut Vec<SpHandle>, decl: &Scout) -> RefPtr {
        let b = self
            .stores
            .node_store
            .resolve(decl.node_always(&self.structural_positions));
        let t = b.get_type();
        log::info!(
            "now search for {:?} at {:?}",
            &t,
            decl.make_position(&self.structural_positions, &self.stores)
        );
        let name = {
            let mut i = None;

            if t == Type::Identifier {
                i = Some(*b.get_label());
            } else if t == Type::LocalVariableDeclaration || t == Type::SpreadParameter {
                for xx in b.children().unwrap().iter_children() {
                    let bb = self.stores.node_store.resolve(*xx);
                    if bb.get_type() == Type::VariableDeclarator {
                        i = self.extract_identifier(&bb);
                        break;
                    }
                }
            } else {
                i = self.extract_identifier(&b);
            }
            let i = i.unwrap();
            let name = self.stores.label_store.resolve(&i);
            log::info!("search uses of {:?}", name);
            LabelPtr::new(i, IdentifierFormat::from(name))
        };

        let mm = self.ana.solver.intern(RefsEnum::MaybeMissing);
        self.ana.solver.intern(RefsEnum::ScopedIdentifier(mm, name))
    }

    fn go_through_block(
        &mut self,
        r: &mut Vec<SpHandle>,
        cursor: &mut Cursor,
        qual_ref: RefPtr,
    ) -> Result<(), SearchStopEvent> {
        let mm = self.ana.solver.intern(RefsEnum::MaybeMissing);
        let xx = cursor.curr.ok_or(SearchStopEvent::NoMore)?;
        let bb = self.stores.node_store.resolve(xx);
        let t = bb.get_type();

        if t == Type::FormalParameters || t == Type::InferredParameters {
            self.up(cursor);
            return self.go_through_block(r, cursor, qual_ref);
        }

        for (i, xx) in bb.children().unwrap().iter_children().enumerate() {
            cursor.scout.goto(*xx, i);
            if Some(*xx) != cursor.prev {
                log::debug!(
                    "search {} in block children {:?} {:?}",
                    DisplayRef::from((
                        self.ana.solver.nodes.with(*&qual_ref),
                        &self.stores.label_store
                    )),
                    cursor.curr,
                    cursor
                        .scout
                        .make_position(&self.structural_positions, &self.stores)
                );
                r.extend({
                    let p = &mm;
                    let i = &qual_ref;
                    let s = &cursor.scout;
                    usage::RefsFinder::new(
                        &self.stores,
                        &mut self.ana,
                        &mut self.structural_positions,
                    )
                    .find_all_with::<true>(*p, *i, s.clone())
                });
            }
            cursor.scout.up(&self.structural_positions);
        }

        if t == Type::ResourceSpecification
            || t == Type::EnhancedForStatement
            || t == Type::TypeParameters
        {
            self.up(cursor);
            self.go_through_block(r, cursor, qual_ref)?;
        }

        Ok(())
    }

    fn find_this<'b>(&mut self, r: &mut Vec<SpHandle>, decl: &Scout) {
        let b = self
            .stores
            .node_store
            .resolve(decl.node_always(&self.structural_positions));
        let mm = self.ana.solver.intern(RefsEnum::MaybeMissing);
        let mut scout = decl.clone();
        for (i, xx) in b.children().unwrap().iter_children().enumerate() {
            scout.goto(*xx, i);

            log::debug!("try search this");
            r.extend(
                usage::RefsFinder::new(&self.stores, &mut self.ana, &mut self.structural_positions)
                    .find_all_is_this(mm, scout.clone()),
            );
            scout.up(&self.structural_positions);
        }
    }

    fn init_type_decl<'b>(&mut self, r: &mut Vec<SpHandle>, decl: &Scout) -> RefPtr {
        let b = self
            .stores
            .node_store
            .resolve(decl.node_always(&self.structural_positions));
        let t = b.get_type();
        log::info!(
            "now search for {:?} at {:?}",
            &t,
            decl.make_position(&self.structural_positions, &self.stores)
        );
        let name = {
            let i = self.extract_identifier(&b).unwrap();
            let name = self.stores.label_store.resolve(&i);
            log::info!("search uses of {:?}", name);
            LabelPtr::new(i, IdentifierFormat::from(name))
        };
        let mm = self.ana.solver.intern(RefsEnum::MaybeMissing);
        // let thiss = self.ana.solver.intern(RefsEnum::This(mm));
        let qual_ref = self.ana.solver.intern(RefsEnum::ScopedIdentifier(mm, name));
        let qual_thiss = self.ana.solver.intern(RefsEnum::This(qual_ref));

        let mut scout = decl.clone();
        for (i, xx) in b.children().unwrap().iter_children().enumerate() {
            scout.goto(*xx, i);

            log::debug!("try search this");
            // r.extend(self.search(&mm, &thiss, &decl)); for now use something more explicit
            r.extend(
                usage::RefsFinder::new(&self.stores, &mut self.ana, &mut self.structural_positions)
                    .find_all_is_this(mm, scout.clone()),
            );

            log::debug!(
                "try search {}",
                DisplayRef::from((
                    self.ana.solver.nodes.with(qual_ref),
                    &self.stores.label_store
                ))
            );
            r.extend(self.search(&mm, &qual_ref, &scout));

            log::debug!(
                "try search {}",
                DisplayRef::from((
                    self.ana.solver.nodes.with(qual_thiss),
                    &self.stores.label_store
                ))
            );
            r.extend(self.search(&mm, &qual_thiss, &scout));
            scout.up(&self.structural_positions);
        }
        qual_ref
    }

    fn up(&self, mut cursor: &mut Cursor) {
        cursor.prev = cursor.curr.clone();
        cursor.curr = cursor.scout.up(&self.structural_positions);
    }

    fn go_through_type_declarations_with_fully_qual_ref(
        &mut self,
        r: &mut Vec<SpHandle>,
        cursor: &mut Cursor,
        fq_decl_ref: RefPtr,
    ) -> Result<(), SearchStopEvent> {
        let mm = self.ana.solver.intern(RefsEnum::MaybeMissing);
        // go through classes if inner, stops at blocks and object creation expr
        loop {
            let x = cursor.curr.clone().ok_or(SearchStopEvent::NoMore)?;
            let b = self.stores.node_store.resolve(x);
            let t = b.get_type();
            if t.is_type_body() {
                log::debug!(
                    "try search {}",
                    DisplayRef::from((
                        self.ana.solver.nodes.with(fq_decl_ref),
                        &self.stores.label_store
                    ))
                );
                for (i, xx) in b.children().unwrap().iter_children().enumerate() {
                    cursor.scout.goto(*xx, i);
                    if Some(*xx) != cursor.prev.clone() {
                        // search ?.A or ?.B.A or ...
                        r.extend(self.search(&mm, &fq_decl_ref, &cursor.scout));
                    }
                    cursor.scout.up(&self.structural_positions);
                }
                // TODO do things to find type of field at the same level than type decl
                // should loop through siblings
                self.up(cursor);
                let xx = cursor.curr.clone().ok_or(SearchStopEvent::NoMore)?;
                let bb = self.stores.node_store.resolve(xx);
                let tt = bb.get_type();
                if tt == Type::ObjectCreationExpression {
                    return Err(SearchStopEvent::Blocked);
                } else if tt == Type::EnumBody {
                    for (i, xx) in bb.children().unwrap().iter_children().enumerate() {
                        cursor.scout.goto(*xx, i);
                        if Some(*xx) != cursor.prev.clone() {
                            // search ?.A or ?.B.A or ...
                            r.extend(self.search(&mm, &fq_decl_ref, &cursor.scout));
                        }
                        cursor.scout.up(&self.structural_positions);
                    }
                    // TODO do things to find type of field at the same level than type decl
                    // should loop through siblings
                    self.up(cursor);
                    let xx = cursor.curr.clone().ok_or(SearchStopEvent::NoMore)?;
                    let bb = self.stores.node_store.resolve(xx);
                    let tt = bb.get_type();

                    if !tt.is_type_declaration() {
                        panic!("{:?}", tt);
                    }
                } else if !tt.is_type_declaration() {
                    panic!("{:?}", tt);
                };
                let mm = self.ana.solver.intern(RefsEnum::MaybeMissing);
                log::debug!(
                    "try search {}",
                    DisplayRef::from((
                        self.ana.solver.nodes.with(fq_decl_ref),
                        &self.stores.label_store
                    ))
                );
                r.extend(self.search(&mm, &fq_decl_ref, &cursor.scout));
                // TODO do things to find type of field at the same level than type decl
                // should loop through members searching for parent qualified
                self.up(cursor);
            } else if t == Type::Program {
                return Ok(());
            } else if t == Type::ObjectCreationExpression {
                return Err(SearchStopEvent::Blocked);
            } else if t == Type::Block {
                return Err(SearchStopEvent::Blocked); // TODO check if really done with search
            } else {
                todo!("{:?}", t)
            }
        }
    }
    fn go_through_type_declarations(
        &mut self,
        r: &mut Vec<SpHandle>,
        prev_offset: usize,
        cursor: &mut Cursor,
        mut qual_ref: RefPtr,
    ) -> Result<RefPtr, SearchStopEvent> {
        let mm = self.ana.solver.intern(RefsEnum::MaybeMissing);
        // go through classes if inner, stops at blocks and object creation expr
        loop {
            let x = cursor.curr.clone().ok_or(SearchStopEvent::NoMore)?;
            let b = self.stores.node_store.resolve(x);
            let t = b.get_type();
            if t.is_type_body() {
                log::debug!(
                    "try search {}",
                    DisplayRef::from((
                        self.ana.solver.nodes.with(qual_ref),
                        &self.stores.label_store
                    ))
                );
                for (i, xx) in b.children().unwrap().iter_children().enumerate() {
                    cursor.scout.goto(*xx, i);
                    if Some(*xx) != cursor.prev.clone() {
                        // search ?.A or ?.B.A or ...
                        log::trace!("go_through_type_declarations s1");
                        r.extend(self.search(&mm, &qual_ref, &cursor.scout));
                    }
                    cursor.scout.up(&self.structural_positions);
                }
                // TODO do things to find type of field at the same level than type decl
                // should loop through siblings
                // self.structural_positions.check_with(&self.stores, &cursor.scout).expect("before");
                self.up(cursor);
                // self.structural_positions.check_with(&self.stores, &cursor.scout).expect("after");
                let xx = cursor.curr.clone().ok_or(SearchStopEvent::NoMore)?;
                let bb = self.stores.node_store.resolve(xx);
                let tt = bb.get_type();
                let (_, bb, _) = if tt == Type::ObjectCreationExpression {
                    return Err(SearchStopEvent::Blocked);
                } else if tt == Type::EnumBody {
                    for (i, xx) in bb.children().unwrap().iter_children().enumerate() {
                        cursor.scout.goto(*xx, i);
                        if Some(*xx) != cursor.prev.clone() {
                            // search ?.A or ?.B.A or ...
                            log::trace!("go_through_type_declarations s2");
                            r.extend(self.search(&mm, &qual_ref, &cursor.scout));
                        }
                        cursor.scout.up(&self.structural_positions);
                    }
                    // TODO do things to find type of field at the same level than type decl
                    // should loop through siblings
                    self.up(cursor);
                    let xx = cursor.curr.clone().ok_or(SearchStopEvent::NoMore)?;
                    let bb = self.stores.node_store.resolve(xx);
                    let tt = bb.get_type();

                    if !tt.is_type_declaration() {
                        panic!("{:?}", tt);
                    }
                    (xx, bb, tt)
                } else if !tt.is_type_declaration() {
                    panic!("{:?}", tt);
                } else {
                    (xx, bb, tt)
                };
                let name = {
                    let i = self.extract_identifier(&bb).unwrap();
                    let name = self.stores.label_store.resolve(&i);
                    log::info!("search uses of {:?}", name);
                    LabelPtr::new(i, IdentifierFormat::from(name))
                };
                let mm = self.ana.solver.intern(RefsEnum::MaybeMissing);
                let parent_ref = self.ana.solver.intern(RefsEnum::ScopedIdentifier(mm, name));
                qual_ref = self
                    .ana
                    .solver
                    .try_solve_node_with(qual_ref, parent_ref)
                    .unwrap();
                log::debug!(
                    "try search {}",
                    DisplayRef::from((
                        self.ana.solver.nodes.with(qual_ref),
                        &self.stores.label_store
                    ))
                );
                log::trace!("go_through_type_declarations s3");
                r.extend(self.search(&mm, &qual_ref, &cursor.scout));
                // TODO do things to find type of field at the same level than type decl
                // should loop through members searching for parent qualified
                self.up(cursor);
            } else if t == Type::Program {
                return cursor
                    .curr
                    .and(Some(qual_ref))
                    .ok_or(SearchStopEvent::NoMore);
            } else if t == Type::ObjectCreationExpression {
                return Err(SearchStopEvent::Blocked);
            } else if t == Type::ConstructorBody {
                let mut scout = cursor.scout.clone();
                for (i, xx) in b
                    .children()
                    .unwrap()
                    .iter_children()
                    .skip(prev_offset)
                    .enumerate()
                {
                    scout.goto(*xx, i);
                    log::trace!("go_through_type_declarations s4");
                    r.extend(self.search(&mm, &qual_ref, &cursor.scout));
                    scout.up(&self.structural_positions);
                }
                return Err(SearchStopEvent::Blocked);
            } else if t == Type::Block {
                let mut scout = cursor.scout.clone();
                for (i, xx) in b
                    .children()
                    .unwrap()
                    .iter_children()
                    .skip(prev_offset)
                    .enumerate()
                {
                    scout.goto(*xx, i);
                    log::trace!("go_through_type_declarations s5");
                    r.extend(self.search(&mm, &qual_ref, &cursor.scout));
                    scout.up(&self.structural_positions);
                }
                return Err(SearchStopEvent::Blocked); // TODO check if really done with search
            } else {
                todo!("{:?}", t)
            }
        }
    }
    fn go_through_type_declarations_for_field(
        &mut self,
        r: &mut Vec<SpHandle>,
        cursor: &mut Cursor,
        mut qual_ref: RefPtr,
    ) -> Result<RefPtr, SearchStopEvent> {
        let mm = self.ana.solver.intern(RefsEnum::MaybeMissing);
        let thiss = self.ana.solver.intern(RefsEnum::This(mm));
        let thiss_qual_ref = self
            .ana
            .solver
            .try_solve_node_with(qual_ref, thiss)
            .unwrap();
        // go through classes if inner, stops at blocks and object creation expr
        loop {
            let x = cursor.curr.clone().ok_or(SearchStopEvent::NoMore)?;
            let b = self.stores.node_store.resolve(x);
            let t = b.get_type();
            if t.is_type_body() {
                log::debug!(
                    "try search {}",
                    DisplayRef::from((
                        self.ana.solver.nodes.with(qual_ref),
                        &self.stores.label_store
                    ))
                );
                for (i, xx) in b.children().unwrap().iter_children().enumerate() {
                    cursor.scout.goto(*xx, i);
                    if Some(*xx) != cursor.prev.clone() {
                        // search ?.A or ?.B.A or ...
                        r.extend(self.search(&mm, &qual_ref, &cursor.scout));
                        r.extend(self.search(&mm, &thiss_qual_ref, &cursor.scout));
                    }
                    cursor.scout.up(&self.structural_positions);
                }
                // TODO do things to find type of field at the same level than type decl
                // should loop through siblings
                self.up(cursor);
                let xx = cursor.curr.clone().ok_or(SearchStopEvent::NoMore)?;
                let bb = self.stores.node_store.resolve(xx);
                let tt = bb.get_type();
                let (bb, tt) = if tt == Type::EnumBody {
                    self.up(cursor);
                    let xx = cursor.curr.clone().ok_or(SearchStopEvent::NoMore)?;
                    let bb = self.stores.node_store.resolve(xx);
                    let tt = bb.get_type();
                    (bb, tt)
                } else {
                    (bb, tt)
                };
                if tt == Type::ObjectCreationExpression {
                    return Err(SearchStopEvent::Blocked);
                } else if !tt.is_type_declaration() {
                    panic!("{:?}", tt);
                }
                let name = {
                    let i = self.extract_identifier(&bb).unwrap();
                    let name = self.stores.label_store.resolve(&i);
                    log::info!("search uses of {:?}", name);
                    LabelPtr::new(i, IdentifierFormat::from(name))
                };
                let mm = self.ana.solver.intern(RefsEnum::MaybeMissing);
                let parent_ref = self.ana.solver.intern(RefsEnum::ScopedIdentifier(mm, name));
                qual_ref = self
                    .ana
                    .solver
                    .try_solve_node_with(qual_ref, parent_ref)
                    .unwrap();
                log::debug!(
                    "try search {}",
                    DisplayRef::from((
                        self.ana.solver.nodes.with(qual_ref),
                        &self.stores.label_store
                    ))
                );
                r.extend(self.search(&mm, &qual_ref, &cursor.scout));
                // TODO do things to find type of field at the same level than type decl
                // should loop through members searching for parent qualified
                self.up(cursor);
            } else if t == Type::Program {
                return cursor
                    .curr
                    .and(Some(qual_ref))
                    .ok_or(SearchStopEvent::NoMore);
            } else if t == Type::ObjectCreationExpression {
                return Err(SearchStopEvent::Blocked);
            } else if t == Type::Block {
                return Err(SearchStopEvent::Blocked); // TODO check if really done with search
            } else {
                todo!("{:?}", t)
            }
        }
    }
    fn go_through_program(
        &mut self,
        r: &mut Vec<SpHandle>,
        cursor: &mut Cursor,
        qual_ref: RefPtr,
    ) -> Result<(RefPtr, RefPtr), SearchStopEvent> {
        let before_p_ref;
        let mut max_qual_ref = qual_ref;
        let mut package_ref = self.ana.solver.intern(RefsEnum::MaybeMissing);
        // go through classes if inner
        let x = cursor.curr.clone().ok_or(SearchStopEvent::NoMore)?;
        let b = self.stores.node_store.resolve(x);
        let t = b.get_type();
        assert_eq!(t, Type::Program);
        log::debug!(
            "go through program {:?} {:?} {}",
            cursor.curr,
            cursor
                .scout
                .make_position(&self.structural_positions, &self.stores),
            b.child_count()
        );
        // go through program i.e. package declaration
        before_p_ref = max_qual_ref;
        for (i, xx) in b.children().unwrap().iter_children().enumerate() {
            cursor.scout.goto(*xx, i);
            let bb = self.stores.node_store.resolve(*xx);
            let tt = bb.get_type();
            log::debug!("in program {:?}", tt);
            if tt == Type::PackageDeclaration {
                package_ref = remake_pkg_ref(&self.stores, &mut self.ana, *xx)
                    .ok_or(SearchStopEvent::Blocked)?;
                max_qual_ref = self
                    .ana
                    .solver
                    .try_solve_node_with(max_qual_ref, package_ref)
                    .unwrap();
                log::info!(
                    "now have fully qual ref {} with package decl {}",
                    DisplayRef::from((
                        self.ana.solver.nodes.with(max_qual_ref),
                        &self.stores.label_store
                    )),
                    DisplayRef::from((
                        self.ana.solver.nodes.with(package_ref),
                        &self.stores.label_store
                    ))
                );
            } else if tt.is_type_declaration() {
                log::debug!(
                    "try search {}",
                    DisplayRef::from((
                        self.ana.solver.nodes.with(max_qual_ref),
                        &self.stores.label_store
                    ))
                );
                if Some(*xx) != cursor.prev.clone() && max_qual_ref != before_p_ref {
                    // search ?.A in file ie. other top level classes
                    r.extend(self.search(&package_ref, &before_p_ref, &cursor.scout));
                }
                // search /.p.A in file
                r.extend(self.search(&package_ref, &max_qual_ref, &cursor.scout));
            }
            cursor.scout.up(&self.structural_positions);
        }
        self.up(cursor);
        cursor
            .curr
            .and(Some((package_ref, max_qual_ref)))
            .ok_or(SearchStopEvent::NoMore)
    }

    fn go_through_package<'b>(
        &mut self,
        r: &mut Vec<SpHandle>,
        cursor: &mut Cursor,
        mirror_packages: &[Scout],
        package_ref: &RefPtr,
        max_qual_ref: &RefPtr,
    ) -> Result<(), SearchStopEvent> {
        let xx = cursor.curr.ok_or(SearchStopEvent::NoMore)?;
        let bb = self.stores.node_store.resolve(xx);
        let t = bb.get_type();
        log::info!(
            "search in package {:?} {:?} {:?}",
            cursor.curr,
            t,
            cursor
                .scout
                .make_position(&self.structural_positions, &self.stores)
        );

        for (i, xx) in bb.children().unwrap().iter_children().enumerate() {
            cursor.scout.goto(*xx, i);
            if Some(*xx) != cursor.prev {
                log::debug!(
                    "search {} in package children {:?} {:?}",
                    DisplayRef::from((
                        self.ana.solver.nodes.with(*max_qual_ref),
                        &self.stores.label_store
                    )),
                    cursor.curr,
                    cursor
                        .scout
                        .make_position(&self.structural_positions, &self.stores)
                );
                r.extend(self.search(package_ref, max_qual_ref, &cursor.scout));
            }
            cursor.scout.up(&self.structural_positions);
        }

        // go through same package but in other folders eg. `src/test/java`
        for pack in mirror_packages {
            let mut pack = pack.clone();
            let bb = self
                .stores
                .node_store
                .resolve(pack.node_always(&self.structural_positions));
            for (i, xx) in bb.children().unwrap().iter_children().enumerate() {
                let bb = self.stores.node_store.resolve(*xx);
                let t = bb.get_type();
                pack.goto(*xx, i);
                if t == Type::Program {
                    log::debug!(
                        "search {} in other package children {:?}",
                        DisplayRef::from((
                            self.ana.solver.nodes.with(*max_qual_ref),
                            &self.stores.label_store
                        )),
                        pack.make_position(&self.structural_positions, &self.stores),
                    );
                    r.extend(self.search(package_ref, max_qual_ref, &pack));
                }
                pack.up(&self.structural_positions);
            }
        }
        self.up(cursor);
        Ok(())
    }

    fn go_through_directories(
        &mut self,
        r: &mut Vec<SpHandle>,
        cursor: &mut Cursor,
        package_ref: &RefPtr,
        fq_decl_ref: &RefPtr,
        limit: &NodeIdentifier,
    ) -> Result<(), SearchStopEvent> {
        loop {
            log::debug!(
                "search in directory {:?} {:?}",
                cursor.curr,
                cursor
                    .scout
                    .make_position(&self.structural_positions, &self.stores)
            );
            let xx = cursor.curr.ok_or(SearchStopEvent::NoMore)?;
            let bb = self.stores.node_store.resolve(xx);
            // log::debug!("search in package {:?} {:?}", cursor.curr, t);
            for (i, xx) in bb.children().unwrap().iter_children().enumerate() {
                cursor.scout.goto(*xx, i);
                if Some(*xx) != cursor.prev {
                    r.extend(self.search(package_ref, fq_decl_ref, &cursor.scout));
                }
                cursor.scout.up(&self.structural_positions);
            }
            if &xx == limit {
                return Ok(());
            }
            self.up(cursor);
        }
    }
    fn go_through_folders(
        &mut self,
        r: &mut Vec<SpHandle>,
        cursor: &mut Cursor,
        package_ref: &RefPtr,
        fq_decl_ref: &RefPtr,
        other_folders: &[Scout],
    ) -> Result<(), SearchStopEvent> {
        log::debug!(
            "search in folder {:?} {:?}",
            cursor.curr,
            cursor
                .scout
                .make_position(&self.structural_positions, &self.stores)
        );
        for x in other_folders {
            let scout = x.clone();
            let xx = x.node_always(&self.structural_positions);
            let bb = self.stores.node_store.resolve(xx);
            for (i, xx) in bb.children().unwrap().iter_children().enumerate() {
                cursor.scout.goto(*xx, i);
                if Some(*xx) != cursor.prev {
                    r.extend(self.search(package_ref, fq_decl_ref, &scout));
                }
                cursor.scout.up(&self.structural_positions);
            }
        }
        Ok(())
    }

    fn extract_identifier(&mut self, b: &HashedNodeRef) -> Option<LabelIdentifier> {
        for xx in b.children().unwrap().iter_children() {
            let bb = self.stores.node_store.resolve(*xx);
            if bb.get_type() == Type::Identifier {
                let i = bb.get_label();
                return Some(*i);
            }
        }
        None
    }

    /// top down search of references matching [`p`][`i`]
    fn search(&mut self, p: &RefPtr, i: &RefPtr, s: &Scout) -> Vec<SpHandle> {
        // self.structural_positions
        //     .check_with(&self.stores, s)
        //     .expect("search");
        usage::RefsFinder::new(&self.stores, &mut self.ana, &mut self.structural_positions)
            .find_all(*p, *i, s.clone())
    }
}

pub fn goto_by_name<T: TreePath<NodeIdentifier>>(
    stores: &SimpleStores,
    mut p: T,
    name: &str,
) -> Option<T> {
    p.node()
        .and_then(|x| child_by_name_with_idx(stores, *x, name))
        .and_then(|(n, i)| {
            p.goto(n, i);
            Some(p)
        })
}

pub fn find_package_in_other_folders<
    'a,
    T: TreePath<NodeIdentifier> + Clone,
    U: Borrow<Path>,
    V: Iterator<Item = (U, T)>,
>(
    stores: &SimpleStores,
    package: &Path,
    root_package_file_path: &Path,
    other_folders: V,
) -> Vec<T> {
    let rel = package
        .strip_prefix(root_package_file_path)
        .expect("a relative path");
    other_folders
        .filter_map(|(p, x)| {
            let path = p.borrow().join(rel);
            let mut r = x.clone();
            for n in path.components() {
                let x = *r.node().unwrap();
                let n = std::os::unix::prelude::OsStrExt::as_bytes(n.as_os_str());
                let n = std::str::from_utf8(n).unwrap();
                let aaa = child_by_name_with_idx(stores, x, n);
                if let Some((x, i)) = aaa {
                    r.goto(x, i);
                } else {
                    return None;
                }
            }
            Some(r)
        })
        .collect()
}
