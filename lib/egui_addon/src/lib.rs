#![allow(unused)]
pub mod async_exec;
pub mod code_editor;
pub mod egui_utils;
pub mod hscroll;
pub mod interactive_split;
pub mod meta_edge;
pub mod multi_split;
pub mod syntax_highlighting;

#[derive(Debug, Clone)]
pub struct Lang {
    pub name: String,
    #[cfg(feature = "ts_highlight")]
    pub lang: tree_sitter::Language,
}

impl std::hash::Hash for Lang {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

pub trait Languages {
    fn get(&self, lang: &str) -> Option<Lang>;
}
