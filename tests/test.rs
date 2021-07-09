#[cfg(test)]
use kaolinite::{document::*, event::*, regex, row::*, st, utils::*};
use sugars::hmap;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

#[test]
fn test_width() {
    assert_eq!(st!("a").width(), 1);
    assert_eq!(st!("好").width(), 2);
    assert_eq!(st!("好a好").width(), 5);
    assert_eq!(st!("𒌧").width(), 1);
    assert_eq!(st!('a').width(), 1);
    assert_eq!('好'.width().unwrap_or(0), 2);
    assert_eq!('𒌧'.width().unwrap_or(0), 1);
}

#[test]
fn test_line_splitter() {
    let l = regex!("(\\r\\n|\\n)");
    assert_eq!(
        l.split("hello\nthere\n").collect::<Vec<_>>(),
        vec![st!("hello"), st!("there"), st!("")],
    );
    assert_eq!(
        l.split("hello\r\nthere\r\n").collect::<Vec<_>>(),
        vec![st!("hello"), st!("there"), st!("")],
    );
    assert_eq!(
        l.split("hello\r\nthere\na好a").collect::<Vec<_>>(),
        vec![st!("hello"), st!("there"), st!("a好a")],
    );
}

#[test]
fn test_row() {
    // Loading
    let row = Row::new("", 4);
    assert!(row.is_empty());
    let mut row = Row::new("aa好b好c", 4);
    assert!(!row.is_empty());
    assert_eq!(row.text, vec!['a', 'a', '好', 'b', '好', 'c']);
    assert_eq!(row.indices, vec![0, 1, 2, 4, 5, 7, 8]);
    // Editing
    row.insert(3, "hao", 4).unwrap();
    row.insert(2, "ni", 4).unwrap();
    assert_eq!(row.render_full(4), "aani好haob好c");
    row.remove(3..7).unwrap();
    assert_eq!(row.render_full(4), "aanob好c");
    // Bounds checking
    assert!(row.insert(10000, "nope", 4).is_err());
    assert!(row.remove(10000..482228).is_err());
    // Rendering
    assert_eq!(row.render(5.., 4), "好c");
    assert_eq!(row.render(6.., 4), " c");
    assert_eq!(row.render(7.., 4), "c");
    assert_eq!(row.render(100.., 4), "");
    let row = Row::new("aa好\tb好c", 4);
    assert_eq!(row.render_full(4), "aa好    b好c");
    assert_eq!(row.render_raw(), "aa好\tb好c");
    // Words
    let row = Row::new("The quick brown fox jumped over the lazy dog!", 4);
    assert_eq!(row.words(), vec![0, 4, 10, 16, 20, 27, 32, 36, 41, 45]);
    assert_eq!(row.next_word_back(0), 0);
    assert_eq!(row.next_word_back(29), 27);
    assert_eq!(row.next_word_back(27), 20);
    assert_eq!(row.next_word_back(48), 45);
    let row = Row::new("\tHello", 4);
    assert_eq!(row.words(), vec![0, 1, 6]);
    let row = Row::new("\t\tHel\tlo", 4);
    assert_eq!(row.words(), vec![0, 1, 2, 6, 8]);
    assert_eq!(row.next_word_forth(0), 1);
    assert_eq!(row.next_word_forth(4), 6);
    assert_eq!(row.next_word_forth(2), 6);
    assert_eq!(row.next_word_forth(10), 8);
    let row = Row::new("\t\tHel\t\tlo", 4);
    assert_eq!(row.words(), vec![0, 1, 2, 7, 9]);
    let row = Row::new(" The quick brown fox jumped over the lazy dog!", 4);
    assert_eq!(row.next_word_forth(0), 1);
    // Character pointers
    let row = Row::new("呢逆反驳船r舱s", 4);
    assert_eq!(row.get_char_ptr(0), 0);
    assert_eq!(row.get_char_ptr(2), 1);
    assert_eq!(row.get_char_ptr(4), 2);
    assert_eq!(row.get_char_ptr(6), 3);
    assert_eq!(row.get_char_ptr(8), 4);
    assert_eq!(row.get_char_ptr(10), 5);
    assert_eq!(row.get_char_ptr(11), 6);
    assert_eq!(row.get_char_ptr(13), 7);
    assert_eq!(row.get_char_ptr(14), 8);
    let row = Row::new("sr饿t肚子rsf萨t订", 4);
    assert_eq!(row.get_char_ptr(0), 0);
    assert_eq!(row.get_char_ptr(1), 1);
    assert_eq!(row.get_char_ptr(2), 2);
    assert_eq!(row.get_char_ptr(4), 3);
    assert_eq!(row.get_char_ptr(5), 4);
    assert_eq!(row.get_char_ptr(7), 5);
    assert_eq!(row.get_char_ptr(9), 6);
    assert_eq!(row.get_char_ptr(10), 7);
    assert_eq!(row.get_char_ptr(11), 8);
    assert_eq!(row.get_char_ptr(12), 9);
    assert_eq!(row.get_char_ptr(14), 10);
    assert_eq!(row.get_char_ptr(15), 11);
    assert_eq!(row.get_char_ptr(17), 12);
    // Splitting
    let row = Row::new("sr饿t肚子rsf萨t订", 4);
    assert_eq!(
        row.split(0).unwrap(),
        (
            Row {
                text: vec![],
                indices: vec![0],
                modified: true,
                tokens: vec![],
                needs_rerender: true,
            },
            Row::new("sr饿t肚子rsf萨t订", 4)
        ),
    );
    let mut dummy = Row::new("sr饿t肚子rsf萨t订", 4);
    dummy.modified = true;
    assert_eq!(row.split(12).unwrap(), (dummy, Row::new("", 4)),);
    let mut dummy = Row::new("sr饿t", 4);
    dummy.modified = true;
    assert_eq!(row.split(4).unwrap(), (dummy, Row::new("肚子rsf萨t订", 4)),);
    // Splicing
    let mut left = Row::new("sr饿t", 4);
    let right = Row::new("肚子rsf萨t订", 4);
    let mut dummy = Row::new("sr饿t肚子rsf萨t订", 4);
    dummy.modified = true;
    assert_eq!(left.splice(right), dummy);
    let mut left = Row::new("", 4);
    let right = Row::new("sr饿t肚子rsf萨t订", 4);
    let mut dummy = Row::new("sr饿t肚子rsf萨t订", 4);
    dummy.modified = true;
    assert_eq!(left.splice(right), dummy);
    let mut left = Row::new("sr饿t肚子rsf萨t订", 4);
    let right = Row::new("", 4);
    let mut dummy = Row::new("sr饿t肚子rsf萨t订", 4);
    dummy.modified = true;
    assert_eq!(left.splice(right), dummy);
}

