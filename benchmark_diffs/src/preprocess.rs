use std::path::{Path, PathBuf};

use hyper_ast::{
    cyclomatic::Mcc,
    filter::{Bloom, BloomSize, BF},
    full::FullNode,
    hashed::{self, IndexingHashBuilder, MetaDataHashsBuilder, SyntaxNodeHashs},
    store::{
        defaults::{LabelIdentifier, NodeIdentifier},
        nodes::legion::{compo, EntryRef, NodeStore, PendingInsert, CS},
        SimpleStores,
    },
    tree_gen::{BasicGlobalData, SubTreeMetrics},
    types::{LabelStore as _, Type},
};
use hyper_ast_gen_ts_java::{
    impact::partial_analysis::PartialAnalysis,
    legion_with_refs::{BulkHasher, JavaTreeGen, Local, MDCache, MD},
};

pub fn iter_dirs(root_buggy: &std::path::Path) -> impl Iterator<Item = std::fs::DirEntry> {
    std::fs::read_dir(root_buggy)
        .expect(&format!("{:?} should be a dir", root_buggy))
        .into_iter()
        .filter_map(|x| x.ok())
        .filter(|x| x.file_type().unwrap().is_dir())
}

pub fn parse_string_pair<'a>(
    java_tree_gen: &mut JavaTreeGen<'a, '_>,
    buggy: &'a str,
    fixed: &'a str,
) -> (
    FullNode<BasicGlobalData, Local>,
    FullNode<BasicGlobalData, Local>,
) {
    let full_node1 = parse_unchecked(buggy, "", java_tree_gen);
    let full_node2 = parse_unchecked(fixed, "", java_tree_gen);
    (full_node1, full_node2)
}

fn parse_unchecked<'b: 'stores, 'stores>(
    content: &'b str,
    name: &str,
    java_tree_gen: &mut JavaTreeGen<'stores, '_>,
) -> FullNode<BasicGlobalData, Local> {
    let tree = match JavaTreeGen::tree_sitter_parse(content.as_bytes()) {
        Ok(t) => t,
        Err(t) => t,
    };
    let full_node1 = java_tree_gen.generate_file(name.as_bytes(), content.as_bytes(), tree.walk());
    full_node1
}
fn parse<'b: 'stores, 'stores>(
    content: &'b str,
    name: &str,
    java_tree_gen: &mut JavaTreeGen<'stores, '_>,
) -> Result<FullNode<BasicGlobalData, Local>, FullNode<BasicGlobalData, Local>> {
    match JavaTreeGen::tree_sitter_parse(content.as_bytes()) {
        Ok(tree) => {
            Ok(java_tree_gen.generate_file(name.as_bytes(), content.as_bytes(), tree.walk()))
        }
        Err(tree) => {
            Err(java_tree_gen.generate_file(name.as_bytes(), content.as_bytes(), tree.walk()))
        }
    }
}

pub struct JavaPreprocessFileSys {
    pub main_stores: SimpleStores,
    pub java_md_cache: MDCache,
}

impl JavaPreprocessFileSys {
    fn java_generator(&mut self, text: &[u8]) -> JavaTreeGen {
        let line_break = if text.contains(&b'\r') {
            "\r\n".as_bytes().to_vec()
        } else {
            "\n".as_bytes().to_vec()
        };
        JavaTreeGen {
            line_break,
            stores: &mut self.main_stores,
            md_cache: &mut self.java_md_cache,
        }
    }

    pub(crate) fn help_handle_java_file(
        &mut self,
        path: PathBuf,
        w: &mut JavaAcc,
        filesys: &mut FileSys,
    ) {
        let file = filesys.find_file(&path);
        if !file.is_file() {
            return;
        }
        let text = file.content();
        let name = file.name();
        if let Ok(full_node) = parse(&text, &name, &mut self.java_generator(text.as_bytes())) {
            let full_node = full_node.local;
            let skiped_ana = false; // TODO ez upgrade to handle skipping in files
            self.java_md_cache
                .insert(full_node.compressed_node, MD::from(full_node.clone()));
            let name = self.main_stores.label_store.get_or_insert(name);
            assert!(!w.children_names.contains(&name));
            w.push(name, full_node, skiped_ana);
        }
    }

