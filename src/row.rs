//! row: Tools for inserting and removing characters
//!
//! This contains the [Row] struct. Occasionally, you might
//! require some row-specific information such as how it looks when rendered,
//! or where the word boundaries in it are.

use crate::document::FileInfo;
use crate::event::{Error, Result, Status};
use crate::st;
use crate::utils::{raw_indices, width, width_char, BoundedRange};

/// A struct that contains all the basic tools necessary to manage rows in a document
#[derive(Debug, PartialEq, Clone)]
pub struct Row {
    /// All the characters within the row
    pub text: Vec<char>,
    /// Corresponding display widths for each character
    pub indices: Vec<usize>,
    /// A tool for determining if the row has been edited
    /// ```
    /// use kaolinite::row::Row;
    /// let mut row = Row::new("Hello");
    /// assert_eq!(row.modified, false);
    /// row.insert(5, ", world!");
    /// assert_eq!(row.modified, true);
    /// row.modified = false;
    /// assert_eq!(row.modified, false);
    /// ```
    /// This is ideal for optimisation
    pub modified: bool,
    /// Holds a reference to file information
    pub info: *mut FileInfo,
}

impl Row {
    /// Create a new row from raw text
    #[cfg(not(tarpaulin_include))]
    pub fn new<S: Into<String>>(raw: S) -> Self {
        let raw = raw.into();
        let text: Vec<char> = raw.chars().collect();
        Self {
            indices: Row::raw_to_indices(&text, 4),
            text,
            modified: false,
            info: std::ptr::null_mut(),
        }
    }

    /// This method provides a neat way to link a row to a file info
    /// You usually don't need to use this method yourself
    /// It is used by Document to link it's [`FileInfo`](crate::document::FileInfo) struct
    /// to each Row for configuration purposes
    pub fn link(mut self, link: *mut FileInfo) -> Self {
        self.info = link;
        let tabs = self.get_tab_width();
        if tabs != 4 {
            self.indices = Row::raw_to_indices(&self.text, tabs);
        }
        self
    }

    /// Insert text at a position
    /// # Errors
    /// Will return `Err` if `start` is out of range of the row
    pub fn insert<S: Into<String>>(&mut self, start: usize, text: S) -> Result<Status> {
        if start > self.width() {
            return Err(Error::OutOfRange);
        }
        let text = text.into();
        self.text.splice(start..start, text.chars());
        let tabs = self.get_tab_width();
        self.indices = Row::raw_to_indices(&self.text, tabs);
        self.modified = true;
        Ok(Status::None)
    }

    /// Remove text in a range
    ///
    /// This takes in an inclusive or exclusive range: `..` and `..=` only.
    /// # Errors
    /// Will return `Err` if `range` is out of range of the row
    #[allow(clippy::needless_pass_by_value)]
    pub fn remove<R>(&mut self, range: R) -> Result<Status>
    where
        R: BoundedRange,
    {
        let (start, end) = (range.first(), range.last());
        if start > self.width() {
            return Err(Error::OutOfRange);
        }
        let shift = self.indices[end] - self.indices[start];
        self.text.splice(start..end, []);
        self.indices.splice(start..end, []);
        self.indices
            .iter_mut()
            .skip(start)
            .for_each(|i| *i -= shift);
        self.modified = true;
        Ok(Status::None)
    }

    /// Splits this row into two separate rows
    /// # Errors
    /// Will return `Err` if `idx` is out of range of the row
    pub fn split(&self, idx: usize) -> Result<(Row, Row)> {
        let left = Row {
            text: self.text.get(..idx).ok_or(Error::OutOfRange)?.to_vec(),
            indices: self.indices.get(..=idx).ok_or(Error::OutOfRange)?.to_vec(),
            info: self.info,
            modified: true,
        };
        let mut right = Row {
            text: self.text.get(idx..).ok_or(Error::OutOfRange)?.to_vec(),
            indices: self.indices.get(idx..).ok_or(Error::OutOfRange)?.to_vec(),
            info: self.info,
            modified: false,
        };
        // Shift down
        let shift = *right.indices.first().unwrap_or(&0);
        right.indices.iter_mut().for_each(|i| *i -= shift);
        Ok((left, right))
    }

    /// Joins this row with another row
    pub fn splice(&mut self, mut row: Row) -> Row {
        let mut indices = self.indices.clone();
        let shift = *self.indices.last().unwrap_or(&0);
        row.indices.remove(0);
        row.indices.iter_mut().for_each(|i| *i += shift);
        indices.append(&mut row.indices);
        let mut text = self.text.clone();
        text.append(&mut row.text);
        Row {
            indices,
            text,
            modified: true,
            info: self.info,
        }
    }