#[test]
fn test_document() {
    // Test creation
    assert_eq!(
        FileInfo::default(),
        FileInfo {
            file: None,
            is_dos: false,
            tab_width: 4
        }
    );
    let mut doc = Document::new((10, 3));
    assert!(!doc.modified);
    doc.open("examples/test.txt").expect("File not found");
    assert!(!doc.modified);
    assert_eq!(
        doc.rows,
        vec![Row {
            text: vec![],
            indices: vec![0],
            modified: false,
            tokens: vec![],
            needs_rerender: true,
        }]
    );
    doc.open("examples/test2.txt").expect("File not found");
    assert!(!doc.modified);
    assert_eq!(
        doc.rows,
        vec![
            Row::new("My", doc.info.tab_width),
            Row::new("new好", doc.info.tab_width),
            Row::new("document", doc.info.tab_width),
            Row::new("好", doc.info.tab_width),
        ]
    );
    // Test row retrieval
    assert_eq!(
        doc.row(1).unwrap().clone(),
        Row::new("new好", doc.info.tab_width)
    );
    doc.cursor.y = 1;
    assert_eq!(
        doc.current_row().unwrap().clone(),
        Row::new("new好", doc.info.tab_width)
    );
    assert_eq!(doc.current_row().unwrap().len(), 4);
    assert_eq!(doc.current_row().unwrap().width(), 5);
    // Test editing
    doc.row_mut(0).unwrap().insert(1, ",", 4).unwrap();
    doc.row_mut(0).unwrap().remove(2..3).unwrap();
    assert!(!doc.modified);
    assert_eq!(
        doc.rows,
        vec![
            Row {
                text: vec!['M', ','],
                indices: vec![0, 1, 2],
                modified: true,
                needs_rerender: true,
                tokens: vec![],
            },
            Row::new("new好", doc.info.tab_width),
            Row::new("document", doc.info.tab_width),
            Row::new("好", doc.info.tab_width),
        ]
    );
    // Test rendering
    assert_eq!(doc.render(), "M,\nnew好\ndocument\n好\n");
    // Test goto
    doc.open("examples/test3.txt").expect("File not found");
    assert_eq!(doc.cursor, Loc { x: 0, y: 0 });
    assert_eq!(doc.offset, Loc { x: 0, y: 0 });
    assert_eq!(doc.char_ptr, 0);
    doc.goto_x(1).unwrap();
    assert_eq!(doc.cursor, Loc { x: 2, y: 0 });
    assert_eq!(doc.offset, Loc { x: 0, y: 0 });
    assert_eq!(doc.char_ptr, 1);
    doc.goto_x(13).unwrap();
    assert_eq!(doc.cursor, Loc { x: 0, y: 0 });
    assert_eq!(doc.offset, Loc { x: 24, y: 0 });
    assert_eq!(doc.char_ptr, 13);
    doc.goto_x(15).unwrap();
    assert_eq!(doc.cursor, Loc { x: 4, y: 0 });
    assert_eq!(doc.offset, Loc { x: 24, y: 0 });
    assert_eq!(doc.char_ptr, 15);
    doc.goto_x(0).unwrap();
    assert_eq!(doc.cursor, Loc { x: 0, y: 0 });
    assert_eq!(doc.offset, Loc { x: 0, y: 0 });
    assert_eq!(doc.char_ptr, 0);
    doc.goto_y(1).unwrap();
    assert_eq!(doc.cursor, Loc { x: 0, y: 1 });
    assert_eq!(doc.offset, Loc { x: 0, y: 0 });
    assert_eq!(doc.char_ptr, 0);
    doc.goto_y(6).unwrap();
    assert_eq!(doc.cursor, Loc { x: 0, y: 2 });
    assert_eq!(doc.offset, Loc { x: 0, y: 4 });
    assert_eq!(doc.char_ptr, 0);
    doc.goto_y(4).unwrap();
    assert_eq!(doc.cursor, Loc { x: 0, y: 0 });
    assert_eq!(doc.offset, Loc { x: 0, y: 4 });
    assert_eq!(doc.char_ptr, 0);
    doc.goto_y(0).unwrap();
    assert_eq!(doc.cursor, Loc { x: 0, y: 0 });
    assert_eq!(doc.offset, Loc { x: 0, y: 0 });
    assert_eq!(doc.char_ptr, 0);
    doc.goto((1, 1)).unwrap();
    assert_eq!(doc.cursor, Loc { x: 1, y: 1 });
    assert_eq!(doc.offset, Loc { x: 0, y: 0 });
    assert_eq!(doc.char_ptr, 1);
    doc.goto_y(0).unwrap();
    assert_eq!(doc.cursor, Loc { x: 0, y: 0 });
    assert_eq!(doc.offset, Loc { x: 0, y: 0 });
    assert_eq!(doc.char_ptr, 0);
    assert!(doc.goto_x(1000000).is_err());
    assert!(doc.goto_y(1000000).is_err());
    assert!(!doc.modified);
    // Check status line info
    let info = doc.status_line_info();
    let dummy = hmap! {
        "row" => st!("1"), "column" => st!("0"), "total" => st!("8"),
        "file" => st!("test3.txt"), "full_file" => st!("examples/test3.txt"),
        "type" => st!("Plain Text"), "modified" => st!(""), "extension" => st!("txt"),
    };
    assert_eq!(info, dummy);
    let mut doc = Document::new((10, 10));
    doc.cursor.x = 3;
    doc.cursor.y = 5;
    doc.char_ptr = 3;
    doc.modified = true;
    let info = doc.status_line_info();
    let dummy = hmap! {
        "row" => st!("6"), "column" => st!("3"), "total" => st!("0"),
        "file" => st!("[No Name]"), "full_file" => st!("[No Name]"),
        "type" => st!("Unknown"), "modified" => st!("[+]"), "extension" => st!(""),
    };
    assert_eq!(info, dummy);
    // Test full rendering
    doc.open("examples/test8.txt").unwrap();
    assert_eq!(doc.render_full(), "    tab\nnotab\n");
    assert_eq!(doc.render(), "\ttab\nnotab\n");
}

