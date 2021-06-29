use kaolinite::document::Document;
use kaolinite::event::Event;
use kaolinite::st;
use std::time::Instant as Inst;

fn main() {
    let mut doc = Document::new((10, 3));
    // Editing operations
    let start = Inst::now();
    for _ in 0..100000 {
        doc.execute(Event::InsertRow(0, st!("h"))).unwrap();
        doc.execute(Event::Insert((1, 0).into(), 'i')).unwrap();
        doc.execute(Event::SplitDown((1, 0).into())).unwrap();
        doc.execute(Event::SpliceUp((1, 0).into())).unwrap();
        doc.execute(Event::Remove((0, 0).into(), 'h')).unwrap();
        doc.execute(Event::RemoveRow(0, st!("h"))).unwrap();
    }
    let end = Inst::now();
    /*
    0.1.0: 302ns
    0.1.1: 300ns
    */
    println!("Editing operation avg: {:?}", (end - start) / 600000);
    // Document opening
    let start = Inst::now();
    doc.open("./examples/big.txt").unwrap();
    let end = Inst::now();
    println!("Big file open: {:?}", end - start);
    /*
    0.1.0: 294ms
    0.1.1: 295ms
    */
}
