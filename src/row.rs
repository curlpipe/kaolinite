//! row: Tools for inserting and removing characters
//!
//! This contains the [Row] struct. Occasionally, you might
//! require some row-specific information such as how it looks when rendered,
//! or where the word boundaries in it are.

use crate::document::FileInfo;
use crate::event::{Error, Result, Status};
use crate::st;
use crate::utils::raw_indices;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

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
        self.indices = Row::raw_to_indices(&self.text, tabs);
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
        // TODO: Optimise
        let tabs = self.get_tab_width();
        self.indices = Row::raw_to_indices(&self.text, tabs);
        self.modified = true;
        Ok(Status::None)
    }

    /// Remove text in a range
    /// # Errors
    /// Will return `Err` if `range` is out of range of the row
    pub fn remove<R>(&mut self, range: R) -> Result<Status>
    where
        R: std::ops::RangeBounds<usize>,
    {
        if let std::ops::Bound::Included(start) = range.start_bound() {
            if start > &self.width() {
                return Err(Error::OutOfRange);
            }
        }
        self.text.splice(range, []);
        // TODO: Optimise
        let tabs = self.get_tab_width();
        self.indices = Row::raw_to_indices(&self.text, tabs);
        self.modified = true;
        Ok(Status::None)
    }

    /// Splits this row into two separate rows
    /// # Errors
    /// Will return `Err` if `idx` is out of range of the row
    pub fn split(&self, idx: usize) -> Result<(Row, Row)> {
        let left = self
            .text
            .get(..idx)
            .ok_or(Error::OutOfRange)?
            .iter()
            .fold(st!(""), |a, x| format!("{}{}", a, x));
        let right = self
            .text
            .get(idx..)
            .ok_or(Error::OutOfRange)?
            .iter()
            .fold(st!(""), |a, x| format!("{}{}", a, x));
        let mut left = Row::new(left).link(self.info);
        left.modified = true;
        let right = Row::new(right).link(self.info);
        Ok((left, right))
    }

    /// Joins this row with another row
    pub fn splice(&mut self, mut row: Row) -> Row {
        let mut text = self.text.clone();
        text.append(&mut row.text);
        let indices = Row::raw_to_indices(&text, self.get_tab_width());
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
    /// let mut doc = Document::new();
    /// // Imagine if test.txt were `The quick brown fox`
    /// doc.open("test.txt").expect("Failed to open file");
    /// // This would get the word boundaries of the first row: [0, 4, 10, 16, 19]
    /// println!("{:?}", doc.row(0).words());
    /// ```
    #[must_use]
    pub fn words(&self) -> Vec<usize> {
        crate::utils::words(self)
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
        let text = self.render_full();
        // Return an empty string if start is out of range
        if start >= text.width() {
            return st!("");
        }
        // Obtain the character indices
        let ind = raw_indices(&text, &self.indices);
        // Shift the cut point forward until on a character boundary
        let space = !ind.contains_key(&start);
        while !ind.contains_key(&start) {
            start += 1;
        }
        // Perform cut and format
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
        self.render_full().width()
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
            .map(|c| {
                if c == &&'\t' {
                    tab_width
                } else {
                    c.width().unwrap_or(0)
                }
            })
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
