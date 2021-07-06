//! document: Tools for opening and saving files
//!
//! Here is where you'll find the most important struct: [Document]
//! Please see the documentation over at [kaolinite](crate) for more information
//!
//! This module also contains the [`FileInfo`] struct, which contains information
//! about the opened file, which holds things like the file name, file ending and tab width.
//!
//! See the structs section below to find out more about each struct

use crate::event::{Error, Event, Result, Status};
use crate::row::Row;
use crate::utils::{filetype, width_char, Loc, Size};
use crate::{regex, st};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

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
    /// Create a `FileInfo` struct with default data
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
    /// Boolean that changes when the file is edited via the event executor
    pub modified: bool,
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
    ///
    /// The argument `size` takes in a [Size](crate::utils::Size) struct. This should
    /// store information about the terminal size.
    ///
    /// If you plan to implement things
    /// like status lines or tabs, you should subtract them from the size height, as this
    /// size is purely for the file viewport size.
    #[cfg(not(tarpaulin_include))]
    pub fn new<S: Into<Size>>(size: S) -> Self {
        Self {
            info: FileInfo::default(),
            rows: vec![],
            modified: false,
            cursor: Loc::default(),
            offset: Loc::default(),
            size: size.into(),
            char_ptr: 0,
        }
    }

    /// Open a file at a specified path into this document.
    ///
    /// This will also reset the cursor position, offset position,
    /// file name, contents and line ending information
    /// # Errors
    /// Will return `Err` if `path` does not exist or the user does not have
    /// permission to read from it.
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
        self.modified = false;
        // Load in the rows
        self.rows = self.raw_to_rows(&raw);
        Ok(())
    }

    /// Save a file
    ///
    /// This will reset `modified` to `false`, as it has been saved back to it's original file.
    /// # Errors
    /// Will return `Err` if the file path the document came from wasn't able to be written
    /// to, potentially because of file permission errors.
    pub fn save(&mut self) -> Result<()> {
        let data = self.render();
        let file = self.info.file.as_ref().ok_or(Error::NoFileName)?;
        fs::write(file, data)?;
        self.modified = false;
        Ok(())
    }

    /// Save a file to a specified path
    ///
    /// Similar to [save](Document::save) but takes a file argument, and saves it there.
    /// This method also doesn't change `modified`.
    /// # Errors
    /// Will return `Err` if the provided file path wasn't able to be written to,
    /// potentially because fo file permission errors.
    pub fn save_as(&self, file: &str) -> Result<()> {
        let data = self.render();
        fs::write(file, data)?;
        Ok(())
    }

    /// Execute an event in this document
    ///
    /// This method is the main method that should be used to modify the document.
    /// It takes in an [Event](crate::event::Event) enum.
    ///
    /// This method also takes advantage of undo & redo functionality and
    /// the document modificatior indicator and moves your cursor automatically.
    /// If you change the rows in the document directly, you will not gain access
    /// to these benefits, but you can always manually handle these features if need be.
    /// # Errors
    /// Will return `Err` if the event tried to modifiy data outside the scope of the
    /// document.
    pub fn execute(&mut self, event: Event) -> Result<Status> {
        match event {
            Event::Insert(loc, ch) => {
                self.goto(loc)?;
                self.row_mut(loc.y)?.insert(loc.x, ch)?;
                self.modified = true;
                self.move_right()
            }
            Event::Remove(mut loc, _) => {
                if loc.x == 0 {
                    return Ok(Status::StartOfRow);
                }
                loc.x -= 1;
                self.goto(loc)?;
                self.row_mut(loc.y)?.remove(loc.x..=loc.x)?;
                self.modified = true;
                Ok(Status::None)
            }
            Event::InsertRow(loc, st) => {
                self.rows.insert(loc, Row::new(st).link(&mut self.info));
                self.modified = true;
                self.goto_y(loc)?;
                Ok(Status::None)
            }
            Event::RemoveRow(loc, _) => {
                self.goto_y(loc - if loc == 0 { 0 } else { 1 })?;
                self.rows.remove(loc);
                self.modified = true;
                Ok(Status::None)
            }
            Event::SpliceUp(loc) => {
                if loc.y == 0 {
                    return Ok(Status::StartOfDocument);
                }
                let mut upper = self.row(loc.y - 1)?.clone();
                let x = upper.len();
                let lower = self.row(loc.y)?.clone();
                self.rows[loc.y - 1] = upper.splice(lower);
                self.modified = true;
                self.rows.remove(loc.y);
                self.goto((x, loc.y - 1))?;
                Ok(Status::None)
            }
            Event::SplitDown(loc) => {
                let (left, right) = self.row(loc.y)?.split(loc.x)?;
                self.rows[loc.y] = left;
                self.modified = true;
                self.rows.insert(loc.y + 1, right);
                self.goto((0, loc.y + 1))?;
                Ok(Status::None)
            }
        }
    }

    /// Move the cursor to a specific x and y coordinate
    /// # Errors
    /// Will return `Err` if the location provided is out of scope of the document.
    pub fn goto<L: Into<Loc>>(&mut self, loc: L) -> Result<()> {
        let loc = loc.into();
        self.goto_y(loc.y)?;
        self.goto_x(loc.x)?;
        Ok(())
    }

    /// Move the cursor to a specific x coordinate
    /// # Errors
    /// Will return `Err` if the location provided is out of scope of the document.
    pub fn goto_x(&mut self, x: usize) -> Result<()> {
        // Bounds checking
        if self.char_ptr == x {
            return Ok(());
        } else if x > self.current_row()?.len() {
            return Err(Error::OutOfRange);
        }
        // Gather and update information
        let viewport = self.offset.x..self.offset.x + self.size.w;
        self.char_ptr = x;
        let x = *self
            .current_row()?
            .indices
            .get(x)
            .ok_or(Error::OutOfRange)?;
        // Start movement
        if x < self.size.w {
            // Cursor is in view when offset is 0
            self.offset.x = 0;
            self.cursor.x = x;
        } else if viewport.contains(&x) {
            // If the point is in viewport already, only move cursor
            self.cursor.x = x - self.offset.x;
        } else {
            // If the point is out of viewport, set cursor to 0, and adjust offset
            self.cursor.x = 0;
            self.offset.x = x;
        }
        Ok(())
    }

    /// Move the cursor to a specific y coordinate
    /// # Errors
    /// Will return `Err` if the location provided is out of scope of the document.
    pub fn goto_y(&mut self, y: usize) -> Result<()> {
        // Bounds checking
        if self.raw_loc().y == y {
            return Ok(());
        } else if y > self.rows.len() {
            return Err(Error::OutOfRange);
        }
        let viewport = self.offset.y..self.offset.y + self.size.h;
        if y < self.size.h {
            // Cursor is in view when offset is 0
            self.offset.y = 0;
            self.cursor.y = y;
        } else if viewport.contains(&y) {
            // If the point is in viewport already, only move cursor
            self.cursor.y = y - self.offset.y;
        } else {
            // If the point is out of viewport, move cursor to bottom, and adjust offset
            self.cursor.y = self.size.h - 1;
            self.offset.y = y - (self.size.h - 1);
        }
        // Snap to grapheme boundary
        self.snap_grapheme()?;
        // Correct char pointer
        self.char_ptr = self.current_row()?.get_char_ptr(self.raw_loc().x);
        Ok(())
    }

    /// Move the cursor to the left
    /// # Errors
    /// Will return `Err` if the cursor is out of scope of the document
    pub fn move_left(&mut self) -> Result<Status> {
        // Check to see if the cursor is already as far left as possible
        if self.char_ptr == 0 {
            return Ok(Status::StartOfRow);
        }
        // Traverse the grapheme
        for _ in 0..self.get_width(-1)? {
            // Determine whether to change offset or cursor
            if self.cursor.x == 0 {
                self.offset.x -= 1;
            } else {
                self.cursor.x -= 1;
            }
        }
        self.char_ptr -= 1;
        Ok(Status::None)
    }

    /// Move the cursor to the right
    /// # Errors
    /// Will return `Err` if the cursor is out of scope of the document
    pub fn move_right(&mut self) -> Result<Status> {
        // Check to see if the cursor is already as far right as possible
        if self.char_ptr == self.current_row()?.len() {
            return Ok(Status::EndOfRow);
        }
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
    /// # Errors
    /// Will return `Err` if the cursor is out of scope of the document
    pub fn move_up(&mut self) -> Result<Status> {
        // Check to see if the cursor is already as far up as possible
        if self.raw_loc().y == 0 {
            return Ok(Status::StartOfDocument);
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
        self.char_ptr = self.current_row()?.get_char_ptr(self.raw_loc().x);
        Ok(Status::None)
    }

    /// Move the cursor downwards
    /// # Errors
    /// Will return `Err` if the cursor is out of scope of the document
    pub fn move_down(&mut self) -> Result<Status> {
        // Check to see if the cursor is already as far up as possible
        if self.raw_loc().y == self.rows.len() {
            return Ok(Status::EndOfDocument);
        }
        // Determine whether to change offset or cursor
        if self.cursor.y == self.size.h - 1 {
            self.offset.y += 1;
        } else {
            self.cursor.y += 1;
        }
        // Snap to grapheme boundary
        std::mem::drop(self.snap_grapheme());
        // Correct char pointer
        self.char_ptr = if let Ok(row) = self.current_row() {
            row.get_char_ptr(self.raw_loc().x)
        } else {
            // Move to 0 when entering row below document
            self.cursor.x = 0;
            self.offset.x = 0;
            0
        };
        Ok(Status::None)
    }

    /// Work out the line number text to use
    #[must_use]
    pub fn line_number(&self, row: usize) -> String {
        let total = self.rows.len().to_string().len();
        let num = (row + 1).to_string();
        format!("{}{}", " ".repeat(total - num.len()), num)
    }

    /// A helper function that returns info about the document
    /// in a [`HashMap`] type.
    ///
    /// It will return (with keys for the hashmap):
    /// - Row number: `row`
    /// - Column number: `column`
    /// - Total rows: `total`
    /// - File name (no path): `file`
    /// - File name (full path): `full_file`
    /// - File type: `type`
    /// - Modifed indicator: `modified`
    /// - File extension: `extension`
    pub fn status_line_info(&self) -> HashMap<&str, String> {
        let row = self.loc().y + 1;
        let total = self.rows.len();
        let column = self.loc().x;
        let modified = if self.modified { "[+]" } else { "" };
        let (full_file, file, ext) = if let Some(name) = &self.info.file {
            let f = Path::new(&name)
                .file_name()
                .unwrap()
                .to_str()
                .unwrap_or(name)
                .to_string();
            let e = Path::new(&name)
                .extension()
                .unwrap()
                .to_str()
                .unwrap_or("")
                .to_string();
            (name.clone(), f, e)
        } else {
            (st!("[No Name]"), st!("[No Name]"), st!(""))
        };
        let mut info = HashMap::new();
        info.insert("row", st!(row));
        info.insert("column", st!(column));
        info.insert("total", st!(total));
        info.insert("file", st!(file));
        info.insert("full_file", st!(full_file));
        info.insert("type", filetype(&ext).unwrap_or_else(|| st!("Unknown")));
        info.insert("modified", st!(modified));
        info.insert("extension", ext);
        info
    }

    /// Render the document into the correct form
    #[must_use]
    pub fn render(&self) -> String {
        let line_ending = if self.info.is_dos { "\r\n" } else { "\n" };
        self.rows
            .iter()
            .map(Row::render_raw)
            .collect::<Vec<_>>()
            .join(line_ending)
            + line_ending
    }

    /// Shift the cursor back to the nearest grapheme boundary
    fn snap_grapheme(&mut self) -> Result<()> {
        // Collect information
        let row = self.current_row()?;
        let start = self.raw_loc().x;
        let mut ptr = self.raw_loc().x;
        // Shift back until on boundary
        while !row.indices.contains(&ptr) {
            ptr -= 1;
        }
        // Work out required adjustment
        let adjustment = start - ptr;
        // Perform adjustment
        for _ in 0..adjustment {
            if self.cursor.x == 0 {
                self.offset.x -= 1;
            } else {
                self.cursor.x -= 1;
            }
        }
        Ok(())
    }

    /// Take raw text and convert it into Row structs
    fn raw_to_rows(&mut self, text: &str) -> Vec<Row> {
        let text = regex!("(\\r\\n|\\n)$").replace(text, "").to_string();
        let rows: Vec<&str> = regex!("(\\r\\n|\\n)").split(&text).collect();
        rows.iter()
            .map(|s| Row::new(*s).link(&mut self.info))
            .collect()
    }

    /// Return a reference to a row in the document
    /// # Errors
    /// This will error if the index is out of range
    pub fn row(&self, index: usize) -> Result<&Row> {
        self.rows.get(index).ok_or(Error::OutOfRange)
    }

    /// Return a mutable reference to a row in the document
    /// # Errors
    /// This will error if the index is out of range
    pub fn row_mut(&mut self, index: usize) -> Result<&mut Row> {
        self.rows.get_mut(index).ok_or(Error::OutOfRange)
    }

    /// Get the current row
    /// # Errors
    /// This will error if the cursor position isn't on a existing row
    pub fn current_row(&self) -> Result<&Row> {
        self.row(self.raw_loc().y)
    }

    /// Get the width of a character
    fn get_width(&self, offset: i128) -> Result<usize> {
        // TODO: Optimise using arithmetic rather than width calculation
        let idx = (self.char_ptr as i128 + offset) as usize;
        let ch = self.current_row()?.text[idx];
        Ok(width_char(ch, self.info.tab_width))
    }

    /// Get the current position in the document
    ///
    /// This ought to be used by the document only as it returns the display indices
    /// Use the `Document::loc` function instead.
    #[must_use]
    pub const fn raw_loc(&self) -> Loc {
        Loc {
            x: self.cursor.x + self.offset.x,
            y: self.cursor.y + self.offset.y,
        }
    }

    /// Get the current position in the document
    ///
    /// This will return the character and row indices
    #[must_use]
    pub const fn loc(&self) -> Loc {
        Loc {
            x: self.char_ptr,
            y: self.cursor.y + self.offset.y,
        }
    }
}
