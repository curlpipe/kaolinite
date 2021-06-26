// event.rs - Contains enums that represent the status of the editor and events
use thiserror::Error;

/// Neater error type
pub type Result<T> = std::result::Result<T, Error>;

/// Event represents all the document events that could occur
#[derive(Debug)]
pub enum Event {
    /// Opens a file in the document
    OpenFile(String),
}

/// Status contains the states the document can be in after an event execution
#[derive(Debug, PartialEq)]
pub enum Status {
    /// Cursor reaches the end of a row
    EndOfRow,
    /// Cursor reaches the start of a row
    StartOfRow,
    /// Cursor reaches the start of the document
    EndOfDocument,
    /// Cursor reaches the start of the document
    StartOfDocument,
    /// Nothing of note
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
