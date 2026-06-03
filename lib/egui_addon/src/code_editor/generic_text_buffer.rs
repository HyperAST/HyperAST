use std::ops::Range;

pub trait AsText {
    fn text(&self) -> &str;
}

impl AsText for String {
    fn text(&self) -> &str {
        &self
    }
}

impl AsText for str {
    fn text(&self) -> &str {
        &self
    }
}

/// Trait constraining what types [`crate::TextEdit`] may use as
/// an underlying buffer.
///
/// Most likely you will use a [`String`] which implements [`TextBuffer`].
pub trait TextBuffer {
    /// Main difference wrt. egui's [`egui::TextBuffer`]
    type Ref: ?Sized + AsText;

    /// Can this text be edited?
    fn is_mutable(&self) -> bool;

    fn as_reference(&self) -> &Self::Ref;

    /// Returns this buffer as a `str`.
    fn as_str(&self) -> &str {
        self.as_reference().text()
    }

    /// Reads the given character range.
    fn char_range(&self, char_range: Range<usize>) -> &str {
        assert!(char_range.start <= char_range.end);
        let start_byte = self.byte_index_from_char_index(char_range.start);
        let end_byte = self.byte_index_from_char_index(char_range.end);
        &self.as_str()[start_byte..end_byte]
    }

    fn byte_index_from_char_index(&self, char_index: usize) -> usize {
        byte_index_from_char_index(self.as_str(), char_index)
    }

    /// Inserts text `text` into this buffer at character index `char_index`.
    ///
    /// # Notes
    /// `char_index` is a *character index*, not a byte index.
    ///
    /// # Return
    /// Returns how many *characters* were successfully inserted
    fn insert_text(&mut self, text: &str, char_index: usize) -> usize;

    /// Deletes a range of text `char_range` from this buffer.
    ///
    /// # Notes
    /// `char_range` is a *character range*, not a byte range.
    fn delete_char_range(&mut self, char_range: Range<usize>);

    /// Clears all characters in this buffer
    fn clear(&mut self) {
        self.delete_char_range(0..self.as_str().len());
    }

    /// Replaces all contents of this string with `text`
    fn replace(&mut self, text: &str) {
        self.clear();
        self.insert_text(text, 0);
    }

    /// replace a range of chars
    /// default to insert_text(text,char_range.start());delete_char_range(char_range)
    fn replace_range(&mut self, text: &str, char_range: Range<usize>) -> usize {
        let char_index = char_range.start;
        self.delete_char_range(char_range);
        self.insert_text(text, char_index)
    }

    /// Clears all characters in this buffer and returns a string of the contents.
    fn take(&mut self) -> String {
        let s = self.as_str().to_owned();
        self.clear();
        s
    }
}

impl TextBuffer for String {
    type Ref = String;
    fn is_mutable(&self) -> bool {
        true
    }

    fn as_reference(&self) -> &String {
        &self
    }
    fn insert_text(&mut self, text: &str, char_index: usize) -> usize {
        // Get the byte index from the character index
        let byte_idx = self.byte_index_from_char_index(char_index);

        // Then insert the string
        self.insert_str(byte_idx, text);

        text.chars().count()
    }

    fn delete_char_range(&mut self, char_range: Range<usize>) {
        assert!(char_range.start <= char_range.end);

        // Get both byte indices
        let byte_start = self.byte_index_from_char_index(char_range.start);
        let byte_end = self.byte_index_from_char_index(char_range.end);

        // Then drain all characters within this range
        self.drain(byte_start..byte_end);
    }

    fn clear(&mut self) {
        self.clear();
    }

    fn replace(&mut self, text: &str) {
        *self = text.to_owned();
    }

    fn take(&mut self) -> String {
        std::mem::take(self)
    }
}

/// Immutable view of a `&str`!
impl<'a> TextBuffer for &'a str {
    type Ref = str;

    fn is_mutable(&self) -> bool {
        false
    }

    fn as_reference(&self) -> &str {
        self
    }

    fn insert_text(&mut self, _text: &str, _ch_idx: usize) -> usize {
        0
    }

    fn delete_char_range(&mut self, _ch_range: Range<usize>) {}
}

pub fn byte_index_from_char_index(s: &str, char_index: usize) -> usize {
    for (ci, (bi, _)) in s.char_indices().enumerate() {
        if ci == char_index {
            return bi;
        }
    }
    s.len()
}

pub fn char_index_from_byte_index(s: &str, byte_index: usize) -> usize {
    let mut ci = 0;
    let mut i = 0;
    let mut it = s.chars();
    while i < byte_index {
        let Some(c) = it.next() else {
            break;
        };
        let len = c.len_utf8();
        i += len;
        ci += 1;
    }
    ci
}

pub fn char_index_from_byte_index2(
    s: &str,
    byte_index1: usize,
    byte_index2: usize,
) -> (usize, usize) {
    let mut ci = 0;
    let mut i = 0;
    let mut it = s.chars();
    while i < byte_index1 {
        let Some(c) = it.next() else {
            break;
        };
        let len = c.len_utf8();
        i += len;
        ci += 1;
    }
    let ci0 = ci;
    while i < byte_index2 {
        let Some(c) = it.next() else {
            break;
        };
        let len = c.len_utf8();
        i += len;
        ci += 1;
    }
    (ci0, ci)
}
