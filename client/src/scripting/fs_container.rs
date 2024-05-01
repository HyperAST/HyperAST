use rhai::Dynamic;

use super::{estimate::Estimate, finalize::Finalize};

#[derive(Clone)]
pub(super) struct FsContainer<T> {
    name: String,
    pub(super) content: T,
}

impl<T> FsContainer<T> {
    pub(super) fn new(name: String, content: T) -> Self {
        Self { name, content }
    }
}

impl Finalize for FsContainer<Dynamic> {
    type Output = rhai::Map;
    fn finalize(self) -> Self::Output {
        let mut acc = rhai::Map::new();

        self.finalize_aux(&mut acc, "");

        acc
    }
}
impl FsContainer<Dynamic> {
    fn finalize_aux(self, acc: &mut rhai::Map, path: &str) {
        if self.content.is_array() {
            let path = format!("{}{}/", path, self.name);
            let arr: rhai::Array = self.content.cast();
            for x in arr {
                if x.is::<FsContainer<Dynamic>>() {
                    let x: FsContainer<Dynamic> = x.cast();
                    x.finalize_aux(acc, &path);
                } else {
                    acc.insert(path.to_string().into(), x);
                }
            }
        } else {
            let path = format!("{}{}", path, self.name);
            acc.insert(path.into(), self.content);
        }
    }
}
