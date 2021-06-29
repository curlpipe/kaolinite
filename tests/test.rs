#[cfg(test)]
use kaolinite::{document::*, event::*, row::*, st, utils::*};
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
    assert_eq!(
        LINE_ENDING_SPLITTER
            .split("hello\nthere\n")
            .collect::<Vec<_>>(),
        vec![st!("hello"), st!("there"), st!("")],
    );
    assert_eq!(
        LINE_ENDING_SPLITTER
            .split("hello\r\nthere\r\n")
            .collect::<Vec<_>>(),
        vec![st!("hello"), st!("there"), st!("")],
    );
    assert_eq!(
        LINE_ENDING_SPLITTER
            .split("hello\r\nthere\na好a")
            .collect::<Vec<_>>(),
        vec![st!("hello"), st!("there"), st!("a好a")],
    );
}

#[test]
fn test_row() {
    // Loading
    let mut row = Row::new("aa好b好c");
    assert_eq!(row.text, vec!['a', 'a', '好', 'b', '好', 'c']);
    assert_eq!(row.indices, vec![0, 1, 2, 4, 5, 7, 8]);
    // Editing
    row.insert(3, "hao").unwrap();
    row.insert(2, "ni").unwrap();
    assert_eq!(row.render_full(), "aani好haob好c");
    row.remove(3..7).unwrap();
    assert_eq!(row.render_full(), "aanob好c");
    // Bounds checking
    assert!(row.insert(10000, "nope").is_err());
    assert!(row.remove(10000..482228).is_err());
    // Rendering
    assert_eq!(row.render(5..), "好c");
    assert_eq!(row.render(6..), " c");
    assert_eq!(row.render(7..), "c");
    assert_eq!(row.render(100..), "");
    let mut row = Row::new("aa好\tb好c");
    assert_eq!(row.render_full(), "aa好    b好c");
    assert_eq!(row.render_raw(), "aa好\tb好c");
    // Words
    let row = Row::new("The quick brown fox jumped over the lazy dog!");
    assert_eq!(row.words(), vec![0, 4, 10, 16, 20, 27, 32, 36, 41, 45]);
    let row = Row::new("\tHello");
    assert_eq!(row.words(), vec![0, 4, 9]);
    let row = Row::new("\t\tHel\tlo");
    assert_eq!(row.words(), vec![0, 4, 8, 15, 17]);
    // Character pointers
    let row = Row::new("呢逆反驳船r舱s");
    assert_eq!(row.get_char_ptr(0), 0);
    assert_eq!(row.get_char_ptr(2), 1);
    assert_eq!(row.get_char_ptr(4), 2);
    assert_eq!(row.get_char_ptr(6), 3);
    assert_eq!(row.get_char_ptr(8), 4);
    assert_eq!(row.get_char_ptr(10), 5);
    assert_eq!(row.get_char_ptr(11), 6);
    assert_eq!(row.get_char_ptr(13), 7);
    assert_eq!(row.get_char_ptr(14), 8);
    let row = Row::new("sr饿t肚子rsf萨t订");
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
    let row = Row::new("sr饿t肚子rsf萨t订");
    assert_eq!(
        row.split(0).unwrap(),
        (
            Row {
                text: vec![],
                indices: vec![0],
                modified: true,
                info: std::ptr::null_mut()
            },
            Row::new("sr饿t肚子rsf萨t订")
        ),
    );
    let mut dummy = Row::new("sr饿t肚子rsf萨t订");
    dummy.modified = true;
    assert_eq!(row.split(12).unwrap(), (dummy, Row::new("")),);
    let mut dummy = Row::new("sr饿t");
    dummy.modified = true;
    assert_eq!(row.split(4).unwrap(), (dummy, Row::new("肚子rsf萨t订")),);
    // Splicing
    let mut left = Row::new("sr饿t");
    let right = Row::new("肚子rsf萨t订");
    let mut dummy = Row::new("sr饿t肚子rsf萨t订");
    dummy.modified = true;
    assert_eq!(left.splice(right), dummy);
    let mut left = Row::new("");
    let right = Row::new("sr饿t肚子rsf萨t订");
    let mut dummy = Row::new("sr饿t肚子rsf萨t订");
    dummy.modified = true;
    assert_eq!(left.splice(right), dummy);
    let mut left = Row::new("sr饿t肚子rsf萨t订");
    let right = Row::new("");
    let mut dummy = Row::new("sr饿t肚子rsf萨t订");
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
            info: &mut doc.info
        }]
    );
    doc.open("examples/test2.txt").expect("File not found");
    assert!(!doc.modified);
    assert_eq!(
        doc.rows,
        vec![
            Row::new("My").link(&mut doc.info),
            Row::new("new好").link(&mut doc.info),
            Row::new("document").link(&mut doc.info),
            Row::new("好").link(&mut doc.info),
            Row::new("").link(&mut doc.info),
        ]
    );
    // Test row retrieval
    assert_eq!(
        doc.row(1).unwrap().clone(),
        Row::new("new好").link(&mut doc.info)
    );
    doc.cursor.y = 1;
    assert_eq!(
        doc.current_row().unwrap().clone(),
        Row::new("new好").link(&mut doc.info)
    );
    assert_eq!(doc.current_row().unwrap().len(), 4);
    assert_eq!(doc.current_row().unwrap().width(), 5);
    // Test editing
    doc.row_mut(0).unwrap().insert(1, ",").unwrap();
    doc.row_mut(0).unwrap().remove(2..3).unwrap();
    assert!(!doc.modified);
    assert_eq!(
        doc.rows,
        vec![
            Row {
                text: vec!['M', ','],
                indices: vec![0, 1, 2],
                modified: true,
                info: &mut doc.info
            },
            Row::new("new好").link(&mut doc.info),
            Row::new("document").link(&mut doc.info),
            Row::new("好").link(&mut doc.info),
            Row::new("").link(&mut doc.info),
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
            info: &mut doc.info,
        }]
    );
    doc.execute(Event::Remove((1, 0).into(), 'e')).unwrap();
    assert_eq!(
        doc.rows,
        vec![Row {
            text: vec!['h', 'l', 'l', 'o'],
            indices: vec![0, 1, 2, 3, 4],
            modified: true,
            info: &mut doc.info,
        }]
    );
    assert!(doc.modified);
    assert!(doc.save().is_ok());
    assert!(!doc.modified);
    assert_eq!(
        std::fs::read_to_string(doc.info.file.as_ref().unwrap()).unwrap(),
        "hllo"
    );
    doc.execute(Event::Insert((1, 0).into(), 'e')).unwrap();
    assert!(doc.modified);
    assert!(doc.save().is_ok());
    assert!(!doc.modified);
    assert_eq!(
        std::fs::read_to_string(doc.info.file.as_ref().unwrap()).unwrap(),
        "hello"
    );
    // Save as
    assert!(!std::path::Path::new("examples/temp.txt").exists());
    assert!(doc.save_as("examples/temp.txt").is_ok());
    assert_eq!(
        std::fs::read_to_string("examples/temp.txt").unwrap(),
        st!("hello")
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
    assert_eq!(doc.render(), st!("\thello\n    hello"));
    println!("{}", doc.info.tab_width);
    assert_eq!(doc.row(0).unwrap().indices, vec![0, 2, 3, 4, 5, 6, 7]);
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
    assert_eq!(doc.render(), st!("\n"));
    assert_eq!(doc.loc(), Loc { x: 0, y: 1 });
    doc.goto((0, 0)).unwrap();
    assert_eq!(
        doc.execute(Event::Insert((0, 1).into(), '!')).unwrap(),
        Status::None
    );
    assert_eq!(doc.render(), st!("\n!"));
    assert_eq!(doc.loc(), Loc { x: 1, y: 1 });
    doc.goto((0, 0)).unwrap();
    assert_eq!(
        doc.execute(Event::Insert((0, 1).into(), ':')).unwrap(),
        Status::None
    );
    assert_eq!(doc.render(), st!("\n:!"));
    assert_eq!(doc.loc(), Loc { x: 1, y: 1 });
    doc.goto((0, 0)).unwrap();
    assert_eq!(
        doc.execute(Event::Insert((0, 0).into(), 'q')).unwrap(),
        Status::None
    );
    assert_eq!(doc.render(), st!("q\n:!"));
    assert_eq!(doc.loc(), Loc { x: 1, y: 0 });
    doc.goto((0, 0)).unwrap();
    assert_eq!(
        doc.execute(Event::Insert((1, 0).into(), 'x')).unwrap(),
        Status::None
    );
    assert_eq!(doc.render(), st!("qx\n:!"));
    assert_eq!(doc.loc(), Loc { x: 2, y: 0 });
    doc.goto((0, 0)).unwrap();
    assert_eq!(
        doc.execute(Event::SpliceUp((2, 0).into())).unwrap(),
        Status::None
    );
    assert_eq!(doc.render(), st!("qx:!"));
    assert_eq!(doc.loc(), Loc { x: 2, y: 0 });
    doc.goto((0, 0)).unwrap();
    assert_eq!(
        doc.execute(Event::SplitDown((2, 0).into())).unwrap(),
        Status::None
    );
    assert_eq!(doc.render(), st!("qx\n:!"));
    assert_eq!(doc.loc(), Loc { x: 0, y: 1 });
    doc.goto((0, 0)).unwrap();
    assert_eq!(
        doc.execute(Event::RemoveRow(1, st!(":!"))).unwrap(),
        Status::None
    );
    assert_eq!(doc.render(), st!("qx"));
    assert_eq!(doc.loc(), Loc { x: 0, y: 0 });
    doc.goto((0, 0)).unwrap();
    assert_eq!(
        doc.execute(Event::Remove((1, 0).into(), 'x')).unwrap(),
        Status::None
    );
    assert_eq!(doc.render(), st!("q"));
    assert_eq!(doc.loc(), Loc { x: 0, y: 0 });
}