#[test]
fn test_movement() {
    let mut doc = Document::new((10, 3));
    doc.open("examples/test3.txt").expect("File not found");
    doc.cursor.y = 2;
    // Move left
    doc.cursor.x = 2;
    doc.offset.x = 3;
    doc.char_ptr = 5;
    assert_eq!(doc.move_left().unwrap(), Status::None);
    assert_eq!(doc.cursor.x, 1);
    assert_eq!(doc.offset.x, 3);
    assert_eq!(doc.move_left().unwrap(), Status::None);
    assert_eq!(doc.cursor.x, 0);
    assert_eq!(doc.offset.x, 3);
    assert_eq!(doc.move_left().unwrap(), Status::None);
    assert_eq!(doc.cursor.x, 0);
    assert_eq!(doc.offset.x, 2);
    assert_eq!(doc.move_left().unwrap(), Status::None);
    assert_eq!(doc.cursor.x, 0);
    assert_eq!(doc.offset.x, 1);
    assert_eq!(doc.move_left().unwrap(), Status::None);
    assert_eq!(doc.cursor.x, 0);
    assert_eq!(doc.offset.x, 0);
    assert_eq!(doc.move_left().unwrap(), Status::StartOfRow);
    assert_eq!(doc.cursor.x, 0);
    assert_eq!(doc.offset.x, 0);
    // Move right
    doc.cursor.x = 8;
    doc.offset.x = 2;
    doc.char_ptr = 10;
    assert_eq!(doc.move_right().unwrap(), Status::None);
    assert_eq!(doc.cursor.x, 9);
    assert_eq!(doc.offset.x, 2);
    assert_eq!(doc.move_right().unwrap(), Status::None);
    assert_eq!(doc.cursor.x, 9);
    assert_eq!(doc.offset.x, 3);
    assert_eq!(doc.move_right().unwrap(), Status::None);
    assert_eq!(doc.cursor.x, 9);
    assert_eq!(doc.offset.x, 4);
    assert_eq!(doc.move_right().unwrap(), Status::None);
    assert_eq!(doc.cursor.x, 9);
    assert_eq!(doc.offset.x, 5);
    assert_eq!(doc.move_right().unwrap(), Status::EndOfRow);
    assert_eq!(doc.cursor.x, 9);
    assert_eq!(doc.offset.x, 5);
    // Move up
    doc.cursor.y = 1;
    doc.offset.y = 3;
    assert_eq!(doc.move_up().unwrap(), Status::None);
    assert_eq!(doc.cursor.y, 0);
    assert_eq!(doc.offset.y, 3);
    assert_eq!(doc.move_up().unwrap(), Status::None);
    assert_eq!(doc.cursor.y, 0);
    assert_eq!(doc.offset.y, 2);
    assert_eq!(doc.move_up().unwrap(), Status::None);
    assert_eq!(doc.cursor.y, 0);
    assert_eq!(doc.offset.y, 1);
    assert_eq!(doc.move_up().unwrap(), Status::None);
    assert_eq!(doc.cursor.y, 0);
    assert_eq!(doc.offset.y, 0);
    assert_eq!(doc.move_up().unwrap(), Status::StartOfDocument);
    assert_eq!(doc.cursor.y, 0);
    assert_eq!(doc.offset.y, 0);
    // Move down
    doc.cursor.y = 1;
    doc.offset.y = 3;
    assert_eq!(doc.move_down().unwrap(), Status::None);
    assert_eq!(doc.cursor.y, 2);
    assert_eq!(doc.offset.y, 3);
    assert_eq!(doc.move_down().unwrap(), Status::None);
    assert_eq!(doc.cursor.y, 2);
    assert_eq!(doc.offset.y, 4);
    assert_eq!(doc.move_down().unwrap(), Status::None);
    assert_eq!(doc.cursor.y, 2);
    assert_eq!(doc.offset.y, 5);
    assert_ne!(doc.cursor.x, 0);
    assert_ne!(doc.offset.x, 0);
    assert_eq!(doc.move_down().unwrap(), Status::None);
    assert_eq!(doc.cursor.y, 2);
    assert_eq!(doc.offset.y, 6);
    assert_eq!(doc.cursor.x, 0);
    assert_eq!(doc.offset.x, 0);
    assert_eq!(doc.move_down().unwrap(), Status::EndOfDocument);
    assert_eq!(doc.cursor.y, 2);
    assert_eq!(doc.offset.y, 6);
    assert_eq!(doc.cursor.x, 0);
    assert_eq!(doc.offset.x, 0);
}

