use super::types::Commit;
use egui_addon::code_editor::{self, generic_text_buffer::byte_index_from_char_index};
use lazy_static::lazy_static;
use std::{ops::Range, sync::Arc};

lazy_static! {
    static ref COMMIT_STRS: Arc<std::sync::Mutex<BorrowFrameCache<String, ComputeCommitStr>>> = {
        // let mut map = HashMap::new();
        // map.insert("James", vec!["user", "admin"]);
        // map.insert("Jim", vec!["user"]);
        // map
        Default::default()
    };
}

pub struct BorrowFrameCache<Value, Computer> {
    pub(crate) generation: u32,
    pub(crate) computer: Computer,
    pub(crate) cache: nohash_hasher::IntMap<u64, (u32, Value)>,
}

impl<Value, Computer> Default for BorrowFrameCache<Value, Computer>
where
    Computer: Default,
{
    fn default() -> Self {
        Self::new(Computer::default())
    }
}

impl<Value, Computer> BorrowFrameCache<Value, Computer> {
    pub fn new(computer: Computer) -> Self {
        Self {
            generation: 0,
            computer,
            cache: Default::default(),
        }
    }

    /// Must be called once per frame to clear the cache.
    pub fn evice_cache(&mut self) {
        let current_generation = self.generation;
        self.cache.retain(|_key, cached| {
            cached.0 == current_generation // only keep those that were used this frame
        });
        self.generation = self.generation.wrapping_add(1);
    }
}

