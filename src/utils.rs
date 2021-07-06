use crate::row::Row;
use std::collections::HashMap;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Whitespace character array
const WHITESPACE: [char; 2] = [' ', '\t'];

/// String helper macro
#[macro_export]
macro_rules! st {
    ($value:expr) => {
        $value.to_string()
    };
}

/// Lazy regex creation
#[macro_export]
macro_rules! regex {
    ($re:literal $(,)?) => {{
        static RE: once_cell::sync::OnceCell<regex::Regex> = once_cell::sync::OnceCell::new();
        RE.get_or_init(|| regex::Regex::new($re).unwrap())
    }};
}

/// A struct that holds positions
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Loc {
    pub x: usize,
    pub y: usize,
}

impl From<(usize, usize)> for Loc {
    fn from(loc: (usize, usize)) -> Loc {
        let (x, y) = loc;
        Loc { x, y }
    }
}

/// A struct that holds size
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Size {
    pub w: usize,
    pub h: usize,
}

impl From<(usize, usize)> for Size {
    fn from(size: (usize, usize)) -> Size {
        let (w, h) = size;
        Size { w, h }
    }
}

pub trait BoundedRange {
    fn first(&self) -> usize;
    fn last(&self) -> usize;
}

impl BoundedRange for std::ops::Range<usize> {
    fn first(&self) -> usize {
        self.start
    }

    fn last(&self) -> usize {
        self.end
    }
}

impl BoundedRange for std::ops::RangeInclusive<usize> {
    fn first(&self) -> usize {
        *self.start()
    }

    fn last(&self) -> usize {
        *self.end() + 1
    }
}

/// Generate a look up table between the raw and display indices
#[must_use]
pub fn raw_indices(s: &str, i: &[usize], tab_width: usize) -> HashMap<usize, usize> {
    let mut raw = 0;
    let mut indices = HashMap::new();
    indices.insert(0, 0);
    for (c, ch) in s.chars().enumerate() {
        if ch == '\t' {
            for i in 1..=tab_width {
                indices.insert(c + i, raw + i);
            }
            raw += 4;
        } else {
            raw += ch.len_utf8();
            indices.insert(i[c + 1], raw);
        }
    }
    indices
}

/// Retrieve the indices of word boundaries
#[must_use]
pub fn words(row: &Row) -> Vec<usize> {
    // Gather information and set up algorithm
    let mut result = vec![];
    let mut chr = 0;
    let mut pad = true;
    // While still inside the row
    while chr < row.text.len() {
        let c = row.text[chr];
        match c {
            // Move forward through all the spaces
            ' ' => (),
            '\t' => {
                // If we haven't encountered text yet
                if pad {
                    // Make this a word boundary
                    result.push(chr);
                }
            }
            _ => {
                // Set the marker to false, as we're encountering text
                pad = false;
                // Set this as a word boundary
                result.push(chr);
                // Skip through text, end when we find whitespace or the end of the row
                while chr < row.text.len() && !WHITESPACE.contains(&row.text[chr]) {
                    chr += 1;
                }
                // Deal with next lot of whitespace or exit if at the end of the row
                continue;
            }
        }
        // Advance and continue
        chr += 1;
    }
    // Add on the last point on the row as a word boundary
    result.push(row.len());
    result
}

/// Determine the display width of a string
#[must_use]
pub fn width(s: &str, tab: usize) -> usize {
    let s = s.replace('\t', &" ".repeat(tab));
    s.width()
}

/// Determine the display width of a character
#[must_use]
pub fn width_char(c: char, tab: usize) -> usize {
    if c == '\t' {
        tab
    } else {
        c.width().unwrap_or(0)
    }
}

