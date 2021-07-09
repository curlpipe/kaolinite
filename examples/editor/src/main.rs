/*
    Editor - A demonstration of the Kaolinite
    in 364 SLOC (without comments and blanks)
    Uses the Crossterm crate for rendering
    and the Synpotic crate for efficeint syntax highlighting

    This editor has unicode and scrolling support, a basic status line, a command line interface
    handles editing multiple files, saving files, filetype detection, line numbers and word jumping,
    cursor wrapping, syntax highlighting, and undo & redo.
*/

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{read, Event as CEvent, KeyCode as KCode, KeyEvent as KEvent, KeyModifiers as KMod},
    execute,
    style::{Color, SetBackgroundColor as Bg, SetForegroundColor as Fg},
    terminal::{self, Clear, ClearType as ClType, EnterAlternateScreen, LeaveAlternateScreen},
    Result,
};
use kaolinite::document::Document;
use kaolinite::event::{Event, Status};
use kaolinite::st;
use kaolinite::utils::{align_sides, Loc, Size};
use pico_args::Arguments;
use std::io::{stdout, Error, ErrorKind, Write};
use synoptic::{trim, Highlighter, Token};

// Syntax highlighting regular expressions & keywords
// This section looks a bit complicated, but they're just regular expressions
const KEYWORDS: [&str; 63] = [
    "as", "break", "const", "continue", "crate", "else", "enum", "extern", "fn", "for", "if",
    "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return", "self",
    "static", "struct", "trait", "type", "unsafe", "use", "where", "while", "async", "await",
    "dyn", "box", "typeof", "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64",
    "i128", "isize", "f32", "f64", "String", "Vec", "str", "Some", "bool", "None", "Box", "Result",
    "Option", "Ok", "Err", "Self", "std",
];
const STRING: &str = "(?ms)\"(?:[^\"\\\\]*(?:\\\\.[^\"\\\\]*)*)\"";
const COMMENT: [&str; 2] = ["(?m)(//.*)$", r"(?ms)(/\*.*?\*/)"];
const MACRO: &str = r"\b([a-z_][a-zA-Z0-9_]*!)";
const REFERENCE: [&str; 4] = ["(&)", "&mut", "&self", "&str"];
const FUNCTION: [&str; 3] = [
    r"([a-z_][A-Za-z0-9_]*)\s*\(",
    r"\.([a-z_][A-Za-z0-9_]*)\s*\(",
    r"fn\s+([a-z_][A-Za-z0-9_]*)\s*\(",
];
const DIGIT: &str = r"\b(\d+.\d+|\d+)";
const NAMESPACE: &str = r"([A-Za-z0-9_]*)::";
const STRUCT: [&str; 3] = [
    r"(?:trait|enum|struct|impl)\s+([A-Z][A-Za-z0-9_]*)\s*",
    r"([A-Z][A-Za-z0-9_]*)\s*\(",
    r"impl(?:<.*?>|)\\s+([A-Z][A-Za-z0-9_]*)",
];

// Status line background colours
const STATUS_BG: Bg = Bg(Color::Rgb {
    r: 31,
    g: 92,
    b: 62,
});
const RESET_BG: Bg = Bg(Color::Reset);

// Command line interface usage
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

