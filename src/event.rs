//! event: Enums that represent the status of the editor and events
//!
//! This contains the Error types, as well as the possible events you can use

use crate::utils::Loc;

/// Neater error type
pub type Result<T> = std::result::Result<T, Error>;

/// Event represents all the document events that could occur
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// Insert a character at a position.
    /// Takes a location and a character to insert
    Insert(Loc, char),
    /// Remove a character at a position.
    /// Takes a location and the character that has been removed.
    Remove(Loc, char),
    /// Insert a row.
    /// Takes a row index and a string for the row.
    InsertRow(usize, String),
    /// Remove a row.
    /// Takes a row index and a string for the row.
    RemoveRow(usize, String),
    /// Cut a line in half and drop the last half down a line.
    /// This is for times when the enter key is pressed in the middle of a line.
    SplitDown(Loc),
    /// Splice a line with the line above.
    /// This is for times when the backspace key is pressed at the start of a line.
    SpliceUp(Loc),
}

/// Status contains the states the document can be in after an event execution
#[derive(Debug, PartialEq)]
pub enum Status {
    /// Cursor reaches the end of a row.
    /// Useful for if you want to wrap the cursor around when it hits the end of the row.
    EndOfRow,
    /// Cursor reaches the start of a row.
    /// Useful for if you want to wrap the cursor around when it hits the start of the row.
    StartOfRow,
    /// Cursor reaches the start of the document.
    EndOfDocument,
    /// Cursor reaches the start of the document.
    StartOfDocument,
    /// Nothing of note.
    None,
}

/// Error represents the potential failures in function calls when using this API
#[derive(Debug)]
pub enum Error {
    /// Returned when you provide an index that is out of range
    OutOfRange,
    /// When the program is unable to open a file e.g. doesn't exist or file permissions
    FileError(std::io::Error),
    /// Saving an unnamed file
    NoFileName,
}

impl std::fmt::Display for Error {
    #[cfg(not(tarpaulin_include))]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    #[cfg(not(tarpaulin_include))]
    fn from(e: std::io::Error) -> Error {
        Error::FileError(e)
    }
}

/// Event stack is a struct that handles events
#[derive(Debug, Default)]
pub struct EditStack {
    /// Where the current smaller editing events are stored
    pub patch: Vec<Event>,
    /// This is where events that have been done are
    pub done: Vec<Vec<Event>>,
    /// This is where events that have been undone are
    pub undone: Vec<Vec<Event>>,
}

impl EditStack {
    /// Adds an event to the current patch
    pub fn exe(&mut self, event: Event) {
        self.undone.clear();
        self.patch.push(event);
    }

    /// Commit the patch to the done stack
    pub fn commit(&mut self) {
        if !self.patch.is_empty() {
            let patch = std::mem::take(&mut self.patch);
            self.done.push(patch);
        }
    }

    /// Returns the last performed event and moves it around
    pub fn undo(&mut self) -> Option<&Vec<Event>> {
        self.commit();
        let mut done = self.done.pop()?;
        done.reverse();
        self.undone.push(done);
        self.undone.last()
    }

    /// Returns the last undone event and moves it around
    pub fn redo(&mut self) -> Option<&Vec<Event>> {
        let mut undone = self.undone.pop()?;
        undone.reverse();
        self.done.push(undone);
        self.done.last()
    }
}