/// Determine the filetype from the extension
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn filetype(extension: &str) -> Option<String> {
    Some(st!(match extension.to_ascii_lowercase().as_str() {
        "abap" => "ABAP",
        "ada" => "Ada",
        "ahk" | "ahkl" => "AutoHotkey",
        "applescript" | "scpt" => "AppleScript",
        "arc" => "Arc",
        "asp" | "asax" | "ascx" | "ashx" | "asmx" | "aspx" | "axd" => "ASP",
        "as" => "ActionScript",
        "asc" | "ash" => "AGS Script",
        "asm" | "nasm" => "Assembly",
        "awk" | "auk" | "gawk" | "mawk" | "nawk" => "Awk",
        "bat" | "cmd" => "Batch",
        "b" | "bf" => "Brainfuck",
        "c" => "C",
        "cmake" => "CMake",
        "cbl" | "cobol" | "cob" => "Cobol",
        "class" | "java" => "Java",
        "clj" | "cl2" | "cljs" | "cljx" | "cljc" => "Clojure",
        "coffee" => "CoffeeScript",
        "cr" => "Crystal",
        "cu" | "cuh" => "Cuda",
        "cpp" | "cxx" => "C++",
        "cs" | "cshtml" | "csx" => "C#",
        "css" => "CSS",
        "csv" => "CSV",
        "d" | "di" => "D",
        "dart" => "Dart",
        "diff" | "patch" => "Diff",
        "dockerfile" => "Dockerfile",
        "ex" | "exs" => "Elixr",
        "elm" => "Elm",
        "el" => "Emacs Lisp",
        "erb" => "ERB",
        "erl" | "es" => "Erlang",
        "fs" | "fsi" | "fsx" => "F#",
        "f" | "f90" | "fpp" | "for" => "FORTRAN",
        "fish" => "Fish",
        "fth" => "Forth",
        "g4" => "ANTLR",
        "gd" => "GDScript",
        "glsl" | "vert" | "shader" | "geo" | "fshader" | "vrx" | "vsh" | "vshader" | "frag" =>
            "GLSL",
        "gnu" | "gp" | "plot" => "Gnuplot",
        "go" => "Go",
        "groovy" | "gvy" => "Groovy",
        "hlsl" => "HLSL",
        "h" => "C Header",
        "haml" => "Haml",
        "handlebars" | "hbs" => "Handlebars",
        "hs" => "Haskell",
        "hpp" => "C++ Header",
        "html" | "htm" | "xhtml" => "HTML",
        "ini" | "cfg" => "INI",
        "ino" => "Arduino",
        "ijs" => "J",
        "json" => "JSON",
        "jsx" => "JSX",
        "js" => "JavaScript",
        "jl" => "Julia",
        "kt" | "ktm" | "kts" => "Kotlin",
        "ll" => "LLVM",
        "l" | "lex" => "Lex",
        "lua" => "Lua",
        "ls" => "LiveScript",
        "lol" => "LOLCODE",
        "lisp" | "asd" | "lsp" => "Common Lisp",
        "log" => "Log file",
        "m4" => "M4",
        "man" | "roff" => "Groff",
        "matlab" => "Matlab",
        "m" => "Objective-C",
        "ml" => "OCaml",
        "mk" | "mak" => "Makefile",
        "md" | "markdown" => "Markdown",
        "nix" => "Nix",
        "numpy" => "NumPy",
        "opencl" | "cl" => "OpenCL",
        "php" => "PHP",
        "pas" => "Pascal",
        "pl" => "Perl",
        "psl" => "PowerShell",
        "pro" => "Prolog",
        "py" | "pyw" => "Python",
        "pyx" | "pxd" | "pxi" => "Cython",
        "r" => "R",
        "rst" => "reStructuredText",
        "rkt" => "Racket",
        "rb" | "ruby" => "Ruby",
        "rs" => "Rust",
        "sh" => "Shell",
        "scss" => "SCSS",
        "sql" => "SQL",
        "sass" => "Sass",
        "scala" => "Scala",
        "scm" => "Scheme",
        "st" => "Smalltalk",
        "swift" => "Swift",
        "toml" => "TOML",
        "tcl" => "Tcl",
        "tex" => "TeX",
        "ts" | "tsx" => "TypeScript",
        "txt" => "Plain Text",
        "vala" => "Vala",
        "vb" | "vbs" => "Visual Basic",
        "vue" => "Vue",
        "xm" | "x" | "xi" => "Logos",
        "xml" => "XML",
        "y" | "yacc" => "Yacc",
        "yaml" | "yml" => "Yaml",
        "yxx" => "Bison",
        "zsh" => "Zsh",
        _ => return None,
    }))
}
