use std::ops::Range;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Editing {
    text: String,
    caret: usize,
    anchor: usize,
    max_chars: Option<usize>,
}

fn previous_boundary(text: &str, index: usize) -> usize {
    text[..index]
        .char_indices()
        .next_back()
        .map_or(0, |(position, _)| position)
}

fn next_boundary(text: &str, index: usize) -> usize {
    text[index..]
        .char_indices()
        .nth(1)
        .map_or(text.len(), |(offset, _)| index + offset)
}

pub fn byte_to_utf16(text: &str, byte: usize) -> usize {
    text[..byte.min(text.len())].encode_utf16().count()
}

pub fn utf16_to_byte(text: &str, utf16: usize) -> usize {
    let mut units = 0;
    for (byte, character) in text.char_indices() {
        if units >= utf16 {
            return byte;
        }
        units += character.len_utf16();
    }
    text.len()
}

pub fn nearest_boundary(points: &[(usize, f32)], x: f32) -> usize {
    points
        .iter()
        .min_by(|a, b| (a.1 - x).abs().total_cmp(&(b.1 - x).abs()))
        .map_or(0, |point| point.0)
}

impl Editing {
    pub fn new(text: impl Into<String>) -> Self {
        let text = text.into();
        let caret = text.len();
        Self {
            text,
            caret,
            anchor: caret,
            max_chars: None,
        }
    }

    pub fn with_max_chars(mut self, max: usize) -> Self {
        self.max_chars = Some(max);
        self
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn caret(&self) -> usize {
        self.caret
    }

    pub fn char_count(&self) -> usize {
        self.text.chars().count()
    }

    pub fn max_chars(&self) -> Option<usize> {
        self.max_chars
    }

    pub fn selection(&self) -> Range<usize> {
        self.anchor.min(self.caret)..self.anchor.max(self.caret)
    }

    pub fn has_selection(&self) -> bool {
        self.anchor != self.caret
    }

    pub fn selected_text(&self) -> &str {
        &self.text[self.selection()]
    }

    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
        self.caret = self.text.len();
        self.anchor = self.caret;
    }

    pub fn place_caret(&mut self, index: usize, extend: bool) {
        let index = index.min(self.text.len());
        self.caret = index;
        if !extend {
            self.anchor = index;
        }
    }

    pub fn insert(&mut self, input: &str) -> bool {
        let range = self.selection();
        let removed = self.text[range.clone()].chars().count();
        let available = match self.max_chars {
            Some(max) => max.saturating_sub(self.char_count() - removed),
            None => usize::MAX,
        };
        let accepted: String = input.chars().take(available).collect();
        if accepted.is_empty() && !self.has_selection() {
            return false;
        }
        self.text.replace_range(range.clone(), &accepted);
        self.caret = range.start + accepted.len();
        self.anchor = self.caret;
        true
    }

    pub fn replace_utf16(&mut self, range: Range<usize>, input: &str) -> bool {
        let start = utf16_to_byte(&self.text, range.start);
        let end = utf16_to_byte(&self.text, range.end);
        self.anchor = start;
        self.caret = end;
        self.insert(input)
    }

    pub fn backspace(&mut self) -> bool {
        if self.has_selection() {
            return self.insert("");
        }
        if self.caret == 0 {
            return false;
        }
        let previous = previous_boundary(&self.text, self.caret);
        self.text.replace_range(previous..self.caret, "");
        self.caret = previous;
        self.anchor = previous;
        true
    }

    pub fn delete(&mut self) -> bool {
        if self.has_selection() {
            return self.insert("");
        }
        if self.caret >= self.text.len() {
            return false;
        }
        let following = next_boundary(&self.text, self.caret);
        self.text.replace_range(self.caret..following, "");
        true
    }

    pub fn move_left(&mut self, extend: bool) {
        let target = if !extend && self.has_selection() {
            self.selection().start
        } else {
            previous_boundary(&self.text, self.caret)
        };
        self.place_caret(target, extend);
    }

    pub fn move_right(&mut self, extend: bool) {
        let target = if !extend && self.has_selection() {
            self.selection().end
        } else {
            next_boundary(&self.text, self.caret)
        };
        self.place_caret(target, extend);
    }

    pub fn home(&mut self, extend: bool) {
        self.place_caret(0, extend);
    }

    pub fn end(&mut self, extend: bool) {
        self.place_caret(self.text.len(), extend);
    }

    pub fn select_all(&mut self) {
        self.anchor = 0;
        self.caret = self.text.len();
    }

    pub fn take_selection(&mut self) -> Option<String> {
        if !self.has_selection() {
            return None;
        }
        let taken = self.selected_text().to_string();
        self.insert("");
        Some(taken)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typing_appends_at_the_caret() {
        let mut editing = Editing::new("ab");
        assert!(editing.insert("c"));
        assert_eq!(editing.text(), "abc");
        assert_eq!(editing.caret(), 3);
    }

    #[test]
    fn backspace_removes_a_whole_multibyte_character() {
        let mut editing = Editing::new("aé");
        assert!(editing.backspace());
        assert_eq!(editing.text(), "a");
    }

    #[test]
    fn selection_is_replaced_by_typing() {
        let mut editing = Editing::new("hello");
        editing.place_caret(0, false);
        editing.place_caret(5, true);
        editing.insert("bye");
        assert_eq!(editing.text(), "bye");
        assert!(!editing.has_selection());
    }

    #[test]
    fn max_chars_truncates_input() {
        let mut editing = Editing::new("ab").with_max_chars(4);
        editing.insert("cdef");
        assert_eq!(editing.text(), "abcd");
        assert!(!editing.insert("x"));
    }

    #[test]
    fn arrows_collapse_a_selection_to_its_edge() {
        let mut editing = Editing::new("abc");
        editing.select_all();
        editing.move_left(false);
        assert_eq!(editing.caret(), 0);
        assert!(!editing.has_selection());
    }

    #[test]
    fn utf16_positions_round_trip() {
        let text = "a😀b";
        assert_eq!(byte_to_utf16(text, 5), 3);
        assert_eq!(utf16_to_byte(text, 3), 5);
    }

    #[test]
    fn nearest_boundary_picks_the_closest_position() {
        let points = [(0, 0.0), (1, 5.0), (2, 11.0)];
        assert_eq!(nearest_boundary(&points, 7.0), 1);
        assert_eq!(nearest_boundary(&points, 9.0), 2);
    }
}
