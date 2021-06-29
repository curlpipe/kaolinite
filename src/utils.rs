use crate::row::Row;
use std::collections::HashMap;
use unicode_width::UnicodeWidthChar;

/// Whitespace character array
const WHITESPACE: [char; 2] = [' ', '\t'];

/// String helper macro
#[macro_export]
macro_rules! st {
    ($value:expr) => {
        $value.to_string()
    };
}

/// Lazy regex creation
#[macro_export]
macro_rules! regex {
    ($re:literal $(,)?) => {{
        static RE: once_cell::sync::OnceCell<regex::Regex> = once_cell::sync::OnceCell::new();
        RE.get_or_init(|| regex::Regex::new($re).unwrap())
    }};
}

/// A struct that holds positions
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Loc {
    pub x: usize,
    pub y: usize,
}

impl From<(usize, usize)> for Loc {
    fn from(loc: (usize, usize)) -> Loc {
        let (x, y) = loc;
        Loc { x, y }
    }
}

/// A struct that holds size
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Size {
    pub w: usize,
    pub h: usize,
}

impl From<(usize, usize)> for Size {
    fn from(size: (usize, usize)) -> Size {
        let (w, h) = size;
        Size { w, h }
    }
}

/// Generate a look up table between the raw and display indices
#[must_use]
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

/// Retrieve the indices of word boundaries
#[must_use]
pub fn words(row: &Row) -> Vec<usize> {
    // Gather information and set up algorithm
    let tabs = row.get_tab_width();
    let mut result = vec![];
    let mut dis = 0;
    let mut chr = 0;
    let mut pad = true;
    // While still inside the row
    while chr < row.text.len() {
        let c = row.text[chr];
        match c {
            // Move forward through all the spaces
            ' ' => dis += 1,
            '\t' => {
                // If we haven't encountered text yet
                if pad {
                    // Make this a word boundary
                    result.push(dis);
                }
                // Move forward
                dis += tabs;
            }
            _ => {
                // Set the marker to false, as we're encountering text
                pad = false;
                // Set this as a word boundary
                result.push(dis);
                // Skip through text, end when we find whitespace or the end of the row
                while chr < row.text.len() && !WHITESPACE.contains(&row.text[chr]) {
                    dis += row.text[chr].width().unwrap_or(0);
                    chr += 1;
                }
                // Deal with next lot of whitespace or exit if at the end of the row
                continue;
            }
        }
        // Advance and continue
        chr += 1;
    }
    // Add on the last point on the row as a word boundary
    result.push(row.width());
    result
}
