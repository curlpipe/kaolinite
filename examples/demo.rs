use kaolinite::document::Document;
use kaolinite::event::Result;
use unicode_width::UnicodeWidthStr;

fn main() -> Result<()> {
    // Open a file into document
    let mut document = Document::new((10, 10));
    document.open("examples/test.txt")?;
    // Obtain a mutable reference to the first row in the document
    let first = document.row_mut(0)?;
    // Apply some operations
    first.insert(0, "Hello, world! 好好")?;
    first.remove(5..12)?;
    first.remove(5..8)?;
    first.insert(2, "好")?;
    // Print the row
    let len = first.render_full().width();
    for s in 0..=len {
        println!("{:?}", first.render(s..));
    }
    // Check the widths
    println!("{:?}", first.text);
    println!("{:?}", first.indices);
    Ok(())
}
