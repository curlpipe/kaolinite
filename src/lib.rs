//! Welcome to the documentatoin for Kaolinite
//! ## What is Kaolinite?
//!
//! At first, it seems like buliding a text editor is easy,
//! some pepole have made ones in fewer than 1000 lines!
//! But when you try opening files with unicode or large files
//! or implement your own configuration system that allows
//! the user to create custom themes and add their own syntax highlighting
//! it becomes very disorientating very quickly, and when using crates
//! like `syntect` you start seeing the crates your editor depends on
//! stack up and it compiles slower and slower.
//!
//! Kaolinite is a library that has most of the features you'll need
//! in order to create a TUI text editor, like vim or nano. It's lightweight
//! and tries to implement text editing in the most efficient way possible.
//!
//! It doesn't force you to use any TUI library in the Rust ecosystem,
//! so you can choose how to implement your UI. Nor does it force you to use
//! any style of editor, your editor could be modal if you wanted it to be.
//!
//! ## Usage
//!
//! Add this to your `Cargo.toml` file:
//! ```toml
//! [dependencies]
//! kaolinite = "0"
//! ```
//!
//! Or you can use `cargo-edit`:
//!
//! ```sh
//! $ cargo add kaolinite
//! ```
//!
//! The main struct that you'll want to use is [Document](document::Document).
//! This struct handles the insertion and deletion of characters, splitting rows,
//! splicing rows, reading files, saving files, cursor position and scrolling,
//! searcing, syntax highlighting, undo and redo, and unicode grapheme handling.
//!
//! There is also a [Row](row::Row) struct that provides more row-specific
//! operations and information such as finding word boundaries, rendering
//! themselves in certain ways, and determining if the row has been modified.
//! You won't really need to use many of the methods here, as
//! [Document](document::Document) handles most of the row operations you'd need.
//!
//! Here are a few examples of how it would look:
//!
//! ```
//! // Opening a file,
//! use kaolinite::document::Document;
//! let mut doc = Document::new();
//! // Imagine if test.txt were `The quick brown fox`
//! doc.open("test.txt").expect("Failed to open file");
//! // This would get the word boundaries of the first row: [0, 4, 10, 16, 19]
//! println!("{:?}", doc.row(0).words());
//! ```
//!
//! Because this library is quite a large collection of tools, it's hard to demonstrate it
//! here. You can find a examples directory on github
//! with many different examples, including a barebones text editor using the
//! [crossterm](https://docs.rs/crossterm) library, in under 300LOC, with full support for unicode.
//! You can use that as a starting point, if you wish.

#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::cast_sign_loss)]

pub mod document;
pub mod event;
pub mod row;
pub mod utils;
