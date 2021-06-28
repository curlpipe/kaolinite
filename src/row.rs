use crate::event::{Result, Status};
use crate::document::FileInfo;
use crate::utils::raw_indices;
use unicode_width::{UnicodeWidthStr, UnicodeWidthChar};
use crate::st;

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
    /// It is used by Document to link it's FileInfo struct
    /// to each Row for configuration purposes
    pub fn link(mut self, link: *mut FileInfo) -> Self {
        self.info = link;
        let tabs = self.get_tab_width();
        self.indices = Row::raw_to_indices(&self.text, tabs);
        self
    }

    /// Insert text at a position
    pub fn insert<S: Into<String>>(&mut self, start: usize, text: S) -> Result<Status> {
        let text = text.into();
        self.text.splice(start..start, text.chars());
        // TODO: Optimise
        let tabs = self.get_tab_width();
        self.indices = Row::raw_to_indices(&self.text, tabs);
        self.modified = true;
        Ok(Status::None)
    }

    /// Remove text in a range
    pub fn remove(&mut self, range: std::ops::Range<usize>) -> Result<Status> {
        let (start, end) = (range.start, range.end);
        self.text.splice(start..end, []);
        // TODO: Optimise
        let tabs = self.get_tab_width();
        self.indices = Row::raw_to_indices(&self.text, tabs);
        self.modified = true;
        Ok(Status::None)
    }

    /// Retrieve the indices of word boundaries
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
    pub fn render(&self, range: std::ops::RangeFrom<usize>) -> String {
        let mut start = range.start;
        let text = self.render_full();
        // Return an empty string if start is out of range
        if start >= text.width() { return st!(""); }
        // Obtain the character indices
        let ind = raw_indices(&text, &self.indices);
        // Shift the cut point forward until on a character boundary
        let space = !ind.contains_key(&start);
        while !ind.contains_key(&start) { start += 1; }
        // Perform cut and format
        format!("{}{}", if space { " " } else { "" }, &text[ind[&start]..])
    }

    /// Render the entire row
    pub fn render_full(&self) -> String {
        // Retrieve tab width
        let tabs = self.get_tab_width();
        self.text.iter().fold(st!(""), |a, x| {
            format!("{}{}", a, if x == &'\t' { " ".repeat(tabs) } else { x.to_string() })
        })
    }

    /// Find the character length of this row
    pub fn len(&self) -> usize {
        self.text.len()
    }

    /// Find the display width of this row
    pub fn width(&self) -> usize {
        self.render_full().width()
    }

    /// Calculate the character pointer from a display index
    pub fn get_char_ptr(&self, x: usize) -> usize {
        // Handle large values of x
        if x >= self.width() { 
            return self.len() 
        }
        // Calculate the character width
        self.indices.iter().position(|i| &x == i).unwrap()
    }

    /// Find the widths of the characters in raw text
    fn raw_to_indices(text: &[char], tab_width: usize) -> Vec<usize> {
        let mut data = vec![&'\x00'];
        data.splice(1.., text);
        data.iter()
            .map(|c| if c == &&'\t' { tab_width } else { c.width().unwrap_or(0) })
            .scan(0, |a, x| { *a += x; Some(*a) })
            .collect()
    }

    /// Retrieve the tab width from the document info
    pub fn get_tab_width(&self) -> usize {
        if self.info.is_null() { 4 } else { unsafe { &*self.info }.tab_width }
    }
}
