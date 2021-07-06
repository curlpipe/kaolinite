/*
    Editor - A demonstration of the Kaolinite
    in 258 SLOC (without comments and blanks)
    using the Crossterm crate

    This editor has unicode and scrolling support, a basic status line, a command line interface
    handles editing multiple files, saving files, filetype detection, line numbers and word jumping,
    line wrapping
*/

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{read, Event as CEvent, KeyCode as KCode, KeyEvent as KEvent, KeyModifiers as KMod},
    execute,
    style::{Color, SetBackgroundColor as Bg},
    terminal::{self, Clear, ClearType as ClType, EnterAlternateScreen, LeaveAlternateScreen},
    Result,
};
use kaolinite::document::Document;
use kaolinite::event::{Event, Status};
use kaolinite::st;
use kaolinite::utils::{align_sides, Loc, Size};
use pico_args::Arguments;
use std::io::{stdout, Error, ErrorKind, Write};

const STATUS_BG: Bg = Bg(Color::Rgb {
    r: 31,
    g: 92,
    b: 62,
});
const RESET_BG: Bg = Bg(Color::Reset);
const USAGE: &str = "\
Editor: A basic command line text editor that demonstrates the kaolinite crate.

USAGE: editor [files] [options]

OPTIONS:
    --help, -h :  Show this help message

EXAMPLES:
    editor test.txt
    editor test.txt test2.txt
    editor /home/user/dev/editor/main.rs";

fn size() -> Result<Size> {
    // This function gets the size from crossterm and converts it to a Loc
    let (w, h) = terminal::size()?;
    Ok(Size {
        w: w as usize,
        h: h as usize - 1,
    })
}

struct Editor {
    // The stdout to write to
    stdout: std::io::Stdout,
    // Holds the documents that are open
    documents: Vec<Document>,
    // Points to the document that we are currently editing
    doc_ptr: usize,
    // Determines if the editor is active
    active: bool,
}