    /// oid : Oid of a dir such that */src/main/java/ or */src/test/java/
    fn handle_java_directory<'b, 'd: 'b>(
        &mut self,
        path: PathBuf,
        filesys: &mut FileSys,
    ) -> (Local, IsSkippedAna) {
        JavaProcessor::<JavaAcc>::new(self, filesys, path).process()
    }
}

pub struct JavaAcc {
    pub(crate) name: String,
    pub(crate) children: Vec<NodeIdentifier>,
    pub(crate) children_names: Vec<LabelIdentifier>,
    pub(crate) metrics: SubTreeMetrics<SyntaxNodeHashs<u32>>,
    pub(crate) skiped_ana: bool,
    pub(crate) ana: PartialAnalysis,
}

pub(crate) const MAX_REFS: u32 = 10000; //4096;
impl JavaAcc {
    pub(crate) fn new(name: String) -> Self {
        Self {
            name,
            children_names: Default::default(),
            children: Default::default(),
            // simple: BasicAccumulator::new(kind),
            metrics: Default::default(),
            ana: PartialAnalysis::init(&Type::Directory, None, |_| panic!()),
            skiped_ana: false,
        }
    }
    pub(crate) fn push(&mut self, name: LabelIdentifier, full_node: Local, skiped_ana: bool) {
        self.children.push(full_node.compressed_node);
        self.children_names.push(name);
        self.metrics.acc(full_node.metrics);

        if let Some(ana) = full_node.ana {
            if ana.estimated_refs_count() < MAX_REFS
                && skiped_ana == false
                && self.skiped_ana == false
            {
                ana.acc(&Type::Directory, &mut self.ana);
            } else {
                self.skiped_ana = true;
            }
        }
    }
}

pub(crate) type IsSkippedAna = bool;

impl hyper_ast::tree_gen::Accumulator for JavaAcc {
    type Node = (LabelIdentifier, (Local, IsSkippedAna));
    fn push(&mut self, (name, (full_node, skiped_ana)): Self::Node) {
        self.children.push(full_node.compressed_node);
        self.children_names.push(name);
        self.metrics.acc(full_node.metrics);

        if let Some(ana) = full_node.ana {
            if ana.estimated_refs_count() < MAX_REFS
                && skiped_ana == false
                && self.skiped_ana == false
            {
                ana.acc(&Type::Directory, &mut self.ana);
            } else {
                self.skiped_ana = true;
            }
        }
    }
}

// impl Accumulator for JavaAcc {
//     type Unlabeled = (Local, IsSkippedAna);
// }
pub fn parse_filesys(java_gen: &mut JavaPreprocessFileSys, path: &Path) -> Local {
    let a = std::fs::read_dir(path)
        .expect(&format!("{:?} should be a dir", path))
        .into_iter()
        .filter_map(|x| x.ok())
        .map(|x| x);
    let mut w = JavaAcc::new("".to_string());
    for x in a {
        match x.file_type() {
            Ok(t) => {
                if t.is_file() {
                    let file = std::fs::read_to_string(&x.path()).expect("the code");
                    let name = x.file_name();
                    let name = name.to_string_lossy();
                    {
                        let name: &str = &name;
                        let tree = match JavaTreeGen::tree_sitter_parse(file.as_bytes()) {
                            Ok(t) => t,
                            Err(t) => t,
                        };
                        let full_node = java_gen.java_generator(file.as_bytes()).generate_file(
                            name.as_bytes(),
                            file.as_bytes(),
                            tree.walk(),
                        );

                        {
                            let local = full_node.local;
                            let skiped_ana = false; // TODO ez upgrade to handle skipping in files
                            let name = java_gen.main_stores.label_store.get_or_insert(name);
                            w.push(name, local, skiped_ana);
                        }
                    }
                } else if t.is_dir() {
                    let local = parse_filesys(java_gen, &x.path());
                    let skiped_ana = false; // TODO ez upgrade to handle skipping in files
                    let name = java_gen.main_stores.label_store.get_or_insert(
                        x.path()
                            .components()
                            .last()
                            .unwrap()
                            .as_os_str()
                            .to_string_lossy(),
                    );
                    w.push(name, local, skiped_ana);
                } else {
                    todo!("{:?}", x)
                }
            }
            Err(_) => panic!("no file type"),
        };
    }
    make(w, &mut java_gen.main_stores)
}

