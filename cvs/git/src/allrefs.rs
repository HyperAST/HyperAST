use std::{borrow::Borrow, fmt::Display, io::Write, path::Path, time::Instant};

use hyperast::{
    position::{
        Position, SpHandle, StructuralPosition, StructuralPositionStore, TreePath, TreePathMut,
        TypedTreePath,
    },
    store::{
        defaults::{LabelIdentifier, NodeIdentifier},
        nodes::legion::HashedNodeRef,
    },
    types::{
        IterableChildren, LabelStore, Labeled, NodeId, TypeTrait, Typed, TypedNodeStore,
        WithChildren,
    },
};
use hyperast_gen_ts_java::{
    impact::{
        element::{IdentifierFormat, LabelPtr, RefPtr, RefsEnum},
        partial_analysis::PartialAnalysis,
        reference::DisplayRef,
        usage::{self, remake_pkg_ref},
    },
    types::Type,
    usage::declarations::IterDeclarations,
};
use num::ToPrimitive;

use crate::{maven::IterMavenModules, preprocessed::child_by_name_with_idx, SimpleStores};

const REFERENCES_SERIALIZATION_SUMMARY: bool = false;

/// TODO before enabling, make sure the recusive reference search works, it is often needed for members eg. chained calls.
/// By recusive search on for example methods, I mean searching for refs to members with type (including ret type) of containing class of prev member.
const SEARCH_MEMBERS: bool = false;

type JavaIdN = hyperast_gen_ts_java::types::TIdN<NodeIdentifier>;

type Scout = hyperast::position::Scout<NodeIdentifier, u16>;
type TypedScout = hyperast::position::TypedScout<JavaIdN, u16>;

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
        for ExpandedDeclaration(decl, root_folder, of) in declarations {
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
    declaration: &DeclSp,
    root_folder: StructuralPosition,
    other_folders: Vec<StructuralPosition>,
) -> Option<(SearchKinds, Vec<Position>)> {
    let mut structural_positions = StructuralPositionStore::new(root);
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
    declaration: &DeclSp,
    root_folder: StructuralPosition,
    other_folders: Vec<StructuralPosition>,
) -> Option<(SearchKinds, Vec<SpHandle>)> {
    let b = stores
        .node_store
        .try_resolve_typed::<JavaIdN>(declaration.node().unwrap())?
        .0;
    let t = b.get_type();
    let of = other_folders
        .iter()
        .map(|x| (x.make_position(stores).file().to_owned(), x.clone()));
    let decl_pos = &declaration.make_position(stores);
    let p_in_of = find_package_in_other_folders(stores, decl_pos.file(), decl_pos.file(), of);
    let p_in_of: Vec<TypedScout> = p_in_of
        .into_iter()
        .map(|x| {
            structural_positions.type_scout(&mut Into::<Scout>::into((x.clone(), 0)), unsafe {
                JavaIdN::from_ref_id(x.node().unwrap())
            })
        })
        .collect();
    let other_folders: Vec<TypedScout> = other_folders
        .into_iter()
        .map(|x| {
            structural_positions.type_scout(&mut Into::<Scout>::into((x.clone(), 0)), unsafe {
                JavaIdN::from_ref_id(x.node().unwrap())
            })
        })
        .collect();

    let decl = structural_positions
        .type_scout(&mut Into::<Scout>::into((declaration.clone(), 0)), unsafe {
            JavaIdN::from_ref_id(declaration.node().unwrap())
        });
    if t == Type::ClassDeclaration
        || t == Type::InterfaceDeclaration
        || t == Type::AnnotationTypeDeclaration
    {
        let rs = RefsFinder::new(stores, structural_positions)
            .find_type_declaration_references_unchecked(
                decl,
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
                decl,
                root_folder.node().unwrap(),
                &p_in_of,
            );
        Some((SearchKinds::TypeDecl, rs))
    } else if t == Type::ClassBody {
        let rs = RefsFinder::new(stores, structural_positions).find_this_unchecked(decl);
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
            .find_localvar_declaration_references_unchecked(decl);
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

type DeclSp = StructuralPosition;
pub(crate) struct ExpandedDeclaration(DeclSp, MavenModuleSp, Vec<FolderSp>);

pub(crate) fn modules_iter_declarations<'a>(
    stores: &'a SimpleStores,
    modules: IterMavenModules<'a, MavenModuleSp>,
) -> impl Iterator<Item = ExpandedDeclaration> + 'a {
    modules
        .flat_map(|maven_module| maven_module_folders(stores, maven_module))
        .flat_map(|ExpandedMavenModule(f, _m, of)| make_decl_iter(stores, f, of))
}

