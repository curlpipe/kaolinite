use kaolinite::document::Document;
use kaolinite::event::Event;
use kaolinite::st;
use std::time::Instant as Inst;

fn main() {
    let mut doc = Document::new((10, 3));
    // Editing operations
    let start = Inst::now();
    for _ in 0..1000 {
        doc.execute(Event::InsertRow(0, st!("hello world this is a benchmark 1234 I'm trying to make this line as long as possible to test how it reacts with very very very long lines because longer lines make the algorithm slower!"))).unwrap();
        doc.execute(Event::Insert((1, 0).into(), 'i')).unwrap();
        doc.execute(Event::SplitDown((1, 0).into())).unwrap();
        doc.execute(Event::SpliceUp((1, 0).into())).unwrap();
        doc.execute(Event::Remove((0, 0).into(), 'h')).unwrap();
        doc.execute(Event::RemoveRow(0, st!("hello world this is a benchmark 1234 I'm trying to make this line as long as possible to test how it reacts with very very very long lines because longer lines make the algorithm slower!"))).unwrap();
    }
    let end = Inst::now();
    println!("Editing operation avg: {:?}", (end - start) / 6000);
    // Document opening
    let start = Inst::now();
    doc.open("./examples/big.txt").unwrap();
    let end = Inst::now();
    println!("Big file open: {:?}", end - start);
}