trait Accumulator: hyper_ast::tree_gen::Accumulator<Node = (LabelIdentifier, Self::Unlabeled)> {
    type Unlabeled;
}

impl Accumulator for JavaAcc {
    type Unlabeled = (Local, IsSkippedAna);
}

trait Processor<Acc: Accumulator> {
    fn process(&mut self) -> Acc::Unlabeled {
        loop {
            if let Some(current_dir) = self.stack().last_mut().expect("never empty").0.pop() {
                self.pre(current_dir)
            } else if let Some((_, acc)) = self.stack().pop() {
                if let Some(x) = self.post(acc) {
                    return x;
                }
            } else {
                panic!("never empty")
            }
        }
    }
    fn stack(&mut self) -> &mut Vec<(Vec<PathBuf>, Acc)>;
    fn pre(&mut self, current_dir: PathBuf);
    fn post(&mut self, acc: Acc) -> Option<Acc::Unlabeled>;
}

pub(crate) struct JavaProcessor<'fs, 'prepro, Acc> {
    filesys: &'fs mut FileSys,
    prepro: &'prepro mut JavaPreprocessFileSys,
    stack: Vec<(Vec<PathBuf>, Acc)>,
}
impl<'fs, 'prepro> JavaProcessor<'fs, 'prepro, JavaAcc> {
    fn new(
        prepro: &'prepro mut JavaPreprocessFileSys,
        filesys: &'fs mut FileSys,
        path: PathBuf,
    ) -> Self {
        let dir = filesys.find_file(&path);
        let name = dir.name();
        let prepared = prepare_dir_exploration(dir);
        let stack = vec![(prepared, JavaAcc::new(name))];
        Self {
            filesys,
            prepro,
            stack,
        }
    }
}
pub struct FileSys {}

impl FileSys {
    fn find_file(&mut self, path: &Path) -> MyFile {
        MyFile {
            path: path.to_path_buf(),
        }
    }
}

struct MyFile {
    path: PathBuf,
}

impl MyFile {
    fn name(&self) -> String {
        let name = self.path.file_name().unwrap();
        let name = name.to_string_lossy();
        name.to_string()
    }
    fn is_dir(&self) -> bool {
        self.path.is_dir()
    }
    fn is_file(&self) -> bool {
        self.path.is_file()
    }
    fn content(&self) -> String {
        std::fs::read_to_string(&self.path).unwrap()
    }
}

fn prepare_dir_exploration(dir: MyFile) -> Vec<PathBuf> {
    std::fs::read_dir(&dir.path)
        .expect(&format!("{:?} should be a dir", dir.path))
        .into_iter()
        .filter_map(|x| x.ok())
        .map(|x| x.path())
        .collect()
}

