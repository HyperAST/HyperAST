use std::{
    cell::RefCell,
    fmt::Debug,
    hash::Hash,
    ops::Range,
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
};

use lazy_static::lazy_static;

use super::generic_text_buffer::{AsText, TextBuffer};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct EditAwareString {
    pub(crate) id: u64,
    pub(crate) generation: u64,
    pub(crate) string: String,
    #[serde(skip)]
    #[serde(default = "default_bool")]
    pub(crate) reset: AtomicBool,
    #[serde(skip)]
    pub(crate) edit: RefCell<Option<InputEdit>>,
}

impl Debug for EditAwareString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EditAwareString")
            .field("string", &self.string)
            .finish()
    }
}

impl Into<String> for EditAwareString {
    fn into(self) -> String {
        self.string
    }
}

fn default_bool() -> AtomicBool {
    AtomicBool::new(false)
}

impl Clone for EditAwareString {
    fn clone(&self) -> Self {
        let reset = &self.reset.load(Ordering::Relaxed);
        Self {
            id: self.id,
            generation: self.generation,
            string: self.string.clone(),
            reset: reset.clone().into(),
            edit: self.edit.clone(),
        }
    }
}

// impl PartialEq for EditAwareString {
// }

impl Hash for EditAwareString {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.generation.hash(state);
    }
}

lazy_static! {
    static ref EAS_ID_COUNTER: AtomicU64 = AtomicU64::new(0);
}

impl From<String> for EditAwareString {
    fn from(string: String) -> Self {
        let id = EAS_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        Self {
            id,
            generation: 0,
            string,
            reset: AtomicBool::new(false),
            edit: Default::default(),
        }
    }
}

impl<'a> Into<&'a str> for &'a EditAwareString {
    fn into(self) -> &'a str {
        &self.string
    }
}

impl AsText for EditAwareString {
    fn text(&self) -> &str {
        &self.string
    }
}

impl EditAwareString {
    fn id() {}
}

impl TextBuffer for EditAwareString {
    // type Ref<'a> = &'a EditAwareString;
    type Ref = EditAwareString;

    fn is_mutable(&self) -> bool {
        true
    }

    fn as_reference(&self) -> &EditAwareString {
        &self
    }

    fn as_str(&self) -> &str {
        &self.string
    }

    fn insert_text(&mut self, text: &str, char_index: usize) -> usize {
        // Get the byte index from the character index
        let byte_idx = self.byte_index_from_char_index(char_index);

        let edit = Edit {
            position: byte_idx,
            deleted_length: 0,
            inserted_text: text.as_bytes(),
        };

        if let Some(self_edit) = self.edit.get_mut() {
            assert_eq!(self_edit.start_byte, edit.position as u32);
            let input: &mut Vec<u8> = unsafe { self.string.as_mut_vec() };
            let start_byte = edit.position;
            *self_edit = {
                let old_end_byte = self_edit.old_end_byte as usize;
                let new_end_byte = edit.position + edit.inserted_text.len();
                let start_position = self_edit.start_position;
                let old_end_position = self_edit.old_end_position;
                input.splice(start_byte..start_byte, edit.inserted_text.iter().cloned());
                let new_end_position = position_for_offset(input, new_end_byte);
                let edit = InputEdit {
                    start_byte: start_byte as u32,
                    old_end_byte: old_end_byte as u32,
                    new_end_byte: new_end_byte as u32,
                    start_position,
                    old_end_position,
                    new_end_position,
                };
                edit
            }
            .into();
        } else {
            self.generation += 1;
            self.edit = Some(process_edit(unsafe { self.string.as_mut_vec() }, &edit)).into();
        }

        // // Then insert the string
        // self.string.insert_str(byte_idx, text);

        text.chars().count()
    }

    fn delete_char_range(&mut self, char_range: Range<usize>) {
        assert!(char_range.start <= char_range.end);

        // Get both byte indices
        let byte_start = self.byte_index_from_char_index(char_range.start);
        let byte_end = self.byte_index_from_char_index(char_range.end);

        self.generation += 1;
        let edit = Edit {
            position: byte_start,
            deleted_length: byte_end - byte_start,
            inserted_text: &[],
        };

        assert!(self.edit.get_mut().is_none());
        self.edit = Some(process_edit(unsafe { self.string.as_mut_vec() }, &edit)).into();

        // // Then drain all characters within this range
        // self.string.drain(byte_start..byte_end);
    }

    fn clear(&mut self) {
        self.generation += 1;
        self.reset = true.into();
        self.string.clear();
    }

    fn replace(&mut self, text: &str) {
        self.generation += 1;
        self.reset = true.into();
        *self = text.to_owned().into();
    }

    fn take(&mut self) -> String {
        self.generation += 1;
        self.reset = true.into();
        std::mem::take(&mut self.string)
    }
}

pub(crate) fn process_edit(input: &mut Vec<u8>, edit: &Edit<'_>) -> InputEdit {
    let start_byte = edit.position;
    let old_end_byte = edit.position + edit.deleted_length;
    let new_end_byte = edit.position + edit.inserted_text.len();
    let start_position = position_for_offset(input, start_byte);
    let old_end_position = position_for_offset(input, old_end_byte);
    input.splice(start_byte..old_end_byte, edit.inserted_text.iter().cloned());
    let new_end_position = position_for_offset(input, new_end_byte);
    let edit = InputEdit {
        start_byte: start_byte as u32,
        old_end_byte: old_end_byte as u32,
        new_end_byte: new_end_byte as u32,
        start_position,
        old_end_position,
        new_end_position,
    };
    edit
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct InputEdit {
    start_byte: u32,
    old_end_byte: u32,
    new_end_byte: u32,
    start_position: Point,
    old_end_position: Point,
    new_end_position: Point,
}

impl Into<tree_sitter::InputEdit> for InputEdit {
    fn into(self) -> tree_sitter::InputEdit {
        tree_sitter::InputEdit::new(
            self.start_byte,
            self.old_end_byte,
            self.new_end_byte,
            &self.start_position.into(),
            &self.old_end_position.into(),
            &self.new_end_position.into(),
        )
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Point {
    row: u32,
    column: u32,
}

impl Into<tree_sitter::Point> for Point {
    fn into(self) -> tree_sitter::Point {
        tree_sitter::Point::new(self.row, self.column)
    }
}

#[derive(Debug)]
pub struct Edit<'a> {
    pub position: usize,
    pub deleted_length: usize,
    pub inserted_text: &'a [u8],
}
fn position_for_offset(input: &Vec<u8>, offset: usize) -> Point {
    let mut row = 0;
    let mut column = 0;
    for c in &input[0..offset] {
        if *c as char == '\n' {
            row += 1;
            column = 0;
        } else {
            column += 1;
        }
    }
    Point { row, column }
}