fn syntax_lookup(token: &str) -> Fg {
    // Look up the syntax highlighting colour to use
    match token {
        "keyword" => Fg(Color::DarkBlue),
        "string" => Fg(Color::Green),
        "macro" => Fg(Color::Magenta),
        "reference" => Fg(Color::Red),
        "struct" => Fg(Color::Cyan),
        "function" => Fg(Color::Cyan),
        "digit" => Fg(Color::DarkMagenta),
        "namespace" => Fg(Color::Cyan),
        "comment" => Fg(Color::Rgb {
            r: 80,
            g: 80,
            b: 80,
        }),
        _ => Fg(Color::White),
    }
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
    // Rust highlighter
    highlighter: Highlighter,
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
        // Implement rust syntax highlighting
        let mut highlighter = Highlighter::new();
        // Link up all the regular expressions and keywords into the highlighter
        KEYWORDS
            .iter()
            .for_each(|kw| highlighter.add(&format!(r"\b{}\b", kw), "keyword").unwrap());
        highlighter.add(STRING, "string").unwrap();
        highlighter.add(MACRO, "macro").unwrap();
        highlighter.add(DIGIT, "digit").unwrap();
        highlighter.add(NAMESPACE, "namespace").unwrap();
        highlighter.join(&COMMENT, "comment").unwrap();
        highlighter.join(&REFERENCE, "reference").unwrap();
        highlighter.join(&FUNCTION, "function").unwrap();
        highlighter.join(&STRUCT, "struct").unwrap();
        // Return a new editor struct with a stdout ready to go
        Ok(Self {
            stdout: stdout(),
            documents,
            doc_ptr: 0,
            active: true,
            highlighter,
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
                self.action(key)?;
            }
            self.render()?;
        }
        self.finish()?;
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
        // Render the entire document (if needed)
        // This is later used in the `synoptic` for syntax highlighting
        if self.doc().needs_rerender {
            self.doc_mut().render = self.doc().render_full();
            self.doc_mut().needs_rerender = false;
        }
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
                if idx < self.doc().rows.len() {
                    // Run the syntax highlighter
                    if self.doc().row(idx).unwrap().needs_rerender {
                        // Rerender if row has been modified
                        let tokens = self.highlighter.run_line(&self.doc().render, idx).unwrap();
                        self.doc_mut().row_mut(idx).unwrap().tokens = tokens;
                        self.doc_mut().row_mut(idx).unwrap().needs_rerender = false;
                    }
                    // Collect tokens and trim them to fit
                    let tokens = &self.doc().row(idx).unwrap().tokens;
                    let tokens = trim(tokens, self.doc().offset.x);
                    // Insert ANSI codes
                    let mut text = st!("");
                    for tok in tokens {
                        match tok {
                            // Start of a token
                            Token::Start(t) => text.push_str(&syntax_lookup(&t).to_string()),
                            // Text tokens
                            Token::Text(s) => text.push_str(&s),
                            // End of text tokens
                            Token::End(_) => text.push_str(&Fg(Color::Reset).to_string()),
                        }
                    }
                    // Render the newly created row
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
            (KCode::Char('q'), KMod::CONTROL) => self.quit(),
            // Saving
            (KCode::Char('s'), KMod::CONTROL) => self.doc_mut().save().unwrap(),
            // Undo & Redo
            (KCode::Char('z'), KMod::CONTROL) => self.undo(),
            (KCode::Char('y'), KMod::CONTROL) => self.redo(),
            // Quick line insertion
            (KCode::Char('f'), KMod::CONTROL) => self.exe(Event::InsertRow(loc.y + 1, st!(""))),
            (KCode::Char('h'), KMod::CONTROL) => self.delete_row(loc),
            // Switching tabs
            (KCode::Left, KMod::SHIFT) => self.previous_tab(),
            (KCode::Right, KMod::SHIFT) => self.next_tab(),
            // Word jumping
            (KCode::Left, KMod::CONTROL) => self.word_jump_left(loc),
            (KCode::Right, KMod::CONTROL) => self.word_jump_right(loc),
            // Cursor movement
            (KCode::Up, KMod::NONE) => self.up(),
            (KCode::Down, KMod::NONE) => self.down(),
            (KCode::Left, KMod::NONE) => self.left(),
            (KCode::Right, KMod::NONE) => self.right(),
            (KCode::Home, KMod::NONE) => self.doc_mut().goto_x(0).unwrap(),
            (KCode::End, KMod::NONE) => self.end(),
            // Character insertion
            (KCode::Char(c), KMod::NONE | KMod::SHIFT) => self.exe(Event::Insert(loc, c)),
            (KCode::Tab, KMod::NONE | KMod::SHIFT) => self.exe(Event::Insert(loc, '\t')),
            // Character deletion & row splicing
            (KCode::Backspace, KMod::NONE) => self.backspace(loc),
            // Return key
            (KCode::Enter, KMod::NONE) => self.exe(Event::SplitDown(loc)),
            _ => (),
        }
        Ok(())
    }

    fn exe(&mut self, event: Event) {
        // Execute an event
        self.new_row();
        // Depending on certain events, commit patch to undo/redo stack
        // Then execute the event
        match event {
            Event::Insert(_, ' ' | '\t') => {
                self.doc_mut().event_stack.commit();
                self.doc_mut().execute(event).unwrap();
            }
            Event::SplitDown(_)
            | Event::SpliceUp(_)
            | Event::InsertRow(_, _)
            | Event::RemoveRow(_, _) => {
                self.doc_mut().execute(event).unwrap();
                self.doc_mut().event_stack.commit();
            }
            _ => {
                self.doc_mut().execute(event).unwrap();
            }
        }
    }

    fn undo(&mut self) {
        // Undo the last change
        if let Some(patch) = self.doc_mut().event_stack.undo() {
            for event in patch.to_owned() {
                self.doc_mut().back(event).unwrap();
            }
        }
    }

    fn redo(&mut self) {
        // Redo the last change
        if let Some(patch) = self.doc_mut().event_stack.redo() {
            for event in patch.to_owned() {
                self.doc_mut().forth(event).unwrap();
            }
        }
    }

    fn quit(&mut self) {
        // Quit the editor
        self.active = false;
    }

    fn end(&mut self) {
        // Move to the end of the row
        if let Ok(row) = self.doc().current_row() {
            let len = row.len();
            self.doc_mut().goto_x(len).unwrap();
        }
    }

    fn next_tab(&mut self) {
        // Move to the next tab
        if self.doc_ptr + 1 < self.documents.len() {
            self.doc_ptr += 1;
        }
    }

    fn previous_tab(&mut self) {
        // Move to the previous tab
        if self.doc_ptr != 0 {
            self.doc_ptr -= 1;
        }
    }

    fn backspace(&mut self, loc: Loc) {
        // Handle the backspace key
        self.new_row();
        // Determine if delete or splice required
        if loc.x == 0 && loc.y != 0 {
            let x = self.doc().row(loc.y - 1).unwrap().len();
            self.exe(Event::SpliceUp(Loc { x, y: loc.y - 1 }));
        } else {
            let c = self.doc().row(loc.y).unwrap().text[loc.x - 1];
            self.exe(Event::Remove(loc, c));
        }
    }

    fn delete_row(&mut self, loc: Loc) {
        // Delete the current row
        if let Ok(row) = self.doc().row(loc.y) {
            let row = row.render_raw();
            self.exe(Event::RemoveRow(loc.y, row));
        }
    }

    fn word_jump_left(&mut self, loc: Loc) {
        // Jump to the nearest word to the left
        if let Ok(row) = self.doc().current_row() {
            let to = row.next_word_back(loc.x);
            self.doc_mut().goto_x(to).unwrap();
        }
    }

    fn word_jump_right(&mut self, loc: Loc) {
        // Jump to the nearest word to the right
        if let Ok(row) = self.doc().current_row() {
            let to = row.next_word_forth(loc.x);
            self.doc_mut().goto_x(to).unwrap();
        }
    }

    fn up(&mut self) {
        // Move the cursor up
        self.doc_mut().move_up().unwrap();
    }

    fn down(&mut self) {
        // Move the cursor down
        self.doc_mut().move_down().unwrap();
    }

    fn left(&mut self) {
        // Move the cursor to the left
        let status = self.doc_mut().move_left();
        // Wrap cursor if at the start of the row
        if let Ok(Status::StartOfRow) = status {
            // Wrap the cursor around, when moving left
            if let Ok(Status::None) = self.doc_mut().move_up() {
                // Cursor was able to be moved up, move to end of row
                self.end();
            }
        }
    }

    fn right(&mut self) {
        // Move the cursor to the right
        let status = self.doc_mut().move_right();
        // Wrap cursor if at the end of the row
        if let Ok(Status::EndOfRow) = status {
            // Wrap the cursor around, when moving right
            if let Ok(Status::None) = self.doc_mut().move_down() {
                // Cursor was able to be moved down, move to start of row
                self.doc_mut().goto_x(0).unwrap();
            }
        }
    }

    pub fn start(&mut self) -> Result<()> {
        // This will set the terminal up for rendering
        // Enter alternate screen to avoid terminal scrolling and interference
        execute!(self.stdout, EnterAlternateScreen, Clear(ClType::All))?;
        // Enable raw mode for full control of the terminal
        terminal::enable_raw_mode()?;
        Ok(())
    }

    pub fn finish(&mut self) -> Result<()> {
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