#[test]
fn test_unicode_safe_movement() {
    let mut doc = Document::new((10, 3));
    doc.open("examples/test3.txt").expect("File not found");
    // Ensure graphemes are correctly traversed left
    doc.cursor.x = 8;
    doc.offset.x = 3;
    doc.char_ptr = 6;
    assert_eq!(doc.move_left().unwrap(), Status::None);
    assert_eq!(doc.cursor.x, 7);
    assert_eq!(doc.offset.x, 3);
    assert_eq!(doc.char_ptr, 5);
    assert_eq!(doc.move_left().unwrap(), Status::None);
    assert_eq!(doc.cursor.x, 5);
    assert_eq!(doc.offset.x, 3);
    assert_eq!(doc.char_ptr, 4);
    assert_eq!(doc.move_left().unwrap(), Status::None);
    assert_eq!(doc.cursor.x, 3);
    assert_eq!(doc.offset.x, 3);
    assert_eq!(doc.char_ptr, 3);
    assert_eq!(doc.move_left().unwrap(), Status::None);
    assert_eq!(doc.cursor.x, 1);
    assert_eq!(doc.offset.x, 3);
    assert_eq!(doc.char_ptr, 2);
    assert_eq!(doc.move_left().unwrap(), Status::None);
    assert_eq!(doc.cursor.x, 0);
    assert_eq!(doc.offset.x, 2);
    assert_eq!(doc.char_ptr, 1);
    assert_eq!(doc.move_left().unwrap(), Status::None);
    assert_eq!(doc.cursor.x, 0);
    assert_eq!(doc.offset.x, 0);
    assert_eq!(doc.char_ptr, 0);
    assert_eq!(doc.move_left().unwrap(), Status::StartOfRow);
    assert_eq!(doc.cursor.x, 0);
    assert_eq!(doc.offset.x, 0);
    assert_eq!(doc.char_ptr, 0);
    // Ensure graphemes are correctly traversed right
    doc.cursor.x = 6;
    doc.offset.x = 0;
    doc.char_ptr = 3;
    assert_eq!(doc.move_right().unwrap(), Status::None);
    assert_eq!(doc.cursor.x, 8);
    assert_eq!(doc.offset.x, 0);
    assert_eq!(doc.char_ptr, 4);
    assert_eq!(doc.move_right().unwrap(), Status::None);
    assert_eq!(doc.cursor.x, 9);
    assert_eq!(doc.offset.x, 1);
    assert_eq!(doc.char_ptr, 5);
    assert_eq!(doc.move_right().unwrap(), Status::None);
    assert_eq!(doc.cursor.x, 9);
    assert_eq!(doc.offset.x, 2);
    assert_eq!(doc.char_ptr, 6);
    assert_eq!(doc.move_right().unwrap(), Status::None);
    assert_eq!(doc.cursor.x, 9);
    assert_eq!(doc.offset.x, 4);
    assert_eq!(doc.char_ptr, 7);
    assert_eq!(doc.move_right().unwrap(), Status::None);
    assert_eq!(doc.cursor.x, 9);
    assert_eq!(doc.offset.x, 5);
    assert_eq!(doc.char_ptr, 8);
    // When moving down, ensure the char ptr updates correctly
    doc.open("examples/test4.txt").expect("File not found");
    doc.cursor.x = 8;
    doc.char_ptr = 5;
    assert_eq!(doc.move_down().unwrap(), Status::None);
    assert_eq!(doc.cursor.x, 8);
    assert_eq!(doc.char_ptr, 6);
    // When moving up, ensure the char ptr updates correctly
    assert_eq!(doc.move_up().unwrap(), Status::None);
    assert_eq!(doc.cursor.x, 8);
    assert_eq!(doc.char_ptr, 5);
    // Check shift back when cursor moved down into middle of a double width char
    doc.cursor.y = 1;
    doc.cursor.x = 12;
    doc.char_ptr = 8;
    assert_eq!(doc.move_up().unwrap(), Status::None);
    assert_eq!(doc.cursor.y, 0);
    assert_eq!(doc.cursor.x, 11);
    assert_eq!(doc.char_ptr, 7);
    // Check shift back when cursor moved up into middle of a double width char
    assert_eq!(doc.move_down().unwrap(), Status::None);
    assert_eq!(doc.cursor.y, 1);
    assert_eq!(doc.cursor.x, 10);
    assert_eq!(doc.char_ptr, 7);
    // Check shiftback on offset
    doc.cursor.y = 0;
    doc.cursor.x = 0;
    doc.offset.x = 3;
    doc.char_ptr = 2;
    assert_eq!(doc.move_down().unwrap(), Status::None);
    assert_eq!(doc.cursor.y, 1);
    assert_eq!(doc.cursor.x, 0);
    assert_eq!(doc.offset.x, 2);
    assert_eq!(doc.char_ptr, 1);
}

