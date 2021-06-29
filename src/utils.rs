use lazy_regex::{Lazy, Regex, lazy_regex};
use std::collections::HashMap;
use unicode_width::UnicodeWidthChar;
use crate::row::Row;

/// Whitespace character array
const WHITESPACE: [char; 2] = [' ', '\t'];

/// Regex that matches all line delimeters, used for splitting lines
pub static LINE_ENDING_SPLITTER: Lazy<Regex> = lazy_regex!("(\\r\\n|\\n)");
pub static TAB_DETECTION: Lazy<Regex> = lazy_regex!("(?ms)(^\\t)");

/// String helper macro
#[macro_export] macro_rules! st {
    ($value:expr) => { $value.to_string() };
}

/// A struct that holds positions
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Loc {
    pub x: usize,
    pub y: usize,
}

impl Into<Loc> for (usize, usize) {
    fn into(self) -> Loc {
        let (x, y) = self;
        Loc { x, y }
    }
}

/// A struct that holds positions
#[derive(Clone, Copy, Debug, Default, PartialEq)]
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

/// Retrieve the indices of word boundaries
pub fn words(row: &Row) -> Vec<usize> {
    // Gather information and set up algorithm
    let tabs = row.get_tab_width();
    let mut result = vec![];
    let mut dis = 0;
    let mut chr = 0;
    let mut pad = true;
    // While still inside the row
    while chr < row.text.len() {
        match row.text[chr] {
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
