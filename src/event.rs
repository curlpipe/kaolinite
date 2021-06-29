//! event: Enums that represent the status of the editor and events
//!
//! This contains the Error types, as well as the possible events you can use

use crate::utils::Loc;
use thiserror::Error;

/// Neater error type
pub type Result<T> = std::result::Result<T, Error>;

/// Event represents all the document events that could occur
#[derive(Debug)]
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
#[derive(Debug, Error)]
pub enum Error {
    /// Returned when you provide an index that is out of range
    #[error("Out of range")]
    OutOfRange,
    /// When the program is unable to open a file e.g. doesn't exist or file permissions
    #[error("File not found")]
    FileError(#[from] std::io::Error),
    /// Saving an unnamed file
    #[error("No file name for this document")]
    NoFileName,
}
