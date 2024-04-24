pub mod async_exec;
pub mod code_editor;
pub mod egui_utils;
pub mod interactive_split;
pub mod meta_edge;
pub mod multi_split;
pub mod syntax_highlighting;

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

#[derive(Clone, Default)]
pub struct Languages();

impl Languages {
    pub fn get(&self, name: &str) -> Option<Lang> {
        match name {
            "JavaScript" => Some(Lang {
                name: name.into(),
                lang: tree_sitter_javascript::language().into(),
            }),
            _ => None
        }
    }
}
