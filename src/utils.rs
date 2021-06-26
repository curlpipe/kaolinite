use lazy_regex::{Lazy, Regex, lazy_regex};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};
use std::collections::HashMap;

/// Regex that matches all line delimeters, used for splitting lines
pub static LINE_ENDING_SPLITTER: Lazy<Regex> = lazy_regex!("(\\r\\n|\\n)");

/// String helper macro
#[macro_export] macro_rules! st {
    ($value:expr) => { $value.to_string() };
}

/// A struct that holds positions
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Loc {
    pub x: usize,
    pub y: usize,
}

/// A struct that holds positions
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Size {
    pub w: usize,
    pub h: usize,
}

impl Into<Size> for (usize, usize) {
    fn into(self) -> Size {
        let (w, h) = self;
        Size { w, h }
    }
}

trait Width {
    fn width(&self) -> usize;
}

impl Width for String {
    /// Work out the width that will be displayed on the terminal
    #[cfg(not(tarpaulin_include))]
    fn width(&self) -> usize {
        UnicodeWidthStr::width(self.as_str())
    }
}

impl Width for char {
    /// Work out the width that will be displayed on the terminal
    #[cfg(not(tarpaulin_include))]
    fn width(&self) -> usize {
        UnicodeWidthChar::width(*self).unwrap_or(0)
    }
}

/// Generate a look up table between the raw and display indices
pub fn raw_indices(s: &str, i: &[usize]) -> HashMap<usize, usize> {
    let mut raw = 0;
    let mut indices = HashMap::new();
    indices.insert(0, 0);
    for (c, ch) in s.chars().enumerate() {
        raw += ch.len_utf8();
        indices.insert(i[c + 1], raw);
    }
    indices
}