impl<'fs, 'prepro> Processor<JavaAcc> for JavaProcessor<'fs, 'prepro, JavaAcc> {
    fn pre(&mut self, path: PathBuf) {
        let file = self.filesys.find_file(&path);
        let name = file.name();
        if file.is_dir() {
            let acc = JavaAcc::new(name);
            let prepared: Vec<PathBuf> = prepare_dir_exploration(file);
            self.stack.push((prepared, acc));
        } else if file.is_file() {
            if name.ends_with(".java") {
                self.prepro.help_handle_java_file(
                    path,
                    &mut self.stack.last_mut().unwrap().1,
                    self.filesys,
                )
            } else {
                log::debug!("not java source file {:?}", name);
            }
        } else {
            panic!("not file nor dir: {:?}", path);
        }
    }
    fn post(&mut self, acc: JavaAcc) -> Option<(Local, IsSkippedAna)> {
        let skiped_ana = acc.skiped_ana;
        let name = acc.name.clone();
        let full_node = make(acc, &mut self.prepro.main_stores);
        let key = full_node.compressed_node.clone();
        self.prepro
            .java_md_cache
            .insert(key, MD::from(full_node.clone()));
        let name = self.prepro.main_stores.label_store.get_or_insert(name);
        if self.stack.is_empty() {
            Some((full_node, skiped_ana))
        } else {
            let w = &mut self.stack.last_mut().unwrap().1;
            assert!(
                !w.children_names.contains(&name),
                "{:?} {:?}",
                w.children_names,
                name
            );
            w.push(name, full_node.clone(), skiped_ana);
            None
        }
    }

    fn stack(&mut self) -> &mut Vec<(Vec<PathBuf>, JavaAcc)> {
        &mut self.stack
    }
}

