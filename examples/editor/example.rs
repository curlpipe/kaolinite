/*
	Cactus: An editor written in Rust

	It has all the features you'd need in a modern text editor
	including syntax highlighting, file type detection and
	undo and redo
*/

use std::collections::HashMap;

const NAME: &str = "Tom";

pub enum DayOfTheWeek {
	Monday, Tuesday, Wednesday,
	Thursday, Friday, Saturday,
	Sunday,
}

fn greet(name: &str) -> String {
	// Provide a greeting for a person
	format!("Hello, {}", name)
}

fn main() {
	let number = 123;
	let hashmap = HashMap::new();
	let day = DayOfTheWeek::Monday;
	println!("Hello, world!");
	println!("{}", greet(NAME));
}