#[test]
fn test_line_snapping() {
    let mut doc = Document::new((10, 10));
    doc.open("examples/test2.txt").expect("File not found");
    doc.cursor.y = 2;
    doc.cursor.x = 5;
    doc.char_ptr = 5;
    assert_eq!(doc.move_down().unwrap(), Status::None);
    assert_eq!(doc.cursor.x, 2);
    assert_eq!(doc.offset.x, 0);
    assert_eq!(doc.char_ptr, 1);
    doc.cursor.y = 1;
    doc.cursor.x = 5;
    doc.char_ptr = 5;
    assert_eq!(doc.move_up().unwrap(), Status::None);
    assert_eq!(doc.cursor.x, 2);
    assert_eq!(doc.offset.x, 0);
    assert_eq!(doc.char_ptr, 2);
}

#[test]
fn test_save() {
    let mut doc = Document::new((10, 3));
    doc.open("examples/test5.txt").expect("File not found");
    assert!(!doc.modified);
    // Save
    assert_eq!(
        doc.rows,
        vec![Row {
            text: vec!['h', 'e', 'l', 'l', 'o'],
            indices: vec![0, 1, 2, 3, 4, 5],
            modified: false,
            tokens: vec![],
            needs_rerender: true,
        }]
    );
    doc.execute(Event::Remove((2, 0).into(), 'e')).unwrap();
    assert_eq!(
        doc.rows,
        vec![Row {
            text: vec!['h', 'l', 'l', 'o'],
            indices: vec![0, 1, 2, 3, 4],
            modified: true,
            tokens: vec![],
            needs_rerender: true,
        }]
    );
    assert!(doc.modified);
    assert!(doc.save().is_ok());
    assert!(!doc.modified);
    assert_eq!(
        std::fs::read_to_string(doc.info.file.as_ref().unwrap()).unwrap(),
        "hllo\n"
    );
    doc.execute(Event::Insert((1, 0).into(), 'e')).unwrap();
    assert!(doc.modified);
    assert!(doc.save().is_ok());
    assert!(!doc.modified);
    assert_eq!(
        std::fs::read_to_string(doc.info.file.as_ref().unwrap()).unwrap(),
        "hello\n"
    );
    // Save as
    assert!(!std::path::Path::new("examples/temp.txt").exists());
    assert!(doc.save_as("examples/temp.txt").is_ok());
    assert_eq!(
        std::fs::read_to_string("examples/temp.txt").unwrap(),
        st!("hello\n")
    );
    std::fs::remove_file("examples/temp.txt").unwrap();
    assert!(!std::path::Path::new("examples/temp.txt").exists());
}

