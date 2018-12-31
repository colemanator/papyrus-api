//! # Papyrus
//! A micro-service for querying the bible using approximate string searching.

#![allow(unused)]
extern crate csv;
extern crate unicode_normalization;
extern crate caseless;

mod bible;
mod normalise;
mod search;

use std::vec::Vec;
use std::process;
use std::time::Instant;
use bible::Bible;
use normalise::normalise_text;
use search::search;
use std::io;

fn main() -> io::Result<()> {

    let bible = match Bible::new("src/data/t_asv.csv") {
        Ok(bible) => bible,
        Err(_) => process::exit(1)
    };

    let verses = bible.get_verses();

    loop {
        // 1. Ask for each string
        println!("\nSearch verses: ");

        // 2. Read input
        let mut query = String::new();
        io::stdin().read_line(&mut query)
            .expect("Failed to read line");

        // 5. If no input was given exit the program
        if query.chars().count() == 0  {
            process::exit(1);
        }

        let now = Instant::now();
        let matches = search(query, &verses);
        println!("{:?}", now.elapsed());

        for m in matches {
            println!("{:?}\n", m);
        }
    }
}