fn make(acc: JavaAcc, stores: &mut SimpleStores) -> hyper_ast_gen_ts_java::legion_with_refs::Local {
    let node_store = &mut stores.node_store;
    let label_store = &mut stores.label_store;

    let hashs = acc.metrics.hashs;
    let size = acc.metrics.size + 1;
    let height = acc.metrics.height + 1;
    let size_no_spaces = acc.metrics.size_no_spaces + 1;
    let hbuilder = hashed::Builder::new(hashs, &Type::Directory, &acc.name, size_no_spaces);
    let hashable = &hbuilder.most_discriminating();
    let label_id = label_store.get_or_insert(acc.name.clone());

    let eq = eq_node(&Type::Directory, Some(&label_id), &acc.children);

    let insertion = node_store.prepare_insertion(&hashable, eq);

    let compute_md = || {
        let ana = {
            let ana = acc.ana;
            let ana = if acc.skiped_ana {
                log::info!(
                    "shop ana with at least {} refs",
                    ana.lower_estimate_refs_count()
                );
                ana
            } else {
                log::info!(
                    "ref count lower estimate in dir {}",
                    ana.lower_estimate_refs_count()
                );
                log::debug!("refs in directory");
                for x in ana.display_refs(label_store) {
                    log::debug!("    {}", x);
                }
                log::debug!("decls in directory");
                for x in ana.display_decls(label_store) {
                    log::debug!("    {}", x);
                }
                let c = ana.estimated_refs_count();
                if c < MAX_REFS {
                    ana.resolve()
                } else {
                    ana
                }
            };
            log::info!(
                "ref count in dir after resolver {}",
                ana.lower_estimate_refs_count()
            );
            log::debug!("refs in directory after resolve: ");
            for x in ana.display_refs(label_store) {
                log::debug!("    {}", x);
            }
            ana
        };

        let hashs = hbuilder.build();

        let metrics = SubTreeMetrics {
            size,
            height,
            size_no_spaces,
            hashs,
        };

        (ana, metrics)
    };

    if let Some(id) = insertion.occupied_id() {
        let (ana, metrics) = compute_md();
        return Local {
            compressed_node: id,
            metrics,
            ana: Some(ana),
            mcc: Mcc::new(&Type::Directory),
        };
    }

    let (ana, metrics) = compute_md();
    let hashs = hbuilder.build();
    let node_id = compress(
        insertion,
        label_id,
        acc.children,
        acc.children_names,
        size,
        height,
        size_no_spaces,
        hashs,
        acc.skiped_ana,
        &ana,
    );

    let full_node = Local {
        compressed_node: node_id.clone(),
        metrics,
        ana: Some(ana.clone()),
        mcc: Mcc::new(&Type::Directory),
    };
    full_node
}
fn compress(
    insertion: PendingInsert,
    label_id: LabelIdentifier,
    children: Vec<NodeIdentifier>,
    children_names: Vec<LabelIdentifier>,
    size: u32,
    height: u32,
    size_no_spaces: u32,
    hashs: SyntaxNodeHashs<u32>,
    skiped_ana: bool,
    ana: &PartialAnalysis,
) -> NodeIdentifier {
    let vacant = insertion.vacant();
    use tuples::combin::CombinConcat;
    macro_rules! insert {
        ( $c0:expr, $($c:expr),* $(,)? ) => {{
            let c = $c0;
            $(
                let c = c.concat($c);
            )*
            NodeStore::insert_after_prepare(vacant, c)
        }};
    }
    // NOTE needed as macro because I only implemented BulkHasher and Bloom for u8 and u16
    macro_rules! bloom {
        ( $t:ty ) => {{
            type B = $t;
            let it = ana.solver.iter_refs();
            let it = BulkHasher::<_, <B as BF<[u8]>>::S, <B as BF<[u8]>>::H>::from(it);
            let bloom = B::from(it);
            (B::SIZE, bloom)
        }};
    }
    match children.len() {
        0 => insert!((Type::Directory, label_id, hashs, BloomSize::None),),
        _ => {
            assert_eq!(children_names.len(), children.len());
            let c = (
                Type::Directory,
                label_id,
                compo::Size(size),
                compo::Height(height),
                compo::SizeNoSpaces(size_no_spaces),
                hashs,
                CS(children_names.into_boxed_slice()),
                CS(children.into_boxed_slice()),
            );
            match ana.estimated_refs_count() {
                x if x > 2048 || skiped_ana => {
                    insert!(c, (BloomSize::Much,))
                }
                x if x > 1024 => {
                    insert!(c, bloom!(Bloom::<&'static [u8], [u64; 64]>))
                }
                x if x > 512 => {
                    insert!(c, bloom!(Bloom::<&'static [u8], [u64; 32]>))
                }
                x if x > 256 => {
                    insert!(c, bloom!(Bloom::<&'static [u8], [u64; 16]>))
                }
                x if x > 150 => {
                    insert!(c, bloom!(Bloom::<&'static [u8], [u64; 8]>))
                }
                x if x > 100 => {
                    insert!(c, bloom!(Bloom::<&'static [u8], [u64; 4]>))
                }
                x if x > 30 => {
                    insert!(c, bloom!(Bloom::<&'static [u8], [u64; 2]>))
                }
                x if x > 15 => {
                    insert!(c, bloom!(Bloom::<&'static [u8], u64>))
                }
                x if x > 8 => {
                    insert!(c, bloom!(Bloom::<&'static [u8], u32>))
                }
                x if x > 0 => {
                    insert!(c, bloom!(Bloom::<&'static [u8], u16>))
                }
                _ => insert!(c, (BloomSize::None,)),
            }
        }
    }
}
pub fn eq_node<'a>(
    kind: &'a Type,
    label_id: Option<&'a LabelIdentifier>,
    children: &'a [NodeIdentifier],
) -> impl Fn(EntryRef) -> bool + 'a {
    return move |x: EntryRef| {
        let t = x.get_component::<Type>();
        if t != Ok(kind) {
            return false;
        }
        let l = x.get_component::<LabelIdentifier>().ok();
        if l != label_id {
            return false;
        } else {
            let cs = x.get_component::<CS<NodeIdentifier>>();
            let r = match cs {
                Ok(CS(cs)) => cs.as_ref() == children,
                Err(_) => children.is_empty(),
            };
            if !r {
                return false;
            }
        }
        true
    };
}

pub fn parse_dir_pair(
    java_gen: &mut JavaPreprocessFileSys,
    src: &Path,
    dst: &Path,
) -> (Local, Local) {
    let mut filesys = FileSys {};
    let src = java_gen
        .handle_java_directory(src.to_path_buf(), &mut filesys)
        .0;
    let dst = java_gen
        .handle_java_directory(dst.to_path_buf(), &mut filesys)
        .0;
    // let src = parse_filesys(java_gen, src);
    // let dst = parse_filesys(java_gen, dst);
    (src, dst)
}