#[test]
fn test_tab() {
    let mut doc = Document::new((10, 3));
    doc.info.tab_width = 2;
    doc.open("examples/test6.txt").expect("File not found");
    println!("{}", doc.info.tab_width);
    assert_eq!(doc.render(), st!("\thello\n    hello\n"));
    println!("{}", doc.info.tab_width);
    assert_eq!(doc.row(0).unwrap().indices, vec![0, 2, 3, 4, 5, 6, 7]);
    let map = hmap! { 0 => 0, 1 => 1, 2 => 2, 3 => 3, 4 => 4, 5 => 5, 6 => 6 };
    assert_eq!(raw_indices("\thi", &[0, 4, 5, 6], 4), map);
}

#[test]
fn test_execution() {
    let loc: Loc = (2, 4).into();
    assert_eq!(loc, Loc { x: 2, y: 4 });
    let mut doc = Document::new((10, 3));
    assert_eq!(
        doc.execute(Event::InsertRow(0, st!(""))).unwrap(),
        Status::None
    );
    assert_eq!(
        doc.execute(Event::InsertRow(1, st!(""))).unwrap(),
        Status::None
    );
    assert_eq!(doc.render(), st!("\n\n"));
    assert_eq!(doc.loc(), Loc { x: 0, y: 1 });
    doc.goto((0, 0)).unwrap();
    assert_eq!(
        doc.execute(Event::Insert((0, 1).into(), '!')).unwrap(),
        Status::None
    );
    assert_eq!(doc.render(), st!("\n!\n"));
    assert_eq!(doc.loc(), Loc { x: 1, y: 1 });
    doc.goto((0, 0)).unwrap();
    assert_eq!(
        doc.execute(Event::Insert((0, 1).into(), ':')).unwrap(),
        Status::None
    );
    assert_eq!(doc.render(), st!("\n:!\n"));
    assert_eq!(doc.loc(), Loc { x: 1, y: 1 });
    doc.goto((0, 0)).unwrap();
    assert_eq!(
        doc.execute(Event::Insert((0, 0).into(), 'q')).unwrap(),
        Status::None
    );
    assert_eq!(doc.render(), st!("q\n:!\n"));
    assert_eq!(doc.loc(), Loc { x: 1, y: 0 });
    doc.goto((0, 0)).unwrap();
    assert_eq!(
        doc.execute(Event::Insert((1, 0).into(), 'x')).unwrap(),
        Status::None
    );
    assert_eq!(doc.render(), st!("qx\n:!\n"));
    assert_eq!(doc.loc(), Loc { x: 2, y: 0 });
    doc.goto((0, 0)).unwrap();
    assert_eq!(
        doc.execute(Event::SpliceUp((2, 0).into())).unwrap(),
        Status::None
    );
    assert_eq!(doc.render(), st!("qx:!\n"));
    assert_eq!(doc.loc(), Loc { x: 2, y: 0 });
    doc.goto((0, 0)).unwrap();
    assert_eq!(
        doc.execute(Event::SplitDown((2, 0).into())).unwrap(),
        Status::None
    );
    assert_eq!(doc.render(), st!("qx\n:!\n"));
    assert_eq!(doc.loc(), Loc { x: 0, y: 1 });
    doc.goto((0, 0)).unwrap();
    assert_eq!(
        doc.execute(Event::RemoveRow(1, st!(":!\n"))).unwrap(),
        Status::None
    );
    assert_eq!(doc.render(), st!("qx\n"));
    assert_eq!(doc.loc(), Loc { x: 0, y: 1 });
    doc.goto((0, 0)).unwrap();
    assert_eq!(
        doc.execute(Event::Remove((1, 0).into(), 'q')).unwrap(),
        Status::None
    );
    assert_eq!(doc.render(), st!("x\n"));
    assert_eq!(doc.loc(), Loc { x: 0, y: 0 });
    assert_eq!(
        doc.execute(Event::Remove((0, 0).into(), ' ')).unwrap(),
        Status::StartOfRow
    );
    assert_eq!(doc.render(), st!("x\n"));
    assert_eq!(doc.loc(), Loc { x: 0, y: 0 });
}

