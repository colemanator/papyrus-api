#![allow(unused)]
use std::io::{self, BufReader};
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use std::error::Error;
use std::env;
use std::vec::Vec;

#[derive(Debug)]
 struct Verse {
    book: u8,
    chapter: u8,
    verse: u8,
    text: String
 }

fn main() -> io::Result<()> {
    let path = Path::new("src/data/t_asv.csv");
    let display = path.display();

    let file = match File::open(&path) {
        // The `description` method of `io::Error` returns a string that
        // describes the error
        Err(why) => panic!("couldn't open {}: {}", display, why.description()),
        Ok(file) => file,
    };

    let f = BufReader::new(file);

    let mut verses = Vec::new();

    for line in f.lines() {
        let line = line.unwrap();
        let verse_data: Vec<&str> = line.split(",").collect();

        let book: u8 = match verse_data[1].trim().parse() {
            Ok(num) => num,
            Err(_) => continue,
        };

        let chapter: u8 = match verse_data[2].trim().parse() {
            Ok(num) => num,
            Err(_) => continue,
        };

        let verse: u8 = match verse_data[3].trim().parse() {
            Ok(num) => num,
            Err(_) => continue,
        };

        verses.push(
            Verse { book, chapter, verse, text: verse_data[4].to_string() }
        );
    }

    println!("The number is {}", verses.len());
    Ok(())
}