impl Editor {
    pub fn new() -> Result<Self> {
        // Reads arguments, creates document and returns editor ready to go
        let mut args = Arguments::from_env();
        // Check for help
        if args.contains(["-h", "--help"]) {
            println!("{}", USAGE);
            std::process::exit(0);
        }
        // Get the terminal size
        let size = size()?;
        // Set up the documents vector
        let mut documents = vec![];
        // Read all the file arguments
        while let Some(doc) = args.subcommand().unwrap_or_default() {
            // Create a document
            let mut document = Document::new(size);
            // Open the file, returning an error if one of them isn't found
            document.open(&doc).ok().ok_or_else(|| {
                Error::new(ErrorKind::NotFound, format!("Failed to open file {}", doc))
            })?;
            // Push the document to the documents list
            documents.push(document);
        }
        // Handle the event that no files are open
        if documents.is_empty() {
            return Err(Error::new(
                ErrorKind::NotFound,
                "No file arguments were provided",
            ));
        }
        // Return a new editor struct with a stdout ready to go
        Ok(Self {
            stdout: stdout(),
            documents,
            doc_ptr: 0,
            active: true,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        // This will start the editor and run the event loop
        self.start()?;
        // Render initial frame
        self.render()?;
        // Start event loop
        while self.active {
            // Handle key event (ignore other events)
            if let CEvent::Key(key) = read()? {
                self.action(key)?
            }
            // Render frame after event
            self.render()?;
        }
        self.end()?;
        Ok(())
    }

    fn render(&mut self) -> Result<()> {
        // Render a frame
        let size = size()?;
        // Update document width (and offset line numbers)
        let max = self.doc().rows.len().to_string().len() + 2;
        self.doc_mut().size.w = size.w.saturating_sub(max);
        self.doc_mut().size.h = size.h;
        // Hide the cursor
        write!(self.stdout, "{}", Hide)?;
        // Render each row in the document
        for y in 0..=size.h {
            // Move to the correct line and clear it
            execute!(self.stdout, MoveTo(0, y as u16), Clear(ClType::CurrentLine))?;
            if y == size.h {
                // Render status line
                let info = self.doc().status_line_info();
                // Left and right hand sides of status line
                let lhs = format!(
                    " {}{} | {} | {}/{} |",
                    info["file"],
                    info["modified"],
                    info["type"],
                    self.doc_ptr + 1,
                    self.documents.len()
                );
                let rhs = format!("| {}/{} | {} ", info["row"], info["total"], info["column"]);
                // Calculate padding and render the status line with a coloured background
                let tab_width = self.doc().info.tab_width;
                let status_line = align_sides(&lhs, &rhs, size.w, tab_width)
                    .unwrap_or_else(|| " ".repeat(size.w));
                write!(self.stdout, "{}{}{}", STATUS_BG, status_line, RESET_BG)?;
            } else {
                // Render document rows
                // Determine if the line exists, if not, draw a ~
                let idx = y + self.doc().offset.y;
                if let Ok(row) = self.doc().row(idx) {
                    let text = row.render(self.doc().offset.x..);
                    write!(self.stdout, "{} │{}", self.doc().line_number(idx), text)?;
                } else {
                    write!(self.stdout, "{}~ │", " ".repeat(max.saturating_sub(3)))?;
                }
            }
        }
        // Move to that cursor position and show the cursor again
        let Loc { x, y } = self.doc().cursor;
        execute!(self.stdout, MoveTo((x + max) as u16, y as u16), Show)?;
        // Actually render the frame
        self.stdout.flush()?;
        Ok(())
    }

    fn action(&mut self, key: KEvent) -> Result<()> {
        // Function that handles keypress events
        let loc = self.doc().loc();
        match (key.code, key.modifiers) {
            // Quit the editor
            (KCode::Char('q'), KMod::CONTROL) => {
                self.active = false;
            }
            // Cursor movement
            (KCode::Up, KMod::NONE) => {
                self.doc_mut().move_up().unwrap();
            }
            (KCode::Down, KMod::NONE) => {
                self.doc_mut().move_down().unwrap();
            }
            (KCode::Left, KMod::NONE) => {
                let status = self.doc_mut().move_left();
                // Wrap cursor if at the start of the row
                if let Ok(Status::StartOfRow) = status {
                    self.wrap_left()?;
                }
            }
            (KCode::Right, KMod::NONE) => {
                let status = self.doc_mut().move_right();
                // Wrap cursor if at the end of the row
                if let Ok(Status::EndOfRow) = status {
                    self.wrap_right()?;
                }
            }
            (KCode::Home, KMod::NONE) => {
                self.doc_mut().goto_x(0).unwrap();
            }
            (KCode::End, KMod::NONE) => {
                if let Ok(row) = self.doc().current_row() {
                    let len = row.len();
                    self.doc_mut().goto_x(len).unwrap();
                }
            }
            // Character insertion
            (KCode::Char(c), KMod::NONE | KMod::SHIFT) => {
                self.new_row();
                self.doc_mut().execute(Event::Insert(loc, c)).unwrap();
            }
            (KCode::Tab, KMod::NONE | KMod::SHIFT) => {
                self.new_row();
                self.doc_mut().execute(Event::Insert(loc, '\t')).unwrap();
            }
            // Character deletion & row splicing
            (KCode::Backspace, KMod::NONE) => {
                self.new_row();
                // Determine if delete or splice required
                if loc.x == 0 {
                    self.doc_mut().execute(Event::SpliceUp(loc)).unwrap();
                } else {
                    let c = self.doc().row(loc.y).unwrap().text[loc.x - 1];
                    self.doc_mut().execute(Event::Remove(loc, c)).unwrap();
                }
            }
            // Return key
            (KCode::Enter, KMod::NONE) => {
                self.new_row();
                self.doc_mut().execute(Event::SplitDown(loc)).unwrap();
            }
            // Switching tabs
            (KCode::Left, KMod::SHIFT) => {
                if self.doc_ptr != 0 {
                    self.doc_ptr -= 1;
                }
            }
            (KCode::Right, KMod::SHIFT) => {
                if self.doc_ptr + 1 < self.documents.len() {
                    self.doc_ptr += 1;
                }
            }
            // Saving
            (KCode::Char('s'), KMod::CONTROL) => {
                self.doc_mut().save().unwrap();
            }
            // Word jumping
            (KCode::Left, KMod::CONTROL) => {
                if let Ok(row) = self.doc().current_row() {
                    let to = row.next_word_back(loc.x);
                    self.doc_mut().goto_x(to).unwrap();
                }
            }
            (KCode::Right, KMod::CONTROL) => {
                if let Ok(row) = self.doc().current_row() {
                    let to = row.next_word_forth(loc.x);
                    self.doc_mut().goto_x(to).unwrap();
                }
            }
            _ => (),
        }
        Ok(())
    }

    fn wrap_left(&mut self) -> Result<()> {
        // Wrap the cursor around, when moving left
        if let Ok(Status::None) = self.doc_mut().move_up() {
            // Cursor was able to be moved up, move to end of row
            if let Ok(row) = self.doc().current_row() {
                let len = row.len();
                self.doc_mut().goto_x(len).unwrap();
            }
        }
        Ok(())
    }

    fn wrap_right(&mut self) -> Result<()> {
        // Wrap the cursor around, when moving right
        if let Ok(Status::None) = self.doc_mut().move_down() {
            // Cursor was able to be moved down, move to start of row
            self.doc_mut().goto_x(0).unwrap();
        }
        Ok(())
    }

    pub fn start(&mut self) -> Result<()> {
        // This will set the terminal up for rendering
        // Enter alternate screen to avoid terminal scrolling and interference
        execute!(self.stdout, EnterAlternateScreen, Clear(ClType::All))?;
        // Enable raw mode for full control of the terminal
        terminal::enable_raw_mode()?;
        Ok(())
    }

    pub fn end(&mut self) -> Result<()> {
        // This will revert the terminal to the previous settings
        terminal::disable_raw_mode()?;
        // Leave the alternate screen back to the terminal
        execute!(self.stdout, LeaveAlternateScreen)?;
        Ok(())
    }

    fn new_row(&mut self) {
        // Insert a new row if an edit is happening in the row below the end of the document
        let doc = self.doc();
        let y = doc.loc().y;
        if y == doc.rows.len() {
            self.doc_mut()
                .execute(Event::InsertRow(y, st!("")))
                .unwrap();
        }
    }

    fn doc(&self) -> &Document {
        // This will provide a reference to the current document
        &self.documents[self.doc_ptr]
    }

    fn doc_mut(&mut self) -> &mut Document {
        // This will return a mutable reference to the current document
        &mut self.documents[self.doc_ptr]
    }
}

fn main() -> Result<()> {
    // Handle panics
    std::panic::set_hook(Box::new(|e| {
        terminal::disable_raw_mode().unwrap();
        execute!(stdout(), LeaveAlternateScreen, Show).unwrap();
        eprintln!("{}", e);
    }));
    // Set up the editor and run it
    let mut editor = Editor::new()?;
    editor.run()?;
    Ok(())
}
