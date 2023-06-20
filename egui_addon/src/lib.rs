pub mod code_editor;
pub mod egui_utils;
pub mod interactive_split;
pub mod meta_edge;
pub mod multi_split;
pub mod syntax_highlighting;
pub mod async_exec;

#[derive(Debug, Clone)]
pub struct Lang {
    pub name: String,
    pub lang: tree_sitter::Language,
}

impl std::hash::Hash for Lang {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}
