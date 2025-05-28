pub trait Accumulator: crate::tree_gen::Accumulator {
    type Unlabeled;
}

impl Accumulator for Acc {
    type Unlabeled = (Local, );
}

trait ProcessorSke<Acc: Accumulator, O = PathBuf> {
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
    fn stack(&mut self) -> &mut Vec<(Vec<O>, Acc)>;
    fn pre(&mut self, current: O);
    fn post(&mut self, acc: Acc) -> Option<Acc::Unlabeled>;
}

use crate::store::SimpleStores;

use std::path::{Path, PathBuf};

pub fn iter_dirs(root_buggy: &std::path::Path) -> impl Iterator<Item = std::fs::DirEntry> {
    std::fs::read_dir(root_buggy)
        .expect(&format!("{:?} should be a dir", root_buggy))
        .into_iter()
        .filter_map(|x| x.ok())
        .filter(|x| x.file_type().unwrap().is_dir())
}

struct Acc {}

// TODO make it vcs/files or a module of hyperast (it will also serve as an example)
pub struct PreprocessFileSys {
    pub main_stores: SimpleStores<TStore>,
    pub md_cache: MDCache,
}

impl PreprocessFileSys {
    fn generator(&mut self, text: &[u8]) -> TreeGen<TStore> {
        let line_break = if text.contains(&b'\r') {
            "\r\n".as_bytes().to_vec()
        } else {
            "\n".as_bytes().to_vec()
        };
        JavaTreeGen {
            line_break,
            stores: &mut self.main_stores,
            md_cache: &mut self.java_md_cache,
            more: (),
        }
    }

    pub(crate) fn help_handle_file<Acc>(
        &mut self,
        path: PathBuf,
        w: &mut Acc,
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
            assert!(!w.primary.children_names.contains(&name));
            w.push(name, full_node, skiped_ana);
        }
    }

    /// oid : Oid of a dir such that */src/main/java/ or */src/test/java/
    fn handle_java_directory<'b, 'd: 'b>(
        &mut self,
        path: PathBuf,
        filesys: &mut FileSys,
    ) -> (Local, ) {
        JavaProcessor::<JavaAcc>::new(self, filesys, path).process()
    }
}


pub fn parse_filesys(gen: &mut PreprocessFileSys, path: &Path) -> Local {
    let a = std::fs::read_dir(path)
        .expect(&format!("{:?} should be a dir", path))
        .into_iter()
        .filter_map(|x| x.ok())
        .map(|x| x);
    let mut w = Acc::new("".to_string());
    for x in a {
        match x.file_type() {
            Ok(t) => {
                if t.is_file() {
                    let file = std::fs::read_to_string(&x.path()).expect("the code");
                    let name = x.file_name();
                    let name = name.to_string_lossy();
                    {
                        let name: &str = &name;
                        let tree = match TreeGen::<TStore>::tree_sitter_parse(file.as_bytes()) {
                            Ok(t) => t,
                            Err(t) => t,
                        };
                        let full_node = gen.generator(file.as_bytes()).generate_file(
                            name.as_bytes(),
                            file.as_bytes(),
                            tree.walk(),
                        );

                        {
                            let local = full_node.local;
                            let skiped_ana = false; // TODO ez upgrade to handle skipping in files
                            let name = gen.main_stores.label_store.get_or_insert(name);
                            w.push(name, local, skiped_ana);
                        }
                    }
                } else if t.is_dir() {
                    let local = parse_filesys(gen, &x.path());
                    let skiped_ana = false; // TODO ez upgrade to handle skipping in files
                    let name = gen.main_stores.label_store.get_or_insert(
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
    make(w, &mut gen.main_stores)
}

pub(crate) struct Processor<'fs, 'prepro, Acc> {
    filesys: &'fs mut FileSys,
    prepro: &'prepro mut PreprocessFileSys,
    stack: Vec<(Vec<PathBuf>, Acc)>,
}
impl<'fs, 'prepro> Processor<'fs, 'prepro, Acc> {
    fn new(
        prepro: &'prepro mut PreprocessFileSys,
        filesys: &'fs mut FileSys,
        path: PathBuf,
    ) -> Self {
        let dir = filesys.find_file(&path);
        let name = dir.name();
        let prepared = prepare_dir_exploration(dir);
        let stack = vec![(prepared, Acc::new(name))];
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

trait Makeable {
    type Local;
    type Stores;
    fn new(name: String) -> Self;
    fn name(&self) -> String;
    fn make(self, stores: &mut Self::Stores) -> Self::Local;
    fn push(&mut self, name: String, node: Self::Local);
}

impl<'fs, 'prepro, Acc> ProcessorSke<Acc, PathBuf> for Processor<'fs, 'prepro, Acc>
where
    Acc: Makeable + Accumulator,
{
    fn pre(&mut self, path: PathBuf) {
        let file = self.filesys.find_file(&path);
        let name = file.name();
        if file.is_dir() {
            let acc = Acc::new(name);
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
    fn post(&mut self, acc: Acc) -> Option<(<Acc as Makeable>::Local, )> {
        let name = acc.name();
        let full_node = acc.make(&mut self.prepro.main_stores);
        let key = full_node.compressed_node.clone();
        self.prepro
            .md_cache
            .insert(key, MD::from(full_node.clone()));
        let name = self.prepro.main_stores.label_store.get_or_insert(name);
        if self.stack.is_empty() {
            Some((full_node, skiped_ana))
        } else {
            let w = &mut self.stack.last_mut().unwrap().1;
            w.push(name, full_node);
            None
        }
    }

    fn stack(&mut self) -> &mut Vec<(Vec<PathBuf>, Acc)> {
        &mut self.stack
    }
}