#[test]
fn test_filetype() {
    assert_eq!(filetype("asm").unwrap(), st!("Assembly"));
    assert_eq!(filetype("glsl").unwrap(), st!("GLSL"));
    assert_eq!(filetype("py").unwrap(), st!("Python"));
    assert_eq!(filetype("toml").unwrap(), st!("TOML"));
    assert_eq!(filetype("zsh").unwrap(), st!("Zsh"));
    assert!(filetype("xyz").is_none());
}

#[test]
fn test_line_numbers() {
    let mut doc = Document::new((10, 3));
    doc.open("examples/test7.txt").unwrap();
    assert_eq!(doc.line_number(0), st!(" 1"));
    assert_eq!(doc.line_number(20), st!("21"));
}

#[test]
fn test_alignment() {
    assert_eq!(align_middle("hel\tlo!", 10, 2).unwrap(), st!(" hel\tlo! "));
    assert_eq!(align_middle("hel\tlo!", 8, 2).unwrap(), st!("hel\tlo!"));
    assert!(align_middle("hel\tlo!", 5, 2).is_none());
    assert_eq!(
        align_sides(" hel\tl", "o! ", 15, 4).unwrap(),
        st!(" hel\tl   o! ")
    );
    assert_eq!(
        align_sides("hel\tl", "o!", 15, 4).unwrap(),
        st!("hel\tl     o!")
    );
    assert_eq!(align_sides("hel\tl", "o!", 10, 4).unwrap(), st!("hel\tlo!"));
    assert!(align_sides(" hel\tl", "o! ", 10, 4).is_none());
}