impl<Value: 'static + Send + Sync, Computer: 'static + Send + Sync> egui::util::cache::CacheTrait
    for BorrowFrameCache<Value, Computer>
{
    fn update(&mut self) {
        self.evice_cache();
    }

    fn len(&self) -> usize {
        self.cache.len()
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl<Value, Computer> BorrowFrameCache<Value, Computer> {
    /// Get from cache (if the same key was used last frame)
    /// or recompute and store in the cache.
    pub fn get<Key>(&mut self, key: Key) -> &Value
    where
        Key: Copy + std::hash::Hash,
        Computer: egui::util::cache::ComputerMut<Key, Value>,
    {
        let hash = egui::util::hash(key);

        match self.cache.entry(hash) {
            std::collections::hash_map::Entry::Occupied(entry) => {
                let cached = entry.into_mut();
                cached.0 = self.generation;
                &cached.1
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                let value = self.computer.compute(key);
                &entry.insert((self.generation, value)).1
            }
        }
    }
    pub fn get2<Key, Payload>(&mut self, key: Key, payload: Payload) -> &Value
    where
        Key: Copy + std::hash::Hash,
        Computer: egui::util::cache::ComputerMut<(Key, Payload), Value>,
    {
        let hash = egui::util::hash(key);

        match self.cache.entry(hash) {
            std::collections::hash_map::Entry::Occupied(entry) => {
                let cached = entry.into_mut();
                cached.0 = self.generation;
                &cached.1
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                let value = self.computer.compute((key, payload));
                &entry.insert((self.generation, value)).1
            }
        }
    }
    /// WARN panic if absent value
    pub fn access<Key>(&self, key: Key) -> &Value
    where
        Key: std::hash::Hash,
        Computer: egui::util::cache::ComputerMut<Key, Value>,
    {
        let hash = egui::util::hash(&key);
        &self.cache.get(&hash).unwrap().1
    }
    pub fn remove<Key>(&mut self, key: Key) -> Option<Value>
    where
        Key: std::hash::Hash,
    {
        let hash = egui::util::hash(&key);
        self.cache.remove(&hash).map(|x|x.1)
    }
}

#[derive(Default)]
pub struct ComputeCommitStr {
    // map:
}

impl egui::util::cache::ComputerMut<(&str, &Commit), String> for ComputeCommitStr {
    fn compute(&mut self, (forge, commit): (&str, &Commit)) -> String {
        format!(
            "{}/{}/{}/{}",
            forge, commit.repo.user, commit.repo.name, commit.id
        )
    }
}

#[derive(Hash)]
struct AAA {}

#[derive(Default)]
pub struct ComputeCommitStr2 {
    // map:
}

impl egui::util::cache::ComputerMut<(&str, &Commit, &AAA), String> for ComputeCommitStr2 {
    fn compute(&mut self, (forge, commit, _): (&str, &Commit, &AAA)) -> String {
        format!(
            "{}/{}/{}/{}",
            forge, commit.repo.user, commit.repo.name, commit.id
        )
    }
}

pub struct CommitTextBuffer<'a, 'b, 'c> {
    pub(crate) commit: &'a mut Commit,
    pub(crate) forge: String,
    pub(crate) str: &'b mut std::sync::MutexGuard<'c, BorrowFrameCache<String, ComputeCommitStr>>,
}

impl<'a, 'b, 'c> CommitTextBuffer<'a, 'b, 'c> {
    pub(crate) fn new(
        commit: &'a mut Commit,
        forge: String,
        str: &'b mut std::sync::MutexGuard<'c, BorrowFrameCache<String, ComputeCommitStr>>,
    ) -> Self {
        str.get((&forge, commit));
        Self { commit, forge, str }
    }
}

impl<'a, 'b, 'c> super::code_editor::generic_text_buffer::TextBuffer
    for CommitTextBuffer<'a, 'b, 'c>
{
    type Ref = String;
    fn is_mutable(&self) -> bool {
        true
    }
    fn as_reference(&self) -> &Self::Ref {
        self.str.access((&self.forge, &self.commit))
    }

    fn insert_text(&mut self, text: &str, char_index: usize) -> usize {
        // Get the byte index from the character index
        let byte_idx = self.byte_index_from_char_index(char_index);
        if text.starts_with("https://") {
            let text = &text["https://".len()..];
            let split: Vec<_> = text.split("/").collect();
            if split[0] != "github.com" {
                // TODO launch an alert
                // wasm_rs_dbg::dbg!("only github.com is allowed");
                return 0;
            }
            if split.len() == 5 {
                self.commit.repo.user = split[1].to_string();
                self.commit.repo.name = split[2].to_string();
                assert_eq!("commit", split[3].to_string());
                self.commit.id = split[4].to_string();
            }
            // wasm_rs_dbg::dbg!(&self.commit);
            self.str.get((&self.forge, &self.commit));
            return text.chars().count();
        }

        let mut t = self.str.get((&self.forge, &self.commit)).to_string();

        t.insert_str(byte_idx, text);
        let split: Vec<_> = t.split("/").collect();
        if split[0] != "github.com" {
            // TODO launch an alert
            // wasm_rs_dbg::dbg!("only github.com is allowed");
            return 0;
        }
        self.commit.repo.user = split[1].to_string();
        self.commit.repo.name = split[2].to_string();
        self.commit.id = split[3].to_string();

        self.str.get((&self.forge, &self.commit));

        text.chars().count()
    }

    fn delete_char_range(&mut self, _char_range: Range<usize>) {
        // assert!(char_range.start <= char_range.end);

        // // Get both byte indices
        // let byte_start = self.byte_index_from_char_index(char_range.start);
        // let byte_end = self.byte_index_from_char_index(char_range.end);

        // // Then drain all characters within this range
        // self.drain(byte_start..byte_end);
        // todo!()
        // WARN could produce unexpected functional results for the user
    }

    fn replace_range(&mut self, text: &str, char_range: Range<usize>) -> usize {
        // Get the byte index from the character index
        let byte_idx = self.byte_index_from_char_index(char_range.start);
        if text.starts_with("https://") {
            let text = &text["https://".len()..];
            let split: Vec<_> = text.split("/").collect();
            if split[0] != "github.com" {
                // TODO launch an alert
                // wasm_rs_dbg::dbg!(&split[0]);
                // wasm_rs_dbg::dbg!("only github.com is allowed");
                return 0;
            }
            if split.len() == 5 {
                self.commit.repo.user = split[1].to_string();
                self.commit.repo.name = split[2].to_string();
                assert_eq!("commit", split[3].to_string());
                self.commit.id = split[4].to_string();
            }
            // wasm_rs_dbg::dbg!(&split, &self.commit);
            self.str.get((&self.forge, &self.commit));
            return text.chars().count();
        }

        let mut t = self.str.get((&self.forge, &self.commit)).to_string();
        {
            let byte_start = byte_index_from_char_index(&t, char_range.start);
            let byte_end = byte_index_from_char_index(&t, char_range.end);
            t.drain(byte_start..byte_end);
        }
        t.insert_str(byte_idx, text);
        let split: Vec<_> = text.split("/").collect();
        if split[0] != "github.com" {
            // TODO launch an alert
            // wasm_rs_dbg::dbg!("only github.com is allowed");
            return 0;
        }
        self.commit.repo.user = split[1].to_string();
        self.commit.repo.name = split[2].to_string();
        self.commit.id = split[3].to_string();

        self.str.get((&self.forge, &self.commit));

        text.chars().count()
    }

    fn clear(&mut self) {
        // self.clear()
    }

    fn replace(&mut self, _text: &str) {
        // *self = text.to_owned();
    }

    fn take(&mut self) -> String {
        self.str.get((&self.forge, &self.commit)).to_string()
    }
}

pub(crate) fn show_commit_menu(ui: &mut egui::Ui, commit: &mut Commit) -> bool {
    let mut mutex_guard = COMMIT_STRS.lock().unwrap();
    let mut c = CommitTextBuffer::new(commit, "github.com".to_string(), &mut mutex_guard);
    let ml = code_editor::generic_text_edit::TextEdit::multiline(&mut c)
        // .margin(egui::Vec2::new(0.0, 0.0))
        // .desired_width(40.0)
        .id(ui.id().with("commit entry"))
        .show(ui);

    ml.response.changed()
}

pub(crate) fn show_commit_menu2(ui: &mut egui::Ui, commit: &mut Commit) -> bool {
    let c = ui.ctx().memory_mut(|mem| {
        let a = mem
            .caches
            .cache::<BorrowFrameCache<String, ComputeCommitStr>>()
            .get(("github.com", commit));
    });
    let c = ui.ctx().memory_mut(|mem| {
        let aaa =AAA{};
        let a = mem
            .caches
            .cache::<BorrowFrameCache<String, ComputeCommitStr2>>()
            .get(("github.com", commit, &aaa));
    });
    let mut mutex_guard = COMMIT_STRS.lock().unwrap();
    let mut c = CommitTextBuffer::new(commit, "github.com".to_string(), &mut mutex_guard);
    let ml = code_editor::generic_text_edit::TextEdit::multiline(&mut c)
        // .margin(egui::Vec2::new(0.0, 0.0))
        // .desired_width(40.0)
        .id(ui.id().with("commit entry"))
        .show(ui);

    ml.response.changed()
}