    /// Retrieve the indices of word boundaries
    /// ```
    /// // Opening a file,
    /// use kaolinite::document::Document;
    /// let mut doc = Document::new((10, 10));
    /// // Imagine if test.txt were `The quick brown fox`
    /// doc.open("examples/test.txt").expect("Failed to open file");
    /// // This would get the word boundaries of the first row: [0, 4, 10, 16, 19]
    /// println!("{:?}", doc.row(0).unwrap().words());
    /// ```
    #[must_use]
    pub fn words(&self) -> Vec<usize> {
        crate::utils::words(self)
    }

    /// Find the next word in this row from the character index
    #[must_use]
    pub fn next_word_forth(&self, loc: usize) -> usize {
        let bounds = self.words();
        let mut last = *bounds.last().unwrap_or(&0);
        for bound in bounds.iter().rev() {
            if bound <= &loc {
                return last;
            }
            last = *bound;
        }
        unreachable!()
    }

    /// Find the previous word in this row from the character index
    #[must_use]
    pub fn next_word_back(&self, loc: usize) -> usize {
        let bounds = self.words();
        let mut last = 0;
        for bound in &bounds {
            if bound >= &loc {
                return last;
            }
            last = *bound;
        }
        *bounds.last().unwrap_or(&0)
    }

    /// Render part of the row
    /// When trying to render X axis offset, this is the ideal function to use
    /// ```ignore
    /// "He好llo好" // 0..
    /// "e好llo好"  // 1..
    /// "好llo好"   // 2..
    /// " llo好"    // 3..
    /// "llo好"     // 4..
    /// "lo好"      // 5..
    /// "o好"       // 6..
    /// "好"        // 7..
    /// " "         // 8..
    /// ""          // 9..
    /// ```
    /// This also handles double width characters by inserting whitespace when half
    /// of the character is off the screen
    #[must_use]
    pub fn render(&self, range: std::ops::RangeFrom<usize>) -> String {
        let mut start = range.start;
        // Render the row
        let text = self.render_raw();
        let tab_width = self.get_tab_width();
        // Return an empty string if start is out of range
        if start >= width(&text, tab_width) {
            return st!("");
        }
        // Obtain the character indices
        let ind = raw_indices(&text, &self.indices, tab_width);
        // Shift the cut point forward until on a character boundary
        let space = !ind.contains_key(&start);
        while !ind.contains_key(&start) {
            start += 1;
        }
        // Perform cut and format
        let text = text.replace("\t", &" ".repeat(tab_width));
        format!("{}{}", if space { " " } else { "" }, &text[ind[&start]..])
    }

    /// Render the entire row, with tabs converted into spaces
    #[must_use]
    pub fn render_full(&self) -> String {
        // Retrieve tab width
        let tabs = self.get_tab_width();
        self.text.iter().fold(st!(""), |a, x| {
            format!(
                "{}{}",
                a,
                if x == &'\t' {
                    " ".repeat(tabs)
                } else {
                    x.to_string()
                }
            )
        })
    }

    /// Render this row as is, with no tab interference
    #[must_use]
    pub fn render_raw(&self) -> String {
        self.text.iter().fold(st!(""), |a, x| format!("{}{}", a, x))
    }

    /// Find the character length of this row
    #[must_use]
    pub fn len(&self) -> usize {
        self.text.len()
    }

    /// Determine if the row is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Find the display width of this row
    #[must_use]
    pub fn width(&self) -> usize {
        *self.indices.last().unwrap_or(&0)
    }

    /// Calculate the character pointer from a display index
    #[must_use]
    pub fn get_char_ptr(&self, x: usize) -> usize {
        // Handle large values of x
        if x >= self.width() {
            return self.len();
        }
        // Calculate the character width
        self.indices.iter().position(|i| &x == i).unwrap_or(0)
    }

    /// Find the widths of the characters in raw text
    fn raw_to_indices(text: &[char], tab_width: usize) -> Vec<usize> {
        let mut data = vec![&'\x00'];
        data.splice(1.., text);
        data.iter()
            .map(|c| width_char(**c, tab_width))
            .scan(0, |a, x| {
                *a += x;
                Some(*a)
            })
            .collect()
    }

    /// Retrieve the tab width from the document info
    #[must_use]
    pub fn get_tab_width(&self) -> usize {
        if self.info.is_null() {
            4
        } else {
            unsafe { &*self.info }.tab_width
        }
    }
}
