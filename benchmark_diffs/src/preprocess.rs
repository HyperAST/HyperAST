use hyperast::{
    cyclomatic::Mcc,
    hashed::{IndexingHashBuilder, MetaDataHashsBuilder},
    store::{defaults::LabelIdentifier, SimpleStores},
    types::LabelStore as _,
};
use hyperast_vcs_git::java::JavaAcc;
use hyperast_gen_ts_java::{
    legion_with_refs::{self, FNode, JavaTreeGen, Local, MDCache, MD},
    types::{TStore, Type},
};
use std::path::{Path, PathBuf};

pub fn iter_dirs(root_buggy: &std::path::Path) -> impl Iterator<Item = std::fs::DirEntry> {
    std::fs::read_dir(root_buggy)
        .expect(&format!("{:?} should be a dir", root_buggy))
        .into_iter()
        .filter_map(|x| x.ok())
        .filter(|x| x.file_type().unwrap().is_dir())
}

pub fn parse_string_pair<'a>(
    stores: &mut SimpleStores<TStore>,
    md_cache: &mut MDCache,
    // java_tree_gen: &mut JavaTreeGen<'a, '_, TStore>,
    buggy: &'a str,
    fixed: &'a str,
) -> (FNode, FNode) {
    let full_node1 = parse_unchecked(buggy, "", stores, md_cache);
    let full_node2 = parse_unchecked(fixed, "", stores, md_cache);
    (full_node1, full_node2)
}

fn parse_unchecked<'b: 'stores, 'stores>(
    content: &'b str,
    name: &str,
    // java_tree_gen: &mut JavaTreeGen<'stores, '_, TStore>,
    stores: &'stores mut SimpleStores<TStore>,
    md_cache: &'_ mut MDCache,
) -> FNode {
    let tree = match legion_with_refs::tree_sitter_parse(content.as_bytes()) {
        Ok(t) => t,
        Err(t) => t,
    };
    let mut java_tree_gen = JavaTreeGen::new(stores, md_cache);
    let full_node1 = java_tree_gen.generate_file(name.as_bytes(), content.as_bytes(), tree.walk());
    full_node1
}

// TODO make it vcs/files or a module of hyperast (it will also serve as an example)
pub struct JavaPreprocessFileSys {
    pub main_stores: SimpleStores<TStore>,
    pub java_md_cache: MDCache,
}

impl JavaPreprocessFileSys {

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
        let line_break = if text.as_bytes().contains(&b'\r') {
            "\r\n".as_bytes().to_vec()
        } else {
            "\n".as_bytes().to_vec()
        };
        let mut java_tree_gen = JavaTreeGen::new(&mut self.main_stores, &mut self.java_md_cache)
            .with_line_break(line_break);
        let full_node = match legion_with_refs::tree_sitter_parse(text.as_bytes()) {
            Ok(tree) => {
                Ok(java_tree_gen.generate_file(name.as_bytes(), text.as_bytes(), tree.walk()))
            }
            Err(tree) => {
                Err(java_tree_gen.generate_file(name.as_bytes(), text.as_bytes(), tree.walk()))
            }
        };
        if let Ok(full_node) = full_node {
            //parse(&text, &name, &mut java_tree_gen) {
            let full_node = full_node.local;
            let skiped_ana = false; // TODO ez upgrade to handle skipping in files
            self.java_md_cache
                .insert(full_node.compressed_node, MD::from(full_node.clone()));
            let name = self.main_stores.label_store.get_or_insert(name);
            assert!(!w.primary.children_names.contains(&name));
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

pub(crate) type IsSkippedAna = bool;

pub fn parse_filesys(java_gen: &mut JavaPreprocessFileSys, path: &Path) -> Local {
    let a = std::fs::read_dir(path)
        .expect(&format!("{:?} should be a dir", path))
        .into_iter()
        .filter_map(|x| x.ok())
        .map(|x| x);
    let mut w = JavaAcc::new("".to_string(), None);
    for x in a {
        match x.file_type() {
            Ok(t) => {
                if t.is_file() {
                    let file = std::fs::read_to_string(&x.path()).expect("the code");
                    let name = x.file_name();
                    let name = name.to_string_lossy();
                    {
                        let name: &str = &name;
                        let tree = match legion_with_refs::tree_sitter_parse(file.as_bytes()) {
                            Ok(t) => t,
                            Err(t) => t,
                        };

                        let line_break = if file.as_bytes().contains(&b'\r') {
                            "\r\n".as_bytes().to_vec()
                        } else {
                            "\n".as_bytes().to_vec()
                        };
                        let mut java_tree_gen = JavaTreeGen::new(
                            &mut java_gen.main_stores,
                            &mut java_gen.java_md_cache,
                        )
                        .with_line_break(line_break);
                        let full_node = java_tree_gen.generate_file(
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

trait Accumulator: hyperast::tree_gen::Accumulator<Node = (LabelIdentifier, Self::Unlabeled)> {
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
        let stack = vec![(prepared, JavaAcc::new(name, None))];
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
            let acc = JavaAcc::new(name, None);
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
        let name = acc.primary.name.clone();
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
                !w.primary.children_names.contains(&name),
                "{:?} {:?}",
                w.primary.children_names,
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

fn make(
    acc: JavaAcc,
    stores: &mut SimpleStores<TStore>,
) -> hyperast_gen_ts_java::legion_with_refs::Local {
    let node_store = &mut stores.node_store;
    let label_store = &mut stores.label_store;
    let kind = Type::Directory;
    use hyperast::types::ETypeStore;
    let interned_kind = TStore::intern(kind);
    let label_id = label_store.get_or_insert(acc.primary.name.clone());

    let primary = acc
        .primary
        .map_metrics(|m| m.finalize(&interned_kind, &label_id, 0));
    let hashable = primary.metrics.hashs.most_discriminating();
    let eq = hyperast::store::nodes::legion::eq_node(
        &interned_kind,
        Some(&label_id),
        &primary.children,
    );
    let insertion = node_store.prepare_insertion(&hashable, eq);

    if let Some(id) = insertion.occupied_id() {
        let ana = None;
        let metrics = primary.metrics.map_hashs(|h| h.build());
        return Local {
            compressed_node: id,
            metrics,
            ana,
            mcc: Mcc::new(&kind),
            role: None,
            precomp_queries: Default::default(),
        };
    }

    let mut dyn_builder = hyperast::store::nodes::legion::dyn_builder::EntityBuilder::new();

    let ana = None;

    let children_is_empty = primary.children.is_empty();

    // TODO move add_md_ref_ana to better place
    #[cfg(feature = "impact")]
    hyperast_gen_ts_java::legion_with_refs::add_md_ref_ana(
        &mut dyn_builder,
        children_is_empty,
        None,
    );
    // }
    let metrics = primary.persist(&mut dyn_builder, interned_kind, label_id);
    let metrics = metrics.map_hashs(|h| h.build());
    let hashs = metrics.add_md_metrics(&mut dyn_builder, children_is_empty);
    hashs.persist(&mut dyn_builder);

    let vacant = insertion.vacant();
    let node_id = hyperast::store::nodes::legion::NodeStore::insert_built_after_prepare(
        vacant,
        dyn_builder.build(),
    );

    let full_node = Local {
        compressed_node: node_id.clone(),
        metrics,
        ana,
        mcc: Mcc::new(&kind),
        role: None,
        precomp_queries: Default::default(),
    };
    full_node
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
