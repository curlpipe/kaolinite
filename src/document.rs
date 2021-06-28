use crate::utils::{Loc, Size, LINE_ENDING_SPLITTER};
use crate::event::{Result, Error, Status};
use unicode_width::UnicodeWidthChar;
use crate::row::Row;
use std::fs;

/// A struct that stores information about a file
#[derive(Debug, PartialEq, Clone)]
pub struct FileInfo {
    /// The file name of the document
    pub file: Option<String>,
    /// True if \r\n is used, false if \n is used
    pub is_dos: bool,
    /// Tab width of the file in spaces (default is 4, you can overwrite if need be)
    /// There is a slight quirk, however. You must edit this field *directly after*
    /// defining a Document, otherwise, the configuration may not apply.
    pub tab_width: usize,
}

impl Default for FileInfo {
    fn default() -> Self {
        Self {
            file: None,
            is_dos: false,
            tab_width: 4,
        }
    }
}

/// A struct that contains all the basic tools necessary to manage documents
#[derive(Debug, Default)]
pub struct Document {
    /// The information for the current file
    pub info: FileInfo,
    /// All the rows within the document
    pub rows: Vec<Row>,
    /// The size holds how much space the document has to render
    pub size: Size,
    /// A pointer to the character at the current cursor position
    pub char_ptr: usize,
    /// The position within the terminal
    pub cursor: Loc,
    /// Stores information about scrolling
    pub offset: Loc,
}

impl Document {
    /// Create a new document
    #[cfg(not(tarpaulin_include))]
    pub fn new<S: Into<Size>>(size: S) -> Self {
        Self {
            info: FileInfo::default(),
            rows: vec![],
            cursor: Loc::default(),
            offset: Loc::default(),
            size: size.into(),
            char_ptr: 0,
        }
    }
    /// Open a file at a specified path into this document
    /// This will also reset the cursor position, offset position, 
    /// file name, contents and line ending information
    #[cfg(not(tarpaulin_include))]
    pub fn open<P: Into<String>>(&mut self, path: P) -> Result<()> {
        // Read in information
        let path = path.into();
        let raw = fs::read_to_string(&path)?;
        // Reset to default values
        self.info = FileInfo {
            file: Some(path),
            is_dos: raw.contains("\\r\\n"),
            tab_width: self.info.tab_width,
        };
        self.cursor = Loc::default();
        self.offset = Loc::default();
        self.char_ptr = 0;
        // Load in the rows
        self.rows = self.raw_to_rows(&raw);
        Ok(())
    }

    /// Save a file
    pub fn save(&self) -> Result<()> {
        let data = self.render();
        let file = self.info.file.as_ref().ok_or(Error::NoFileName)?;
        fs::write(file, data)?;
        Ok(())
    }

    /// Save a file to a specified path
    pub fn save_as(&self, file: &str) -> Result<()> {
        let data = self.render();
        fs::write(file, data)?;
        Ok(())
    }

    /// Return a reference to a row in the document
    pub fn row(&self, index: usize) -> Result<&Row> {
        Ok(self.rows.get(index).ok_or(Error::OutOfRange)?)
    }

    /// Return a mutable reference to a row in the document
    pub fn row_mut(&mut self, index: usize) -> Result<&mut Row> {
        Ok(self.rows.get_mut(index).ok_or(Error::OutOfRange)?)
    }

    /// Get the current row
    pub fn current_row(&self) -> Result<&Row> {
        Ok(self.row(self.loc().y)?)
    }

    /// Move the cursor to the left
    pub fn move_left(&mut self) -> Result<Status> {
        // Check to see if the cursor is already as far left as possible
        if self.char_ptr == 0 { return Ok(Status::StartOfRow) }
        // Traverse the grapheme
        for _ in 0..self.get_width(-1)? {
            // Determine whether to change offset or cursor
            if self.cursor.x == 0 { 
                self.offset.x -= 1 
            } else { 
                self.cursor.x -= 1 
            }
        }
        self.char_ptr -= 1; 
        Ok(Status::None)
    }

    /// Move the cursor to the right
    pub fn move_right(&mut self) -> Result<Status> {
        // Check to see if the cursor is already as far left as possible
        if self.char_ptr == self.current_row()?.len() { return Ok(Status::EndOfRow) }
        // Traverse the grapheme
        for _ in 0..self.get_width(0)? {
            // Determine whether to change offset or cursor
            if self.cursor.x == self.size.w - 1 {
                self.offset.x += 1;
            } else {
                self.cursor.x += 1;
            }
        }
        self.char_ptr += 1;
        Ok(Status::None)
    }

    /// Move the cursor upwards
    pub fn move_up(&mut self) -> Result<Status> {
        // Check to see if the cursor is already as far up as possible
        if self.loc().y == 0 {
            return Ok(Status::StartOfDocument)
        }
        // Determine whether to change offset or cursor
        if self.cursor.y == 0 {
            self.offset.y -= 1;
        } else {
            self.cursor.y -= 1;
        }
        // Snap to grapheme boundary
        self.snap_grapheme()?;
        // Correct char pointer
        self.char_ptr = self.current_row()?.get_char_ptr(self.loc().x);
        Ok(Status::None)
    }

    /// Move the cursor downwards
    pub fn move_down(&mut self) -> Result<Status> {
        // Check to see if the cursor is already as far up as possible
        if self.loc().y == self.rows.len() - 1 {
            return Ok(Status::EndOfDocument)
        } 
        // Determine whether to change offset or cursor
        if self.cursor.y == self.size.h - 1 {
            self.offset.y += 1;
        } else {
            self.cursor.y += 1;
        }
        // Snap to grapheme boundary
        self.snap_grapheme()?;
        // Correct char pointer
        self.char_ptr = self.current_row()?.get_char_ptr(self.loc().x);
        Ok(Status::None)
    }

    /// Render the document into the correct form
    pub fn render(&self) -> String {
        let line_ending = if self.info.is_dos { "\r\n" } else { "\n" };
        self.rows
            .iter()
            .map(|x| x.render_full())
            .collect::<Vec<_>>()
            .join(line_ending)
    }

    /// Get the width of a character
    fn get_width(&self, offset: isize) -> Result<u16> {
        // TODO: Optimise using arithmetic rather than width calculation
        let idx = (self.char_ptr as isize + offset) as usize;
        Ok(self.current_row()?.text[idx].width().unwrap_or(0) as u16)
    }

    /// Get the current position in the document
    const fn loc(&self) -> Loc {
        Loc {
            x: self.cursor.x + self.offset.x,
            y: self.cursor.y + self.offset.y,
        }
    }

    /// Shift the cursor back to the nearest grapheme boundary
    fn snap_grapheme(&mut self) -> Result<()> {
        // Collect information
        let row = self.current_row()?;
        let start = self.loc().x;
        let mut ptr = self.loc().x;
        // Shift back until on boundary
        while !row.indices.contains(&ptr) { 
            ptr -= 1;
        }
        // Work out required adjustment
        let adjustment = start - ptr;
        // Perform adjustment
        for _ in 0..adjustment {
            if self.cursor.x == 0 { 
                self.offset.x -= 1 
            } else { 
                self.cursor.x -= 1 
            }
        }
        Ok(())
    }

    /// Take raw text and convert it into Row structs
    fn raw_to_rows(&mut self, text: &str) -> Vec<Row> {
        let rows: Vec<&str> = LINE_ENDING_SPLITTER.split(text).collect();
        rows.iter().map(|s| Row::new(*s).link(&mut self.info)).collect()
    }
}