pub(crate) fn iter_declarations<'a>(
    stores: &'a SimpleStores,
    maven_module: MavenModuleSp,
) -> impl Iterator<Item = ExpandedDeclaration> + 'a {
    maven_module_folders(stores, maven_module)
        .into_iter()
        .flat_map(|ExpandedMavenModule(f, _m, of)| make_decl_iter(stores, f, of))
}

fn make_decl_iter(
    stores: &SimpleStores,
    f: FolderSp,
    of: Vec<FolderSp>,
) -> impl Iterator<Item = ExpandedDeclaration> + '_ {
    let n = *f.node().unwrap();
    // let n = unsafe {
    //     JavaIdN::from_id(n)
    // };
    IterDeclarations::new(stores, f.clone(), n)
        .map(move |x| ExpandedDeclaration(x, f.clone(), of.clone()))
}

type MavenModuleSp = StructuralPosition;
type FolderSp = StructuralPosition;

struct ExpandedMavenModule(FolderSp, MavenModuleSp, Vec<FolderSp>);

fn maven_module_folders(
    stores: &SimpleStores,
    maven_module: MavenModuleSp,
) -> Vec<ExpandedMavenModule> {
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
        r.push(ExpandedMavenModule(
            source_tests,
            maven_module.clone(),
            vec![],
        ))
    }
    if let Some(source) = source {
        r.push(ExpandedMavenModule(source, maven_module, test_folders))
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

struct TypedCursor {
    scout: TypedScout,
    prev: Option<JavaIdN>,
    curr: Option<JavaIdN>,
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
        decl: TypedScout,
        limit: &NodeIdentifier,
        other_folders: &[TypedScout],
        mirror_packages: &[TypedScout],
    ) -> Vec<SpHandle> {
        let mut r = vec![];
        // let p = decl.make_position(&self.structural_positions, self.stores);
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
        decl: &TypedScout,
        limit: &NodeIdentifier,
        other_folders: &[TypedScout],
        mirror_packages: &[TypedScout],
    ) -> Result<(), SearchStopEvent> {
        let mut scout = decl.clone();
        self.structural_positions.push_typed(&mut scout); // memory footprint ?
        let qual_ref = self.init_type_decl(r, &decl);

        let prev_offset = scout.offset_always(&self.structural_positions) - 1;
        let prev = Some(scout.node_always(&self.structural_positions).unwrap());
        let curr = scout.up(&self.structural_positions).map(|x| x.unwrap());

        let mut cursor = TypedCursor { scout, prev, curr };

        log::trace!("go_through_type_declarations");
        let qual_ref = self.go_through_type_declarations(r, prev_offset, &mut cursor, qual_ref)?;
        log::trace!("go_through_program");
        let (package_ref, fq_decl_ref) = self.go_through_program(r, &mut cursor, qual_ref)?;
        {
            let mut scout = decl.clone();
            let prev = Some(scout.node_always(&self.structural_positions).unwrap());
            let curr = scout.up(&self.structural_positions).map(|x| x.unwrap());

            let mut cursor = TypedCursor { scout, prev, curr };
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

    fn find_this_unchecked(mut self, decl: TypedScout) -> Vec<SpHandle> {
        let mut r = vec![];
        self.find_this(&mut r, &decl);
        r
    }

    fn find_field_declaration_references_unchecked(
        self,
        decl: TypedScout,
        limit: &NodeIdentifier,
        mirror_packages: &[TypedScout],
    ) -> Vec<SpHandle> {
        let mut r = vec![];
        let p = decl.make_position(&self.structural_positions, self.stores);
        let res = self.find_field_declaration_references(&mut r, &decl, limit, mirror_packages);
        if let Err(err) = res {
            log::error!("search of {} ended with {:?}", p, err);
        }
        r
    }

    fn find_field_declaration_references(
        mut self,
        r: &mut Vec<SpHandle>,
        decl: &TypedScout,
        limit: &NodeIdentifier,
        mirror_packages: &[TypedScout],
    ) -> Result<(), SearchStopEvent> {
        let mut scout = decl.clone();
        self.structural_positions.push_typed(&mut scout); // memory footprint ?
        let qual_ref = self.init_field_decl(r, &decl);

        let prev = Some(scout.node_always(&self.structural_positions).unwrap());
        let curr = scout.up(&self.structural_positions).map(|x| x.unwrap());

        let mut cursor = TypedCursor { scout, prev, curr };

        let qual_ref = self.go_through_type_declarations_for_field(r, &mut cursor, qual_ref)?;
        // TODO make sure it does not need special handling
        let (package_ref, fq_decl_ref) = self.go_through_program(r, &mut cursor, qual_ref)?;
        self.go_through_package(r, &mut cursor, mirror_packages, &package_ref, &fq_decl_ref)?;
        self.go_through_directories(r, &mut cursor, &package_ref, &fq_decl_ref, limit)?;

        Ok(())
    }

    fn init_field_decl<'b>(&mut self, _r: &mut Vec<SpHandle>, decl: &TypedScout) -> RefPtr {
        let b = self
            .stores
            .node_store
            .resolve_typed(&decl.node_always(&self.structural_positions).unwrap());
        let t = b.get_type();
        log::info!(
            "now search for {:?} at {:?}",
            &t,
            decl.make_position(&self.structural_positions, self.stores)
        );
        let name = {
            let mut i = None;
            for xx in b.children().unwrap().iter_children() {
                let bb = self.stores.node_store.try_resolve_typed(xx).unwrap().0;
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

    fn find_localvar_declaration_references_unchecked(self, decl: TypedScout) -> Vec<SpHandle> {
        let mut r = vec![];
        let p = decl.make_position(&self.structural_positions, self.stores);
        let res = self.find_localvar_declaration_references(&mut r, &decl);
        if let Err(err) = res {
            log::error!("search of {} ended with {:?}", p, err);
        }
        r
    }

    fn find_localvar_declaration_references(
        mut self,
        r: &mut Vec<SpHandle>,
        decl: &TypedScout,
    ) -> Result<(), SearchStopEvent> {
        let mut scout = decl.clone();
        self.structural_positions.push_typed(&mut scout); // memory footprint ?
        let qual_ref = self.init_localvar_decl(r, &decl);

        let prev = Some(scout.node_always(&self.structural_positions).unwrap());
        let curr = scout.up(&self.structural_positions).map(|x| x.unwrap());

        let mut cursor = TypedCursor { scout, prev, curr };

        self.go_through_block(r, &mut cursor, qual_ref)
    }

    fn init_localvar_decl<'b>(&mut self, _r: &mut Vec<SpHandle>, decl: &TypedScout) -> RefPtr {
        let b = self
            .stores
            .node_store
            .resolve_typed(&decl.node_always(&self.structural_positions).unwrap());
        let t = b.get_type();
        log::info!(
            "now search for {:?} at {:?}",
            &t,
            decl.make_position(&self.structural_positions, self.stores)
        );
        let name = {
            let mut i = None;

            if t == Type::Identifier {
                i = Some(*b.get_label_unchecked());
            } else if t == Type::LocalVariableDeclaration || t == Type::SpreadParameter {
                for xx in b.children().unwrap().iter_children() {
                    let bb = self.stores.node_store.try_resolve_typed(xx).unwrap().0;
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
        cursor: &mut TypedCursor,
        qual_ref: RefPtr,
    ) -> Result<(), SearchStopEvent> {
        let mm = self.ana.solver.intern(RefsEnum::MaybeMissing);
        let xx = cursor.curr.ok_or(SearchStopEvent::NoMore)?;
        let bb = self.stores.node_store.resolve_typed(&xx);
        let t = bb.get_type();

        if t == Type::FormalParameters || t == Type::InferredParameters {
            self.up_typed(cursor);
            return self.go_through_block(r, cursor, qual_ref);
        }

        for (i, xx) in bb.children().unwrap().iter_children().enumerate() {
            let xx = self.stores.node_store.try_typed(xx).unwrap();
            cursor.scout.goto_typed(xx, num::cast(i).unwrap());
            if Some(xx) != cursor.prev {
                log::debug!(
                    "search {} in block children {:?} {:?}",
                    DisplayRef::from((
                        self.ana.solver.nodes.with(*&qual_ref),
                        &self.stores.label_store
                    )),
                    cursor.curr,
                    cursor
                        .scout
                        .make_position(&self.structural_positions, self.stores)
                );
                r.extend({
                    let p = &mm;
                    let i = &qual_ref;
                    let s = &cursor.scout;
                    usage::RefsFinder::new(
                        self.stores,
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
            self.up_typed(cursor);
            self.go_through_block(r, cursor, qual_ref)?;
        }

        Ok(())
    }

    fn find_this<'b>(&mut self, r: &mut Vec<SpHandle>, decl: &TypedScout) {
        let b = self
            .stores
            .node_store
            .resolve_typed(&decl.node_always(&self.structural_positions).unwrap());
        let mm = self.ana.solver.intern(RefsEnum::MaybeMissing);
        let mut scout = decl.clone();
        for (i, xx) in b.children().unwrap().iter_children().enumerate() {
            let xx = self.stores.node_store.try_typed(xx).unwrap();
            let mut s = scout.clone();
            s.goto_typed(xx, num::cast(i).unwrap());

            log::debug!("try search this");
            r.extend(
                usage::RefsFinder::new(self.stores, &mut self.ana, &mut self.structural_positions)
                    .find_all_is_this(mm, s),
            );
        }
    }

    fn init_type_decl<'b>(&mut self, r: &mut Vec<SpHandle>, decl: &TypedScout) -> RefPtr {
        let b = self
            .stores
            .node_store
            .resolve_typed(&decl.node_always(&self.structural_positions).unwrap());
        let t = b.get_type();
        log::info!(
            "now search for {:?} at {:?}",
            &t,
            decl.make_position(&self.structural_positions, self.stores)
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
            let xx = self.stores.node_store.try_typed(xx).unwrap();
            scout.goto_typed(xx, num::cast(i).unwrap());

            log::debug!("try search this");
            // r.extend(self.search(&mm, &thiss, &decl)); for now use something more explicit
            r.extend(
                usage::RefsFinder::new(self.stores, &mut self.ana, &mut self.structural_positions)
                    .find_all_is_this(mm, scout.to_owned()),
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

    fn up_typed(&self, mut cursor: &mut TypedCursor) {
        cursor.prev = cursor.curr.clone();
        cursor.curr = cursor
            .scout
            .up(&self.structural_positions)
            .map(|x| x.unwrap());
    }

    fn go_through_type_declarations_with_fully_qual_ref(
        &mut self,
        r: &mut Vec<SpHandle>,
        cursor: &mut TypedCursor,
        fq_decl_ref: RefPtr,
    ) -> Result<(), SearchStopEvent> {
        let mm = self.ana.solver.intern(RefsEnum::MaybeMissing);
        // go through classes if inner, stops at blocks and object creation expr
        loop {
            let x = cursor.curr.clone().ok_or(SearchStopEvent::NoMore)?;
            let b = self.stores.node_store.resolve_typed(&x);
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
                    let xx = self.stores.node_store.try_typed(xx).unwrap();
                    cursor.scout.goto_typed(xx, num::cast(i).unwrap());
                    if Some(xx) != cursor.prev.clone() {
                        // search ?.A or ?.B.A or ...
                        r.extend(self.search(&mm, &fq_decl_ref, &cursor.scout));
                    }
                    cursor.scout.up(&self.structural_positions);
                }
                // TODO do things to find type of field at the same level than type decl
                // should loop through siblings
                self.up_typed(cursor);
                let xx = cursor.curr.clone().ok_or(SearchStopEvent::NoMore)?;
                let bb = self.stores.node_store.resolve_typed(&xx);
                let tt = bb.get_type();
                if tt == Type::ObjectCreationExpression {
                    return Err(SearchStopEvent::Blocked);
                } else if tt == Type::EnumBody {
                    for (i, xx) in bb.children().unwrap().iter_children().enumerate() {
                        let xx = self.stores.node_store.try_typed(xx).unwrap();
                        cursor.scout.goto_typed(xx, num::cast(i).unwrap());
                        if Some(xx) != cursor.prev.clone() {
                            // search ?.A or ?.B.A or ...
                            r.extend(self.search(&mm, &fq_decl_ref, &cursor.scout));
                        }
                        cursor.scout.up(&self.structural_positions);
                    }
                    // TODO do things to find type of field at the same level than type decl
                    // should loop through siblings
                    self.up_typed(cursor);
                    let xx = cursor.curr.clone().ok_or(SearchStopEvent::NoMore)?;
                    let bb = self.stores.node_store.resolve_typed(&xx);
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
                self.up_typed(cursor);
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
        prev_offset: u16,
        cursor: &mut TypedCursor,
        mut qual_ref: RefPtr,
    ) -> Result<RefPtr, SearchStopEvent> {
        let mm = self.ana.solver.intern(RefsEnum::MaybeMissing);
        // go through classes if inner, stops at blocks and object creation expr
        loop {
            let x = cursor.curr.clone().ok_or(SearchStopEvent::NoMore)?;
            let b = self.stores.node_store.resolve_typed(&x);
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
                    let xx = self.stores.node_store.try_typed(xx).unwrap();
                    cursor.scout.goto_typed(xx, num::cast(i).unwrap());
                    if Some(xx) != cursor.prev.clone() {
                        // search ?.A or ?.B.A or ...
                        log::trace!("go_through_type_declarations s1");
                        r.extend(self.search(&mm, &qual_ref, &cursor.scout));
                    }
                    cursor.scout.up(&self.structural_positions);
                }
                // TODO do things to find type of field at the same level than type decl
                // should loop through siblings
                // self.structural_positions.check_with(&self.stores, &cursor.scout).expect("before");
                self.up_typed(cursor);
                // self.structural_positions.check_with(&self.stores, &cursor.scout).expect("after");
                let xx = cursor.curr.clone().ok_or(SearchStopEvent::NoMore)?;
                let bb = self.stores.node_store.resolve_typed(&xx);
                let tt = bb.get_type();
                let (_, bb, _) = if tt == Type::ObjectCreationExpression {
                    return Err(SearchStopEvent::Blocked);
                } else if tt == Type::EnumBody {
                    for (i, xx) in bb.children().unwrap().iter_children().enumerate() {
                        let xx = self.stores.node_store.try_typed(xx).unwrap();
                        cursor.scout.goto_typed(xx, num::cast(i).unwrap());
                        if Some(xx) != cursor.prev.clone() {
                            // search ?.A or ?.B.A or ...
                            log::trace!("go_through_type_declarations s2");
                            r.extend(self.search(&mm, &qual_ref, &cursor.scout));
                        }
                        cursor.scout.up(&self.structural_positions);
                    }
                    // TODO do things to find type of field at the same level than type decl
                    // should loop through siblings
                    self.up_typed(cursor);
                    let xx = cursor.curr.clone().ok_or(SearchStopEvent::NoMore)?;
                    let bb = self.stores.node_store.resolve_typed(&xx);
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
                self.up_typed(cursor);
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
                    .skip(prev_offset.to_usize().unwrap())
                    .enumerate()
                {
                    let xx = self.stores.node_store.try_typed(xx).unwrap();
                    scout.goto_typed(xx, num::cast(i).unwrap());
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
                    .skip(prev_offset.to_usize().unwrap())
                    .enumerate()
                {
                    let xx = self.stores.node_store.try_typed(xx).unwrap();
                    scout.goto_typed(xx, num::cast(i).unwrap());
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
        cursor: &mut TypedCursor,
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
            let b = self.stores.node_store.resolve_typed(&x);
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
                    let xx = self.stores.node_store.try_typed(xx).unwrap();
                    cursor.scout.goto_typed(xx, num::cast(i).unwrap());
                    if Some(xx) != cursor.prev.clone() {
                        // search ?.A or ?.B.A or ...
                        r.extend(self.search(&mm, &qual_ref, &cursor.scout));
                        r.extend(self.search(&mm, &thiss_qual_ref, &cursor.scout));
                    }
                    cursor.scout.up(&self.structural_positions);
                }
                // TODO do things to find type of field at the same level than type decl
                // should loop through siblings
                self.up_typed(cursor);
                let xx = cursor.curr.clone().ok_or(SearchStopEvent::NoMore)?;
                let bb = self.stores.node_store.resolve_typed(&xx);
                let tt = bb.get_type();
                let (bb, tt) = if tt == Type::EnumBody {
                    self.up_typed(cursor);
                    let xx = cursor.curr.clone().ok_or(SearchStopEvent::NoMore)?;
                    let bb = self.stores.node_store.resolve_typed(&xx);
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
                self.up_typed(cursor);
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
        cursor: &mut TypedCursor,
        qual_ref: RefPtr,
    ) -> Result<(RefPtr, RefPtr), SearchStopEvent> {
        let before_p_ref;
        let mut max_qual_ref = qual_ref;
        let mut package_ref = self.ana.solver.intern(RefsEnum::MaybeMissing);
        // go through classes if inner
        let x = cursor.curr.clone().ok_or(SearchStopEvent::NoMore)?;
        let b = hyperast::types::TypedNodeStore::resolve(&self.stores.node_store, &x);
        let t = b.get_type();
        assert_eq!(t, Type::Program);
        log::debug!(
            "go through program {:?} {:?} {}",
            cursor.curr,
            cursor
                .scout
                .make_position(&self.structural_positions, self.stores),
            b.child_count()
        );
        // go through program i.e. package declaration
        before_p_ref = max_qual_ref;
        for (i, xx) in b.children().unwrap().iter_children().enumerate() {
            let (bb, xx) =
                hyperast::types::TypedNodeStore::try_resolve(&self.stores.node_store, xx).unwrap();
            cursor.scout.goto_typed(xx, num::cast(i).unwrap());
            let tt = bb.get_type();
            log::debug!("in program {:?}", tt);
            if tt == Type::PackageDeclaration {
                package_ref = remake_pkg_ref(self.stores, &mut self.ana, xx)
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
                if Some(xx) != cursor.prev.clone() && max_qual_ref != before_p_ref {
                    // search ?.A in file ie. other top level classes
                    r.extend(self.search(&package_ref, &before_p_ref, &cursor.scout));
                }
                // search /.p.A in file
                r.extend(self.search(&package_ref, &max_qual_ref, &cursor.scout));
            }
            cursor.scout.up(&self.structural_positions);
        }
        self.up_typed(cursor);
        cursor
            .curr
            .and(Some((package_ref, max_qual_ref)))
            .ok_or(SearchStopEvent::NoMore)
    }

    fn go_through_package<'b>(
        &mut self,
        r: &mut Vec<SpHandle>,
        cursor: &mut TypedCursor,
        mirror_packages: &[TypedScout],
        package_ref: &RefPtr,
        max_qual_ref: &RefPtr,
    ) -> Result<(), SearchStopEvent> {
        let xx = cursor.curr.ok_or(SearchStopEvent::NoMore)?;
        let bb = self.stores.node_store.resolve_typed(&xx);
        let t = bb.get_type();
        log::info!(
            "search in package {:?} {:?} {:?}",
            cursor.curr,
            t,
            cursor
                .scout
                .make_position(&self.structural_positions, self.stores)
        );

        for (i, xx) in bb.children().unwrap().iter_children().enumerate() {
            let xx = self.stores.node_store.try_typed(xx).unwrap();
            cursor.scout.goto_typed(xx, num::cast(i).unwrap());
            if Some(xx) != cursor.prev {
                log::debug!(
                    "search {} in package children {:?} {:?}",
                    DisplayRef::from((
                        self.ana.solver.nodes.with(*max_qual_ref),
                        &self.stores.label_store
                    )),
                    cursor.curr,
                    cursor
                        .scout
                        .make_position(&self.structural_positions, self.stores)
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
                .resolve_typed(&pack.node_always(&self.structural_positions).unwrap());
            for (i, xx) in bb.children().unwrap().iter_children().enumerate() {
                let xx = self.stores.node_store.try_typed(xx).unwrap();
                let t = bb.get_type();
                pack.goto_typed(xx, num::cast(i).unwrap());
                if t == Type::Program {
                    log::debug!(
                        "search {} in other package children {:?}",
                        DisplayRef::from((
                            self.ana.solver.nodes.with(*max_qual_ref),
                            &self.stores.label_store
                        )),
                        pack.make_position(&self.structural_positions, self.stores),
                    );
                    r.extend(self.search(package_ref, max_qual_ref, &pack));
                }
                pack.up(&self.structural_positions);
            }
        }
        self.up_typed(cursor);
        Ok(())
    }

    fn go_through_directories(
        &mut self,
        r: &mut Vec<SpHandle>,
        cursor: &mut TypedCursor,
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
                    .make_position(&self.structural_positions, self.stores)
            );
            let xx = cursor.curr.ok_or(SearchStopEvent::NoMore)?;
            let bb = self.stores.node_store.resolve_typed(&xx);
            // log::debug!("search in package {:?} {:?}", cursor.curr, t);
            for (i, xx) in bb.children().unwrap().iter_children().enumerate() {
                let (_, xx) = self.stores.node_store.try_resolve_typed(xx).unwrap();
                cursor.scout.goto_typed(xx, num::cast(i).unwrap());
                if Some(xx) != cursor.prev {
                    r.extend(self.search(package_ref, fq_decl_ref, &cursor.scout));
                }
                cursor.scout.up(&self.structural_positions);
            }
            if xx.as_id() == limit {
                return Ok(());
            }
            self.up_typed(cursor);
        }
    }
    fn go_through_folders(
        &mut self,
        r: &mut Vec<SpHandle>,
        cursor: &mut TypedCursor,
        package_ref: &RefPtr,
        fq_decl_ref: &RefPtr,
        other_folders: &[TypedScout],
    ) -> Result<(), SearchStopEvent> {
        log::debug!(
            "search in folder {:?} {:?}",
            cursor.curr,
            cursor
                .scout
                .make_position(&self.structural_positions, self.stores)
        );
        for x in other_folders {
            let scout = x.clone();
            let xx = x.node_always(&self.structural_positions).unwrap();
            let bb = self.stores.node_store.resolve_typed(&xx);
            for (i, xx) in bb.children().unwrap().iter_children().enumerate() {
                let (_, xx) = self.stores.node_store.try_resolve_typed(xx).unwrap();
                cursor.scout.goto_typed(xx, num::cast(i).unwrap());
                if Some(xx) != cursor.prev {
                    r.extend(self.search(package_ref, fq_decl_ref, &scout));
                }
                cursor.scout.up(&self.structural_positions);
            }
        }
        Ok(())
    }

    fn extract_identifier(&mut self, b: &HashedNodeRef<'a, JavaIdN>) -> Option<LabelIdentifier> {
        for xx in b.children().unwrap().iter_children() {
            let bb = self
                .stores
                .node_store
                .try_resolve_typed::<JavaIdN>(xx)
                .unwrap()
                .0;
            if bb.get_type() == Type::Identifier {
                let i = bb.get_label_unchecked();
                return Some(*i);
            }
        }
        None
    }

    /// top down search of references matching [`p`][`i`]
    fn search(&mut self, p: &RefPtr, i: &RefPtr, s: &TypedScout) -> Vec<SpHandle> {
        // self.structural_positions
        //     .check_with(&self.stores, s)
        //     .expect("search");
        usage::RefsFinder::new(self.stores, &mut self.ana, &mut self.structural_positions).find_all(
            *p,
            *i,
            s.clone(),
        )
    }
}

pub fn goto_by_name<T: TreePathMut<NodeIdentifier, u16>>(
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
    T: TreePathMut<NodeIdentifier, u16> + Clone,
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
                let n = std::ffi::OsStr::as_encoded_bytes(n.as_os_str());
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
