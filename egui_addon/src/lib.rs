pub mod syntax_highlighting;
pub mod code_editor;

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
