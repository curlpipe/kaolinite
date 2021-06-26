#[cfg(test)]
use kaolinite::{
    utils::*,
    row::*,
    document::*,
    event::*,
    st,
};
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
        LINE_ENDING_SPLITTER.split("hello\nthere\n").collect::<Vec<_>>(),
        vec![st!("hello"), st!("there"), st!("")],
    );
    assert_eq!(
        LINE_ENDING_SPLITTER.split("hello\r\nthere\r\n").collect::<Vec<_>>(),
        vec![st!("hello"), st!("there"), st!("")],
    );
    assert_eq!(
        LINE_ENDING_SPLITTER.split("hello\r\nthere\na好a").collect::<Vec<_>>(),
        vec![st!("hello"), st!("there"), st!("a好a")],
    );
}

#[test]
fn test_row() {
    let mut row = Row::new("aa好b好c");
    assert_eq!(row.text, vec!['a', 'a', '好', 'b', '好', 'c']);
    assert_eq!(row.indices, vec![0, 1, 2, 4, 5, 7, 8]);
    row.insert(3, "hao").unwrap();
    row.insert(2, "ni").unwrap();
    assert_eq!(row.render_full(), "aani好haob好c");
    row.remove(3..7).unwrap();
    assert_eq!(row.render_full(), "aanob好c");
    assert_eq!(row.render(5..), "好c");
    assert_eq!(row.render(6..), " c");
    assert_eq!(row.render(7..), "c");
    let row = Row::new("The quick brown fox jumped over the lazy dog!");
    assert_eq!(row.words(), vec![0, 4, 10, 16, 20, 27, 32, 36, 41]);
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
}

#[test]
fn test_document() {
    let doc = Document::open("examples/test.txt", (10, 10)).expect("File not found");
    assert_eq!(doc.rows, vec![Row { text: vec![], indices: vec![0], modified: false }]);
    let mut doc = Document::open("examples/test2.txt", (10, 10)).expect("File not found");
    assert_eq!(doc.rows, vec![
        Row::new("My"), 
        Row::new("new好"), 
        Row::new("document"), 
        Row::new("好"),
        Row::new(""),
    ]);
    doc.row_mut(0).unwrap().insert(1, ",").unwrap();
    doc.row_mut(0).unwrap().remove(2..3).unwrap();
    assert_eq!(doc.rows, vec![
        Row { text: vec!['M', ','], indices: vec![0, 1, 2], modified: true },
        Row::new("new好"), 
        Row::new("document"), 
        Row::new("好"),
        Row::new(""),
    ]);
    assert_eq!(doc.row(1).unwrap(), &Row::new("new好"));
    doc.cursor.y = 1;
    assert_eq!(doc.current_row().unwrap(), &Row::new("new好"));
    assert_eq!(doc.current_row().unwrap().len(), 4);
    assert_eq!(doc.current_row().unwrap().width(), 5);
    assert_eq!(doc.render(), "M,\nnew好\ndocument\n好\n");
}

#[test]
fn test_movement() {
    let mut doc = Document::open("examples/test3.txt", (10, 3)).expect("File not found");
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
    let mut doc = Document::open("examples/test3.txt", (10, 3)).expect("File not found");
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
    let mut doc = Document::open("./examples/test4.txt", (10, 3)).expect("File not found");
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
    let mut doc = Document::open("examples/test2.txt", (10, 10)).expect("File not found");
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
    let mut doc = Document::open("examples/test5.txt", (10, 3)).expect("File not found");
    // Save
    assert_eq!(doc.rows, vec![Row { 
        text: vec!['h', 'e', 'l', 'l', 'o'], 
        indices: vec![0, 1, 2, 3, 4, 5], 
        modified: false 
    }]);
    doc.row_mut(0).unwrap().remove(0..3).unwrap();
    assert_eq!(doc.rows, vec![Row { 
        text: vec!['l', 'o'], 
        indices: vec![0, 1, 2], 
        modified: true
    }]);
    assert!(doc.save().is_ok());
    assert_eq!(std::fs::read_to_string(doc.info.file.as_ref().unwrap()).unwrap(), "lo");
    doc.row_mut(0).unwrap().insert(0, "hel").unwrap();
    assert!(doc.save().is_ok());
    assert_eq!(std::fs::read_to_string(doc.info.file.as_ref().unwrap()).unwrap(), "hello");
    // Save as
    assert!(!std::path::Path::new("examples/temp.txt").exists());
    assert!(doc.save_as("examples/temp.txt").is_ok());
    assert_eq!(std::fs::read_to_string("examples/temp.txt").unwrap(), st!("hello"));
    std::fs::remove_file("examples/temp.txt").unwrap();
    assert!(!std::path::Path::new("examples/temp.txt").exists());
}