#[test]
fn test_event_stack() {
    let mut events = EditStack::default();
    events.exe(Event::Insert(Loc { x: 0, y: 0 }, 'e'));
    events.exe(Event::Insert(Loc { x: 1, y: 0 }, 'g'));
    events.exe(Event::Insert(Loc { x: 2, y: 0 }, 'g'));
    assert_eq!(
        events.patch,
        vec![
            Event::Insert(Loc { x: 0, y: 0 }, 'e'),
            Event::Insert(Loc { x: 1, y: 0 }, 'g'),
            Event::Insert(Loc { x: 2, y: 0 }, 'g'),
        ]
    );
    events.commit();
    assert!(events.patch.is_empty());
    assert_eq!(
        events.done,
        vec![vec![
            Event::Insert(Loc { x: 0, y: 0 }, 'e'),
            Event::Insert(Loc { x: 1, y: 0 }, 'g'),
            Event::Insert(Loc { x: 2, y: 0 }, 'g'),
        ],]
    );
    assert_eq!(
        events.undo().unwrap(),
        &vec![
            Event::Insert(Loc { x: 2, y: 0 }, 'g'),
            Event::Insert(Loc { x: 1, y: 0 }, 'g'),
            Event::Insert(Loc { x: 0, y: 0 }, 'e'),
        ]
    );
    assert_eq!(
        events.redo().unwrap(),
        &vec![
            Event::Insert(Loc { x: 0, y: 0 }, 'e'),
            Event::Insert(Loc { x: 1, y: 0 }, 'g'),
            Event::Insert(Loc { x: 2, y: 0 }, 'g'),
        ]
    );
}

#[test]
fn test_undoredo() {
    let mut doc = Document::new((10, 3));
    doc.open("examples/test4.txt").unwrap();
    assert_eq!(
        doc.render(),
        st!("船r舱s和蔼t教案汉代t\n句句stdt实话t拦trs截\n")
    );
    // Insert
    doc.execute(Event::Insert((0, 1).into(), 'd')).unwrap();
    doc.execute(Event::Insert((1, 1).into(), 'i')).unwrap();
    doc.execute(Event::Insert((2, 1).into(), 'e')).unwrap();
    doc.event_stack.commit();
    doc.execute(Event::Insert((3, 1).into(), ' ')).unwrap();
    assert_eq!(
        doc.render(),
        st!("船r舱s和蔼t教案汉代t\ndie 句句stdt实话t拦trs截\n")
    );
    // Remove
    doc.execute(Event::Remove((4, 1).into(), ' ')).unwrap();
    assert_eq!(
        doc.render(),
        st!("船r舱s和蔼t教案汉代t\ndie句句stdt实话t拦trs截\n")
    );
    // Insert and delete row
    doc.execute(Event::InsertRow(2, st!("hi"))).unwrap();
    assert_eq!(
        doc.render(),
        st!("船r舱s和蔼t教案汉代t\ndie句句stdt实话t拦trs截\nhi\n")
    );
    doc.execute(Event::RemoveRow(1, st!("die句句stdt实话t拦trs截")))
        .unwrap();
    assert_eq!(doc.render(), st!("船r舱s和蔼t教案汉代t\nhi\n"));
    // Splice up
    doc.execute(Event::SpliceUp((12, 0).into())).unwrap();
    assert_eq!(doc.render(), st!("船r舱s和蔼t教案汉代thi\n"));
    // Split down
    doc.execute(Event::SplitDown((3, 0).into())).unwrap();
    assert_eq!(doc.render(), st!("船r舱\ns和蔼t教案汉代thi\n"));

    doc.back(Event::SplitDown((3, 0).into())).unwrap();
    assert_eq!(doc.render(), st!("船r舱s和蔼t教案汉代thi\n"));
    doc.back(Event::SpliceUp((12, 0).into())).unwrap();
    assert_eq!(doc.render(), st!("船r舱s和蔼t教案汉代t\nhi\n"));
    doc.back(Event::RemoveRow(1, st!("die句句stdt实话t拦trs截")))
        .unwrap();
    assert_eq!(
        doc.render(),
        st!("船r舱s和蔼t教案汉代t\ndie句句stdt实话t拦trs截\nhi\n")
    );
    doc.back(Event::InsertRow(2, st!("hi"))).unwrap();
    assert_eq!(
        doc.render(),
        st!("船r舱s和蔼t教案汉代t\ndie句句stdt实话t拦trs截\n")
    );
    doc.back(Event::Remove((4, 1).into(), ' ')).unwrap();
    assert_eq!(
        doc.render(),
        st!("船r舱s和蔼t教案汉代t\ndie 句句stdt实话t拦trs截\n")
    );
    doc.back(Event::Insert((3, 1).into(), ' ')).unwrap();
    doc.back(Event::Insert((2, 1).into(), 'e')).unwrap();
    doc.back(Event::Insert((1, 1).into(), 'i')).unwrap();
    doc.back(Event::Insert((0, 1).into(), 'd')).unwrap();
    assert_eq!(
        doc.render(),
        st!("船r舱s和蔼t教案汉代t\n句句stdt实话t拦trs截\n")
    );
}
